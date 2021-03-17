// Declare the modules
pub mod ast;
pub mod codegen;
pub mod parser;

extern crate inkwell;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read_to_string(&args[1]).expect("Something went wrong reading the file");
    let result: f64 = codegen::execute(&contents);
    println!("Got result : {:?}", result);
    Ok(())
}
