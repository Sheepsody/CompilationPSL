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

fn main() -> Result<(), ()> {
    Ok(())
}
