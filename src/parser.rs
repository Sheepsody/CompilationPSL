// TODO
// Print
// Else as Option
// Return
// While
// Listes

use super::ast::*;

use pest::iterators::*;
use pest::prec_climber::*;
use pest::Parser;

#[derive(Parser)]
#[grammar = "genko.grammar"] // relative to project `src`
struct GenkoParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(sub, Left),
            Operator::new(mul, Left) | Operator::new(div, Left),
            Operator::new(pow, Right),
            Operator::new(eq, Left)
                | Operator::new(lt, Left)
                | Operator::new(le, Left)
                | Operator::new(gt, Left)
                | Operator::new(ge, Left)
                | Operator::new(and, Left)
                | Operator::new(or, Left),
        ])
    };
}

fn parse_pair(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::num => Node::NumberExpr(pair.as_str().parse::<f64>().unwrap()),
        Rule::ident => Node::IdentExpr(String::from(pair.as_str())),
        Rule::bool => Node::BoolExpr(match pair.as_str() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        }),
        Rule::unaryexpr => {
            let mut pair = pair.into_inner();
            let operator = match pair.next().unwrap().as_rule() {
                Rule::sub => Op::Sub,
                _ => unimplemented!(),
            };
            Node::UnaryExpr {
                op: operator,
                child: Box::new(parse_pairs(pair)),
            }
        }
        // Predecence climbing
        Rule::binaryexpr => parse_pairs(pair.into_inner()),
        Rule::initexpr => {
            let mut pair = pair.into_inner();
            let ident = Box::new(Node::IdentExpr(String::from(pair.next().unwrap().as_str())));
            let expr = Box::new(parse_pairs(pair));
            Node::InitExpr { ident, expr }
        }
        Rule::assignexpr => {
            let mut pair = pair.into_inner();
            let ident = Box::new(Node::IdentExpr(String::from(pair.next().unwrap().as_str())));
            let expr = Box::new(parse_pairs(pair));
            Node::AssignExpr { ident, expr }
        }
        Rule::blockexpr => Node::BlockExpr(
            pair.into_inner()
                .into_iter()
                // .filter(|p| !p.as_str().is_empty())
                .map(|p| parse_pairs(p.into_inner()))
                .collect(),
        ),
        Rule::protoexpr => Node::ProtoExpr(vec![]),
        Rule::funcexpr => {
            let mut pair = pair.into_inner();
            let ident = String::from(pair.next().unwrap().as_str());
            let proto = pair.next().unwrap();
            let protonode = Node::ProtoExpr(
                proto
                    .into_inner()
                    .into_iter()
                    .map(|p| String::from(p.as_str()))
                    .collect(),
            );
            let mut blocknode = Node::BlockExpr(vec![]);
            if !pair.as_str().is_empty() {
                blocknode = parse_pairs(pair);
            };
            Node::FuncExpr {
                ident: Box::new(Node::IdentExpr(ident)),
                proto: Box::new(protonode),
                body: Box::new(blocknode),
            }
        }
        Rule::callexpr => {
            let mut pairs: Vec<Node> = pair
                .into_inner()
                .into_iter()
                .map(|e| parse_pair(e))
                .collect();
            let ident = pairs.remove(0);
            Node::CallExpr {
                ident: Box::new(ident),
                args: pairs,
            }
        }
        Rule::condexpr => {
            let mut pair = pair.into_inner();
            let cond = parse_pair(pair.next().unwrap());
            let cons = parse_pair(pair.next().unwrap());
            let alter = match pair.next() {
                Some(p) => Option::Some(Box::new(parse_pair(p))),
                None => None,
            };
            Node::CondExpr {
                cond: Box::new(cond),
                cons: Box::new(cons),
                alter,
            }
        }
        _ => unreachable!(),
    }
}

fn reduce(lhs: Node, op: Pair<Rule>, rhs: Node) -> Node {
    let operator = match op.as_rule() {
        Rule::add => Op::Add,
        Rule::sub => Op::Sub,
        Rule::mul => Op::Mul,
        Rule::div => Op::Div,
        Rule::pow => Op::Pow,
        Rule::eq => Op::Eq,
        Rule::lt => Op::Lt,
        Rule::gt => Op::Gt,
        Rule::le => Op::Le,
        Rule::ge => Op::Ge,
        Rule::and => Op::And,
        Rule::or => Op::Or,
        _ => unreachable!(),
    };
    Node::BinaryExpr {
        op: operator,
        rhs: Box::new(rhs),
        lhs: Box::new(lhs),
    }
}

fn parse_pairs(pairs: Pairs<Rule>) -> Node {
    PREC_CLIMBER.climb(pairs, parse_pair, reduce)
}

