// Declare the modules
pub mod ast;
pub mod parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use parser::parse;

fn main() {
    // TODO : Load a file
    // TODO : AST Tree printer ?
    parse("test");
}
