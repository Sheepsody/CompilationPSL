use super::ast::{Node, Op};
use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, OptimizationLevel};

/// Defines the `Expr` compiler.
pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    // variables: HashMap<String, PointerValue<'ctx>>,
    // fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn jit_compile_sum(&self, node: &Node) -> Result<FloatValue<'ctx>, &'static str> {
        match node {
            Node::NumberExpr(nb) => Ok(self.context.f64_type().const_float(*nb)),
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs = self.jit_compile_sum(lhs).unwrap();
                let rhs = self.jit_compile_sum(rhs).unwrap();
                match op {
                    Op::Add => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    // TODO: Add other ops
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}

pub fn execute(string: &str) -> f64 {
    use super::parse;
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

    codegen
        .jit_compile_sum(parse(string).get(0).unwrap())
        .ok()
        .unwrap()
        .get_constant()
        .unwrap()
        .0
}

#[cfg(test)]
mod codegen {
    use super::execute;

    #[test]
    fn float() {
        assert_eq!(execute("1;"), 1.0)
    }

    #[test]
    fn add() {
        assert_eq!(execute("1+2;"), 3.0)
    }
}