pub fn parse(string: &str) -> Vec<Node> {
    let pairs = GenkoParser::parse(Rule::program, string).unwrap_or_else(|e| panic!("{}", e));

    pairs
        .into_iter()
        .filter(|p| !p.as_str().is_empty())
        .map(|p| parse_pair(p))
        .collect()
}

#[cfg(test)]
mod parsing {
    use super::*;

    fn parse_single(string: &str) -> Node {
        parse(string).remove(0)
    }

    #[test]
    fn number() {
        assert_eq!(parse_single("1;"), Node::NumberExpr(1.0));
    }

    #[test]
    fn block() {
        assert_eq!(
            parse_single("{1; 2; 3;};"),
            Node::BlockExpr(vec![
                Node::NumberExpr(1.0),
                Node::NumberExpr(2.0),
                Node::NumberExpr(3.0)
            ])
        );
    }

    #[test]
    fn binary() {
        assert_eq!(
            parse_single("1+2;"),
            Node::BinaryExpr {
                op: Op::Add,
                lhs: Box::new(Node::NumberExpr(1.0)),
                rhs: Box::new(Node::NumberExpr(2.0))
            }
        )
    }

    #[test]
    fn identifier() {
        assert_eq!(parse_single("x;"), Node::IdentExpr(String::from("x")))
    }

    #[test]
    fn initialisation() {
        assert_eq!(
            parse_single("let a = 1;"),
            Node::InitExpr {
                ident: Box::new(Node::IdentExpr(String::from("a"))),
                expr: Box::new(Node::NumberExpr(1.0))
            }
        )
    }

    #[test]
    fn assignement() {
        assert_eq!(
            parse_single("a = 1;"),
            Node::AssignExpr {
                ident: Box::new(Node::IdentExpr(String::from("a"))),
                expr: Box::new(Node::NumberExpr(1.0))
            }
        )
    }

    #[test]
    fn func_declaration_empty() {
        assert_eq!(
            parse_single("fn cat() { };"),
            Node::FuncExpr {
                ident: Box::new(Node::IdentExpr(String::from("cat"))),
                proto: Box::new(Node::ProtoExpr(vec![])),
                body: Box::new(Node::BlockExpr(vec![])),
            }
        )
    }

    #[test]
    fn func_declaration() {
        assert_eq!(
            parse_single("fn cat(a, b) {6; 7;};"),
            Node::FuncExpr {
                ident: Box::new(Node::IdentExpr(String::from("cat"))),
                proto: Box::new(Node::ProtoExpr(vec![String::from("a"), String::from("b")])),
                body: Box::new(Node::BlockExpr(vec![
                    Node::NumberExpr(6.0),
                    Node::NumberExpr(7.0)
                ])),
            }
        )
    }

    #[test]
    fn call_empty() {
        assert_eq!(
            parse_single("ze();"),
            Node::CallExpr {
                ident: Box::new(Node::IdentExpr(String::from("ze"))),
                args: vec![]
            }
        )
    }

    #[test]
    fn call() {
        assert_eq!(
            parse_single("yz(1+3, cd);"),
            Node::CallExpr {
                ident: Box::new(Node::IdentExpr(String::from("yz"))),
                args: vec![
                    Node::BinaryExpr {
                        lhs: Box::new(Node::NumberExpr(1.0)),
                        op: Op::Add,
                        rhs: Box::new(Node::NumberExpr(3.0)),
                    },
                    Node::IdentExpr(String::from("cd")),
                ]
            }
        )
    }

    #[test]
    fn bool_false() {
        assert_eq!(parse_single("false;"), Node::BoolExpr(false),)
    }

    #[test]
    fn bool_true() {
        assert_eq!(parse_single("true;"), Node::BoolExpr(true),)
    }

    #[test]
    fn cond_if() {
        assert_eq!(
            parse_single("if true then {1+3;};"),
            Node::CondExpr {
                cond: Box::new(Node::BoolExpr(true)),
                cons: Box::new(Node::BlockExpr(vec![Node::BinaryExpr {
                    lhs: Box::new(Node::NumberExpr(1.0)),
                    op: Op::Add,
                    rhs: Box::new(Node::NumberExpr(3.0)),
                },])),
                alter: Option::None,
            }
        )
    }

    #[test]
    fn cond_if_else() {
        assert_eq!(
            parse_single("if true then {6;} else {4;};"),
            Node::CondExpr {
                cond: Box::new(Node::BoolExpr(true)),
                cons: Box::new(Node::BlockExpr(vec![Node::NumberExpr(6.0),])),
                alter: Some(Box::new(Node::BlockExpr(vec![Node::NumberExpr(4.0),]))),
            }
        )
    }
}
