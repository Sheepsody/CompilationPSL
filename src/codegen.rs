use super::ast::Node;
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
        match *node {
            Node::NumberExpr(nb) => Ok(self.context.f64_type().const_float(nb)),
            _ => unimplemented!(),
        }
    }
}
