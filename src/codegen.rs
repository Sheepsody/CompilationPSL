use super::ast::{Node, Op};
use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{BasicTypeEnum, FloatType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, OptimizationLevel};

type JitFunc = unsafe extern "C" fn() -> f64;

struct RecursiveBuilder<'ctx> {
    f64_type: FloatType<'ctx>,
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value: &'ctx FunctionValue<'ctx>,
}

impl<'ctx> RecursiveBuilder<'ctx> {
    pub fn new(
        f64_type: FloatType<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder,
        fn_value: &'ctx FunctionValue,
    ) -> Self {
        Self {
            f64_type,
            builder,
            context,
            variables: HashMap::new(),
            fn_value,
        }
    }

    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.fn_value.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    pub fn build(&mut self, node: &Node) -> Result<FloatValue<'ctx>, &'static str> {
        match node {
            Node::NumberExpr(nb) => Ok(self.f64_type.const_float(*nb)),
            Node::IdentExpr(name) => match self.variables.get(name.as_str()) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, name.as_str())
                    .into_float_value()),
                None => Err("Could not find a matching variable."),
            },
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs = self.build(lhs).unwrap();
                let rhs = self.build(rhs).unwrap();
                match op {
                    Op::Add => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    Op::Sub => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    // TODO: Add other ops
                    _ => unimplemented!(),
                }
            }
            Node::InitExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let expr = self.build(expr);
                    let alloca = self.create_entry_block_alloca(name);

                    self.builder.build_store(alloca, expr.ok().unwrap());

                    self.variables.insert(name.to_string(), alloca);
                    return expr;
                } else {
                    unimplemented!()
                }
            }
            Node::AssignExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let nval = self.build(expr).unwrap();
                    let var = self
                        .variables
                        .get(name.as_str())
                        .ok_or("Undefined variable.")
                        .unwrap();
                    self.builder.build_store(*var, nval);
                    Ok(nval)
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
    let module = context.create_module("GenKo");
    let builder = context.create_builder();

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    let i64_type = context.f64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let function = module.add_function("jit", fn_type, None);

    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    let mut result: Result<FloatValue, &str> = Result::Err("No return specified");

    let mut recursive_builder = RecursiveBuilder::new(i64_type, &context, &builder, &function);
    for node in parse(string) {
        result = recursive_builder.build(&node);
    }
    builder.build_return(Some(&result.ok().unwrap()));

    unsafe {
        let jit_function: JitFunction<JitFunc> = execution_engine.get_function("jit").unwrap();
        jit_function.call()
    }
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
        assert_eq!(execute("let a = 2+2;a;"), 4.0)
    }

    #[test]
    fn var_use() {
        assert_eq!(execute("let a = 10; a+3;"), 13.0)
    }

    #[test]
    fn var_assign() {
        assert_eq!(execute("let a = 10; a = 8-3; a+3;"), 8.0)
    }
}
