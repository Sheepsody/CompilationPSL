// TODO : Add clean tests (pour les fonctionnalités jusque là)

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use pest::iterators::*;
use pest::prec_climber::*;
use pest::Parser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts;
use std::io::{self, Write};
use std::option::Option;
use std::rc::Rc;
use std::string::String;

type VarDict = HashMap<String, f64>;

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

fn eval(expression: Pairs<Rule>, dict: Rc<RefCell<Box<VarDict>>>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| -> f64 {
            match pair.as_rule() {
                Rule::num => pair.as_str().parse::<f64>().unwrap(),
                Rule::ident => *dict.borrow().get(pair.as_str()).unwrap(),
                Rule::cons => {
                    let mut pair = pair.into_inner();
                    match pair.next().unwrap().as_rule() {
                        Rule::pi => consts::PI,
                        _ => unreachable!(),
                    }
                }
                Rule::binary => eval(pair.into_inner(), dict.clone()),
                Rule::unary => {
                    let mut pair = pair.into_inner();
                    let op = pair.next().unwrap().as_rule();
                    let term = eval(pair, dict.clone());
                    match op {
                        Rule::add => term,
                        Rule::sub => -term,
                        _ => unreachable!(),
                    }
                }
                Rule::call => {
                    let mut pair = pair.into_inner();
                    let func = pair.next().unwrap().as_rule();
                    let term = eval(pair, dict.clone());
                    match func {
                        Rule::cos => term.cos(),
                        _ => unreachable!(),
                    }
                }
                Rule::bool => match pair.as_str() {
                    "true" => 1.0,
                    "false" => 0.0,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::sub => lhs - rhs,
            Rule::mul => lhs * rhs,
            Rule::div => lhs / rhs,
            Rule::pow => lhs.powf(rhs),
            Rule::eq => return if lhs == rhs { 1.0 } else { 0.0 },
            Rule::lt => return if lhs < rhs { 1.0 } else { 0.0 },
            Rule::gt => return if lhs > rhs { 1.0 } else { 0.0 },
            Rule::le => return if lhs <= rhs { 1.0 } else { 0.0 },
            Rule::ge => return if lhs >= rhs { 1.0 } else { 0.0 },
            Rule::and => return if lhs > 0.0 && rhs > 0.0 { 1.0 } else { 0.0 },
            Rule::or => return if lhs > 0.0 || rhs > 0.0 { 1.0 } else { 0.0 },
            _ => unreachable!(),
        },
    )
}

fn print_variables_dict(dict: Rc<RefCell<Box<VarDict>>>) {
    for (key, value) in dict.borrow().as_ref().into_iter() {
        println!("{} : {}", key, value);
    }
}

pub fn execute(string: &str, dict: Rc<RefCell<Box<VarDict>>>) -> Option<f64> {
    let mut output: Option<f64> = Option::None;
    let pairs = MyParser::parse(Rule::program, string).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        if !pair.as_str().is_empty() {
            match pair.as_rule() {
                Rule::init => {
                    let mut pair = pair.into_inner();
                    let ident = pair.next().unwrap().as_str();
                    dict.borrow_mut()
                        .insert(String::from(ident), eval(pair, dict.clone()));
                }
                // FIXME: Should we remove this kind of node and match it implicitely with _ ?
                Rule::exprast => {
                    output = Option::Some(eval(pair.into_inner(), dict.clone()));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
    output
}

fn main() {
    println!("言語 Calculator \n");

    let dict = Rc::new(RefCell::new(Box::new(VarDict::new())));
    loop {
        let mut s = String::new();
        print!(">>> ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut s)
            .expect("Excepted a correct String");

        if s == "close\n" {
            break;
        }

        if MyParser::parse(Rule::program, &s).is_err() {
            println!("Input not correct");
        } else {
            if let Some(result) = execute(&s, dict.clone()) {
                println!("Result : {}", result);
            }
        }
    }
    print_variables_dict(dict.clone());
}
