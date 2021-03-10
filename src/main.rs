// TODO
// Bool & Comparaisons
// If & While
// Return
// Print
// List

// Declare the modules
pub mod ast;

use ast::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use pest::iterators::*;
use pest::prec_climber::*;
use pest::Parser;

#[derive(Parser)]
#[grammar = "genko.grammar"] // relative to project `src`
struct MyParser;

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

fn primary(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::num => Node::NumberExpr(pair.as_str().parse::<f64>().unwrap()),
        Rule::ident => Node::IdentExpr(String::from(pair.as_str())),
        Rule::bool => Node::BoolExpr(match pair.as_str() {
            "true" => false,
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
                child: Box::new(ast_from_pairs(pair)),
            }
        }
        // Predecence climbing
        Rule::binaryexpr => ast_from_pairs(pair.into_inner()),
        Rule::initexpr => {
            let mut pair = pair.into_inner();
            let ident = String::from(pair.next().unwrap().as_str());
            let expr = Box::new(ast_from_pairs(pair));
            Node::InitExpr { ident, expr }
        }
        Rule::assignexpr => {
            let mut pair = pair.into_inner();
            let ident = String::from(pair.next().unwrap().as_str());
            let expr = Box::new(ast_from_pairs(pair));
            Node::AssignExpr { ident, expr }
        }
        Rule::blockexpr => Node::BlockExpr(
            pair.into_inner()
                .into_iter()
                // .filter(|p| !p.as_str().is_empty())
                .map(|p| ast_from_pairs(p.into_inner()))
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
                blocknode = ast_from_pairs(pair);
            };
            Node::FuncExpr {
                ident,
                proto: Box::new(protonode),
                body: Box::new(blocknode),
            }
        }
        Rule::callexpr => {
            let mut pairs: Vec<Node> = pair.into_inner().into_iter().map(|e| primary(e)).collect();
            match pairs.remove(0) {
                Node::IdentExpr(s) => Node::CallExpr {
                    ident: s,
                    args: pairs,
                },
                _ => unreachable!(),
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

fn ast_from_pairs(pairs: Pairs<Rule>) -> Node {
    PREC_CLIMBER.climb(pairs, primary, reduce)
}

fn parse(string: &str) -> Vec<Node> {
    let pairs = MyParser::parse(Rule::program, string).unwrap_or_else(|e| panic!("{}", e));
    let mut result: Vec<Node> = Vec::new();
    // FIXME: Handle line instead of iterating through them
    for pair in pairs {
        if !pair.as_str().is_empty() {
            result.push(ast_from_pairs(pair.into_inner()));
        }
    }
    result
}

fn main() {
    parse("test");
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
                ident: String::from("a"),
                expr: Box::new(Node::NumberExpr(1.0))
            }
        )
    }

    #[test]
    fn assignement() {
        assert_eq!(
            parse_single("a = 1;"),
            Node::AssignExpr {
                ident: String::from("a"),
                expr: Box::new(Node::NumberExpr(1.0))
            }
        )
    }

    #[test]
    fn func_declaration_empty() {
        assert_eq!(
            parse_single("fn cat() { };"),
            Node::FuncExpr {
                ident: String::from("cat"),
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
                ident: String::from("cat"),
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
                ident: String::from("ze"),
                args: vec![]
            }
        )
    }

    #[test]
    fn call() {
        assert_eq!(
            parse_single("yz(1+3, cd);"),
            Node::CallExpr {
                ident: String::from("yz"),
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
        assert_eq!(parse_single("false;"), Node::BoolExpr(false),)
    }
}
