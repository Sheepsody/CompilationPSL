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
    variables: HashMap<String, FloatValue<'ctx>>,
    // variables: HashMap<String, PointerValue<'ctx>>,
    // fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn jit_compile(&mut self, node: &Node) -> Result<FloatValue<'ctx>, &'static str> {
        match node {
            Node::NumberExpr(nb) => Ok(self.context.f64_type().const_float(*nb)),
            Node::IdentExpr(name) => match self.variables.get(name.as_str()) {
                Some(value) => Ok(*value),
                None => Err("Could not find a matching variable."),
            },
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs = self.jit_compile(lhs).unwrap();
                let rhs = self.jit_compile(rhs).unwrap();
                match op {
                    Op::Add => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    // TODO: Add other ops
                    _ => unimplemented!(),
                }
            }
            Node::InitExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let expr = self.jit_compile(expr);
                    self.variables.insert(name.clone(), expr.unwrap());
                    //                    self.builder.build_store(alloca, expr.unwrap());
                    return expr;
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
        }
    }
}

pub fn execute(string: &str) -> f64 {
    use super::parser::parse;

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let mut codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
        variables: HashMap::new(),
    };

    let mut result = 0.0;
    for line in parse(string) {
        result = codegen
            .jit_compile(&line)
            .ok()
            .unwrap()
            .get_constant()
            .unwrap()
            .0
    }

    result
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

    #[test]
    fn var_init() {
        assert_eq!(execute("let a = 2+2;"), 4.0)
    }

    #[test]
    fn var_use() {
        assert_eq!(execute("let a = 10; a+3;"), 13.0)
    }
}
