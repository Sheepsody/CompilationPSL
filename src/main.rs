extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use pest::iterators::*;
use pest::prec_climber::*;
use pest::Parser;
use std::f64::consts;
use std::fs;

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to project `src`
struct MyParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right),
        ])
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| -> f64 {
            match pair.as_rule() {
                Rule::num => pair.as_str().parse::<f64>().unwrap(),
                Rule::cons => {
                    let mut pair = pair.into_inner();
                    match pair.next().unwrap().as_rule() {
                        Rule::pi => consts::PI,
                        _ => unreachable!(),
                    }
                }
                Rule::binary => eval(pair.into_inner()),
                Rule::unary => {
                    let mut pair = pair.into_inner();
                    let op = pair.next().unwrap().as_rule();
                    let term = eval(pair);
                    match op {
                        Rule::add => term,
                        Rule::subtract => -term,
                        _ => unreachable!(),
                    }
                }
                Rule::call => {
                    let mut pair = pair.into_inner();
                    let func = pair.next().unwrap().as_rule();
                    let term = eval(pair);
                    match func {
                        Rule::cos => term.cos(),
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide => lhs / rhs,
            Rule::power => lhs.powf(rhs),
            _ => unreachable!(),
        },
    )
}

fn main() {
    let file = fs::read_to_string("cal.test").expect("Cannot read");
    let pairs = MyParser::parse(Rule::program, &file).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        // FIXME: Is there a better way ? (Although this works all the time)
        match pair.as_str() {
            "" => (),
            _ => println!("{}", eval(pair.into_inner())),
        }
    }
}
