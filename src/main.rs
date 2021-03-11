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

use codegen::CodeGen;
use parser::parse;

use inkwell::context::Context;
use inkwell::OptimizationLevel;

fn main() -> Result<(), ()> {
    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let sum = codegen.jit_compile_sum(parse("1;").get(0).unwrap()).ok();

    println!("{}", sum.unwrap().get_constant().unwrap().0);

    Ok(())
}
