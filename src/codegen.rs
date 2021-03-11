// TODO
// Declare new functions (no access to global variables)
// Call functions

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

struct RecursiveBuilder<'a, 'ctx> {
    f64_type: FloatType<'ctx>,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    pub variables: HashMap<String, PointerValue<'ctx>>,
    fn_value: &'a FunctionValue<'ctx>,
}

impl<'a, 'ctx> RecursiveBuilder<'a, 'ctx> {
    pub fn new(
        f64_type: FloatType<'ctx>,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        fn_value: &'a FunctionValue<'ctx>,
    ) -> Self {
        Self {
            f64_type,
            builder,
            module,
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

            Node::CondExpr { cond, cons, alter } => {
                let parent = *self.fn_value;
                let zero_const = self.context.f64_type().const_float(0.0);

                // create condition by comparing without 0.0 and returning an int
                let cond = self.build(cond)?;
                let cond = self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    cond,
                    zero_const,
                    "ifcond",
                );

                // build branch
                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let cont_bb = self.context.append_basic_block(parent, "ifcont");

                self.builder
                    .build_conditional_branch(cond, then_bb, else_bb);

                // build then block
                self.builder.position_at_end(then_bb);
                let then_val = self.build(cons)?;
                self.builder.build_unconditional_branch(cont_bb);

                let then_bb = self.builder.get_insert_block().unwrap();

                // build else block
                self.builder.position_at_end(else_bb);
                // FIXME
                let mut else_val = self.f64_type.const_float(0.0);
                if let Some(node) = alter {
                    else_val = self.build(node).unwrap();
                }
                self.builder.build_unconditional_branch(cont_bb);

                let else_bb = self.builder.get_insert_block().unwrap();

                // emit merge block
                self.builder.position_at_end(cont_bb);

                let phi = self.builder.build_phi(self.context.f64_type(), "iftmp");

                phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

                Ok(phi.as_basic_value().into_float_value())
            }

            Node::BlockExpr(nodes) => {
                let mut result: Result<FloatValue, &str> = Result::Err("No return specified");
                for node in nodes {
                    result = self.build(node);
                }
                result
            }

            _ => unimplemented!("{:?}", node),
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

    let f64_type = context.f64_type();
    let fn_type = f64_type.fn_type(&[], false);
    let function = module.add_function("jit", fn_type, None);

    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    let mut result: Result<FloatValue, &str> = Result::Err("No return specified");

    let mut recursive_builder =
        RecursiveBuilder::new(f64_type, &context, &module, &builder, &function);
    for node in parse(string) {
        result = recursive_builder.build(&node);
    }
    builder.build_return(Some(&result.ok().unwrap()));

    module.print_to_stderr();

    // println!("{:?}", function.print_to_stderr());
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

    #[test]
    fn cond_true() {
        assert_eq!(execute("if (1) then {1+3;} else {1;};"), 4.0)
    }

    #[test]
    fn cond_false() {
        assert_eq!(execute("if (0) then {1+3;} else {1;};"), 1.0)
    }

    //    #[test]
    //    fn fn_decl() {
    //        assert_eq!(execute("fn test() {1;};"), 0.0)
    //    }
}
