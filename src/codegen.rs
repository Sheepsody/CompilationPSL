// TODO
// Declare new functions (no access to global variables)
// Call functions

use super::ast::{BinaryOp, Node, UnaryOp};
use std::{collections::HashMap, f64::NAN};

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FloatType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, OptimizationLevel};

type JitFunc = unsafe extern "C" fn() -> f64;

struct RecursiveBuilder<'a, 'ctx> {
    f64_type: FloatType<'ctx>,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    pub fn_stack: Vec<FunctionValue<'ctx>>,

    pub variables: HashMap<String, PointerValue<'ctx>>,
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
            variables: HashMap::new(),
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
            Node::IdentExpr(name) => match self.variables.get(name.as_str()) {
                Some(var) => Some(
                    self.builder
                        .build_load(*var, name.as_str())
                        .into_float_value(),
                ),
                None => unreachable!("Could not find a matching variable."),
            },
            Node::UnaryExpr { op, child } => {
                let child = self.build(child).unwrap();
                match op {
                    UnaryOp::Sub => Some(self.builder.build_float_sub(
                        self.f64_type.const_float(0.0),
                        child,
                        "tmpsub",
                    )),
                    UnaryOp::Not => Some(self.builder.build_float_sub(
                        self.f64_type.const_float(1.0),
                        child,
                        "tmpnot",
                    )),
                    _ => unimplemented!("Unary operator {:?} not implemented...", op),
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
                    _ => unimplemented!("Binary operator {:?} not implemented...", op),
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

                let then_val = self.build(cons)?;
                self.builder.build_unconditional_branch(cont_bb);

                let then_bb = self.builder.get_insert_block().unwrap();

                // build else block
                self.block_stack.pop();
                self.block_stack.push(else_bb);
                self.reposition();
                // FIXME
                let mut else_val = self.f64_type.const_float(NAN);
                if let Some(node) = alter {
                    else_val = self.build(node).unwrap();
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
                    self.reposition();

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
                    self.builder.build_return(Some(&body));

                    self.fn_stack.pop();
                    self.block_stack.pop();

                    Some(self.f64_type.const_float(NAN))
                } else {
                    unimplemented!()
                }
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

                Some(self.f64_type.const_float(NAN))
            }

            _ => unimplemented!("{:?}", node),
        }
    }

    fn reposition(&mut self) {
        self.builder
            .position_at_end(*self.block_stack.last().unwrap());
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

    builder.build_return(Some(&result.unwrap()));

    //    module.print_to_stderr();

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
        assert_eq!(execute("!true"), 0.0)
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
        assert_eq!(execute("fn test() {1} 10"), 10.0)
    }

    #[test]
    fn fn_args() {
        assert_eq!(execute("fn test(a) {10+a} test(5)"), 15.0)
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
    fn recursive() {
        assert_eq!(
            execute("fn test(a) { let b=0; if a then {b=test(a-1);} else {b=a;} b} test(10)"),
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
}
