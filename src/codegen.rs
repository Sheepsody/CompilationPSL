// TODO
// Declare new functions (no access to global variables)
// Call functions

use super::ast::{Node, Op};
use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{BasicTypeEnum, FloatType, FunctionType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, OptimizationLevel};

type JitFunc = unsafe extern "C" fn() -> f64;

struct RecursiveBuilder<'a, 'ctx> {
    f64_type: FloatType<'ctx>,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    fn_value: &'a FunctionValue<'ctx>,

    pub variables: HashMap<String, PointerValue<'ctx>>,
    pub current_block: BasicBlock<'ctx>,
}

impl<'a, 'ctx> RecursiveBuilder<'a, 'ctx> {
    pub fn new(
        f64_type: FloatType<'ctx>,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        fn_value: &'a FunctionValue<'ctx>,
        current_block: BasicBlock<'ctx>,
    ) -> Self {
        Self {
            f64_type,
            builder,
            module,
            context,
            variables: HashMap::new(),
            fn_value,
            current_block,
        }
    }

    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
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

    pub fn build(&mut self, node: &Node) -> Option<FloatValue<'ctx>> {
        match node {
            Node::NumberExpr(nb) => Some(self.f64_type.const_float(*nb)),
            Node::IdentExpr(name) => match self.variables.get(name.as_str()) {
                Some(var) => Some(
                    self.builder
                        .build_load(*var, name.as_str())
                        .into_float_value(),
                ),
                None => unreachable!("Could not find a matching variable."),
            },
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs = self.build(lhs).unwrap();
                let rhs = self.build(rhs).unwrap();
                match op {
                    Op::Add => Some(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    Op::Sub => Some(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    // TODO: Add other ops
                    _ => unimplemented!(),
                }
            }
            Node::InitExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let expr = self.build(expr);
                    let alloca = self.create_entry_block_alloca(name);

                    self.builder.build_store(alloca, expr.unwrap());

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
                    Some(nval)
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

                self.current_block = cont_bb;

                Some(phi.as_basic_value().into_float_value())
            }

            Node::BlockExpr(nodes) => {
                let mut result: Option<FloatValue> = None;
                for node in nodes {
                    result = self.build(node);
                }
                result
            }

            Node::FuncExpr { ident, args, body } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let last = self.builder.get_insert_block();

                    // Compiling the prototype
                    let args_types = std::iter::repeat(self.f64_type)
                        .take(args.len())
                        .map(|f| f.into())
                        .collect::<Vec<BasicTypeEnum>>();

                    let args_types = args_types.as_slice();
                    let fn_type = self.f64_type.fn_type(args_types, false);

                    let function = self.module.add_function(name, fn_type, None);

                    for (i, arg) in function.get_param_iter().enumerate() {
                        arg.into_float_value().set_name(args[i].as_str());
                    }

                    // Add function block
                    let entry = self.context.append_basic_block(function, "entry");
                    self.builder.position_at_end(entry);

                    // Build variable map
                    self.variables.reserve(args.len());
                    for (i, arg) in function.get_param_iter().enumerate() {
                        let arg_name = args[i].as_str();
                        let alloca = self.create_entry_block_alloca(arg_name);
                        self.builder.build_store(alloca, arg);
                        self.variables.insert(args[i].clone(), alloca);
                    }
                    // Compile Body
                    let body = self.build(body).unwrap();

                    // Return
                    let ret = self.builder.build_return(Some(&body));

                    Some(self.f64_type.const_float(0.0))
                } else {
                    unimplemented!()
                }
            }

            Node::CallExpr { ident, args } => {
                println!("Got here");
                if let Node::IdentExpr(name) = ident.as_ref() {
                    match self.get_function(name) {
                        Some(fun) => {
                            let mut compiled_args = Vec::with_capacity(args.len());

                            for arg in args {
                                compiled_args.push(self.build(arg)?);
                            }

                            let argsv: Vec<BasicValueEnum> = compiled_args
                                .iter()
                                .by_ref()
                                .map(|&val| val.into())
                                .collect();

                            match self
                                .builder
                                .build_call(fun, argsv.as_slice(), "tmp")
                                .try_as_basic_value()
                                .left()
                            {
                                Some(value) => Some(value.into_float_value()),
                                None => unreachable!("Invalid call produced."),
                            }
                        }
                        None => unreachable!("Unknown function."),
                    }
                } else {
                    unimplemented!();
                }
            }

            _ => unimplemented!("{:?}", node),
        }
    }

    fn reposition(&mut self) {
        self.builder.position_at_end(self.current_block);
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

    let mut current_block = context.append_basic_block(function, "entry");

    let mut result: Option<FloatValue> = None;

    let mut recursive_builder = RecursiveBuilder::new(
        f64_type,
        &context,
        &module,
        &builder,
        &function,
        current_block,
    );

    for node in parse(string) {
        recursive_builder.reposition();
        result = recursive_builder.build(&node);
    }

    builder.build_return(Some(&result.unwrap()));

    // module.print_to_stderr();

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
        assert_eq!(execute("1"), 1.0)
    }

    #[test]
    fn add() {
        assert_eq!(execute("1+2"), 3.0)
    }

    #[test]
    fn variables() {
        assert_eq!(execute("let a = 2+2; a"), 4.0)
    }

    #[test]
    fn fn_decl() {
        assert_eq!(execute("fn test() {1} 10"), 10.0)
    }

    #[test]
    fn fn_args() {
        assert_eq!(execute("fn test(a) {10+a} test(5)"), 15.0)
    }

    #[test]
    fn cond_true() {
        assert_eq!(
            execute("let a=1; if (1) then {a = 3;} else {a = 4;} a"),
            3.0
        )
    }

    fn recursive() {
        assert_eq!(
            execute("fn test(a) { let b=0; if a then {b=test(a-1);} else {b=a;} b} test(10)"),
            0.0
        )
    }
}
