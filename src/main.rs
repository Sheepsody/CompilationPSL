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
        Rule::num => Node::NumberExpr {
            value: pair.as_str().parse::<f64>().unwrap(),
        },
        Rule::ident => Node::IdentExpr {
            name: String::from(pair.as_str()),
        },
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
        assert_eq!(parse_single("1"), Node::NumberExpr { value: 1.0 });
    }

    #[test]
    fn binary() {
        assert_eq!(
            parse_single("1+2"),
            Node::BinaryExpr {
                op: Op::Add,
                lhs: Box::new(Node::NumberExpr { value: 1.0 }),
                rhs: Box::new(Node::NumberExpr { value: 2.0 })
            }
        )
    }

    #[test]
    fn identifier() {
        assert_eq!(
            parse_single("x"),
            Node::IdentExpr {
                name: String::from("x")
            }
        )
    }
}
