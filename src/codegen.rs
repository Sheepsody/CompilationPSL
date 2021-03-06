use super::ast::{BinaryOp, Node, UnaryOp};
use super::parser::parse;
use std::collections::HashMap;
use std::f64::NAN;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FloatType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::{FloatPredicate, OptimizationLevel};

pub type JitFunc = unsafe extern "C" fn() -> f64;

struct RecursiveBuilder<'a, 'ctx> {
    f64_type: FloatType<'ctx>,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    context: &'ctx Context,

    pub fn_stack: Vec<FunctionValue<'ctx>>,
    pub var_stack: Vec<HashMap<String, PointerValue<'ctx>>>,
    pub block_stack: Vec<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> RecursiveBuilder<'a, 'ctx> {
    pub fn new(
        f64_type: FloatType<'ctx>,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        function: &'a FunctionValue<'ctx>,
        block_stack: BasicBlock<'ctx>,
    ) -> Self {
        Self {
            f64_type,
            builder,
            module,
            context,
            fn_stack: vec![*function],
            var_stack: vec![HashMap::new()],
            block_stack: vec![block_stack],
        }
    }

    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self
            .fn_stack
            .last()
            .unwrap()
            .get_first_basic_block()
            .unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    pub fn build(&mut self, node: &Node) -> Option<FloatValue<'ctx>> {
        // Resposition the "Write-Head"
        self.reposition();

        // Add the nodes
        match node {
            Node::NumberExpr(nb) => Some(self.f64_type.const_float(*nb)),
            Node::BoolExpr(b) => match b {
                true => Some(self.f64_type.const_float(1.0)),
                false => Some(self.f64_type.const_float(0.0)),
            },
            Node::IdentExpr(name) => {
                if let Some(var) = self.var_stack.last().unwrap().get(name.as_str()) {
                    return Some(
                        self.builder
                            .build_load(*var, name.as_str())
                            .into_float_value(),
                    );
                };
                if let Some(var) = self.module.get_global(name.as_str()) {
                    let load = var.as_pointer_value();
                    return Some(self.builder.build_load(load, "test").into_float_value());
                };
                unimplemented!("Could not find matching variable");
            }

            Node::UnaryExpr { op, child } => {
                let child = self.build(child).unwrap();
                match op {
                    UnaryOp::Sub => Some(self.builder.build_float_sub(
                        self.f64_type.const_float(0.0),
                        child,
                        "tmpsub",
                    )),
                    // Not a generalized not...
                    UnaryOp::Not => Some(self.builder.build_float_sub(
                        self.f64_type.const_float(1.0),
                        child,
                        "tmpnot",
                    )),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs = self.build(lhs).unwrap();
                let rhs = self.build(rhs).unwrap();
                match op {
                    BinaryOp::Add => Some(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    BinaryOp::Sub => Some(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    BinaryOp::Mul => Some(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    BinaryOp::Div => Some(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    BinaryOp::Pow => unimplemented!(),
                    BinaryOp::Eq => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::UEQ,
                            lhs,
                            rhs,
                            "tmpeq",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    BinaryOp::Ne => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::UNE,
                            lhs,
                            rhs,
                            "tmpne",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    BinaryOp::Lt => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::ULT,
                            lhs,
                            rhs,
                            "tmplt",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    BinaryOp::Gt => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::UGT,
                            lhs,
                            rhs,
                            "tmpgt",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    BinaryOp::Ge => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::UGE,
                            lhs,
                            rhs,
                            "tmpge",
                        );

                        self.builder
                            .build_unsigned_int_to_float(cmp, self.f64_type, "tmpbool")
                    }),
                    BinaryOp::Le => Some({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::ULE,
                            lhs,
                            rhs,
                            "tmple",
                        );

                        self.builder
                            .build_unsigned_int_to_float(cmp, self.f64_type, "tmpmodbool")
                    }),
                    BinaryOp::Modulo => {
                        let div = self.builder.build_float_div(lhs, rhs, "tmpmoddiv");
                        let cast = self.builder.build_float_to_signed_int(
                            div,
                            self.context.i64_type(),
                            "tmpint",
                        );
                        let cast = self.builder.build_signed_int_to_float(
                            cast,
                            self.f64_type,
                            "tmpmodtrunc",
                        );
                        let mul = self.builder.build_float_mul(rhs, cast, "tmpmodmul");
                        Some(self.builder.build_float_sub(lhs, mul, "tmpmod"))
                    }
                    BinaryOp::And => unimplemented!(),
                    BinaryOp::Or => unimplemented!(),
                }
            }
            Node::InitExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let expr = self.build(expr);
                    let alloca = self.create_entry_block_alloca(name);

                    self.builder.build_store(alloca, expr.unwrap());

                    self.var_stack
                        .last_mut()
                        .unwrap()
                        .insert(name.to_string(), alloca);
                    None
                } else {
                    unimplemented!()
                }
            }
            Node::GlobalInitExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let a = self
                        .module
                        .add_global(self.f64_type, Some(AddressSpace::Const), name);
                    a.set_initializer(&self.build(expr)?);
                    None
                } else {
                    unimplemented!()
                }
            }
            Node::AssignExpr { ident, expr } => {
                if let Node::IdentExpr(name) = ident.as_ref() {
                    let nval = self.build(expr).unwrap();

                    if let Some(var) = self.var_stack.last().unwrap().get(name.as_str()) {
                        self.builder.build_store(*var, nval);
                        return Some(nval);
                    }

                    if let Some(var) = self.module.get_global(name.as_str()) {
                        self.builder.build_store(var.as_pointer_value(), nval);
                        return Some(nval);
                    };

                    unreachable!("Could not find var {:?}", name.as_str());
                } else {
                    unimplemented!()
                }
            }

            Node::CondExpr { cond, cons, alter } => {
                let parent = *self.fn_stack.last().unwrap();
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
                self.block_stack.pop();
                self.block_stack.push(then_bb);
                self.reposition();

                let mut then_val = self.f64_type.const_float(NAN);
                if let Some(r) = self.build(cons) {
                    then_val = r;
                }
                self.builder.build_unconditional_branch(cont_bb);

                let then_bb = self.builder.get_insert_block().unwrap();

                // build else block
                self.block_stack.pop();
                self.block_stack.push(else_bb);
                self.reposition();
                // FIXME
                let mut else_val = self.f64_type.const_float(NAN);
                if let Some(node) = alter {
                    if let Some(r) = self.build(node) {
                        else_val = r;
                    }
                }
                self.builder.build_unconditional_branch(cont_bb);

                let else_bb = self.builder.get_insert_block().unwrap();

                // emit merge block
                self.block_stack.pop();
                self.block_stack.push(cont_bb);
                self.reposition();

                let phi = self.builder.build_phi(self.context.f64_type(), "iftmp");

                phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

                self.block_stack.pop();
                self.block_stack.push(cont_bb);

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
                    self.builder.get_insert_block();

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

                    self.fn_stack.push(function);
                    self.block_stack.push(entry);
                    self.var_stack.push(HashMap::new());
                    self.reposition();

                    // Build variable map
                    self.var_stack.reserve(args.len());
                    for (i, arg) in function.get_param_iter().enumerate() {
                        let arg_name = args[i].as_str();
                        let alloca = self.create_entry_block_alloca(arg_name);
                        self.builder.build_store(alloca, arg);
                        self.var_stack
                            .last_mut()
                            .unwrap()
                            .insert(args[i].clone(), alloca);
                    }

                    // Compile Body
                    self.build(body);

                    // FIXME : All functions must return...
                    self.builder.build_return(None);

                    self.fn_stack.pop();
                    self.block_stack.pop();
                    self.var_stack.pop();

                    self.reposition();

                    None
                } else {
                    unimplemented!()
                }
            }

            Node::ReturnExpr { ret } => {
                let ret = self.build(ret)?;
                self.builder.build_return(Some(&ret));
                None
            }

            Node::CallExpr { ident, args } => {
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

                            if fun.count_params() != (argsv.len() as u32) {
                                panic!("PLop");
                            }

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

            Node::WhileExpr {
                cond: condexpr,
                body,
            } => {
                let parent = *self.fn_stack.last().unwrap();
                let zero_const = self.context.f64_type().const_float(0.0);

                // build branch
                let loop_entry = self.context.append_basic_block(parent, "loop");
                let loop_exit = self.context.append_basic_block(parent, "exitloop");

                // Loop condition
                let cond = self.build(condexpr)?;
                let cond = self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    cond,
                    zero_const,
                    "loopcond",
                );

                self.builder
                    .build_conditional_branch(cond, loop_entry, loop_exit);

                self.block_stack.pop();
                self.block_stack.push(loop_entry);
                self.reposition();

                self.build(body);

                // Reloop
                let cond = self.build(condexpr)?;
                let cond = self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    cond,
                    zero_const,
                    "loopcond",
                );

                self.builder
                    .build_conditional_branch(cond, loop_entry, loop_exit);

                // Exit loop
                self.block_stack.pop();
                self.block_stack.push(loop_exit);
                self.reposition();

                None
            }
            _ => unimplemented!("{:?}", node),
        }
    }

    fn reposition(&mut self) {
        self.builder
            .position_at_end(*self.block_stack.last().unwrap());
    }
}

pub fn create_jit_module<'a>(context: &'a Context, string: &str) -> Module<'a> {
    let module = context.create_module("GenKo");

    let builder = context.create_builder();
    let f64_type = context.f64_type();
    let fn_type = f64_type.fn_type(&[], false);
    let function = module.add_function("jit", fn_type, None);

    let block_stack = context.append_basic_block(function, "entry");

    let mut result: Option<FloatValue> = None;

    let mut recursive_builder = RecursiveBuilder::new(
        f64_type,
        &context,
        &module,
        &builder,
        &function,
        block_stack,
    );

    for node in parse(string) {
        result = recursive_builder.build(&node);
    }

    match result {
        Some(r) => builder.build_return(Some(&r)),
        _ => builder.build_return(Some(&context.f64_type().const_float(NAN))),
    };

    module
}

pub fn execute(string: &str) -> f64 {
    let context = Context::create();

    let module = create_jit_module(&context, string);

    // The program is wrapped into a function to use JIT (Just In Time) compilation
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    // Uncomment to print LLVMIR Code
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
    fn not() {
        assert_eq!(execute("!false"), 1.0)
    }

    #[test]
    fn bool_true() {
        assert_eq!(execute("true"), 1.0)
    }

    #[test]
    fn bool_false() {
        assert_eq!(execute("false"), 0.0)
    }

    #[test]
    fn add() {
        assert_eq!(execute("1+2"), 3.0)
    }

    #[test]
    fn unary_sub() {
        assert_eq!(execute("let a=1; -a"), -1.0)
    }

    #[test]
    fn modulo() {
        assert_eq!(execute("10 % 3"), 1.0)
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(execute("2 < 1"), 0.0)
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(execute("1 >= 1"), 1.0)
    }

    #[test]
    fn variables() {
        assert_eq!(execute("let a = 2+2; a"), 4.0)
    }

    #[test]
    fn fn_decl() {
        assert_eq!(execute("fn test() {return 1;} 10"), 10.0)
    }

    #[test]
    fn fn_args() {
        assert_eq!(execute("fn test(a) {return 10+a;} test(5)"), 15.0)
    }

    #[test]
    fn fn_local() {
        assert_eq!(execute("let a=5; fn test() {let a=10;} test(); a"), 5.0)
    }

    #[test]
    #[should_panic]
    fn fn_invalid_params() {
        execute("fn test(a) {} test()");
    }

    #[test]
    fn if_then_cond() {
        assert_eq!(execute("let a=1; if (1 == 1) then {a = 3;} a"), 3.0)
    }

    #[test]
    fn if_then_else_cond() {
        assert_eq!(
            execute("let a=1; if (0 == 1) then {a = 3;} else {a=2;} a"),
            2.0
        )
    }

    #[test]
    fn if_then_else_cond_empty() {
        assert_eq!(execute("let a=1; if (0 == 1) then {} else {} a"), 1.0)
    }

    #[test]
    fn recursive() {
        assert_eq!(
            execute(
                "fn test(a) { let b=0; if a then {b=test(a-1);} else {b=a;} return b;} test(10)"
            ),
            0.0
        )
    }

    #[test]
    fn while_loop() {
        assert_eq!(
            execute("let a=2; let b=0; while (a!=0) {a=a-1; b=b+1;} b"),
            2.0
        )
    }

    #[test]
    fn global_var() {
        assert_eq!(
            execute("global a=2; a=3; fn test() {return a;} test()"),
            3.0
        )
    }
}
