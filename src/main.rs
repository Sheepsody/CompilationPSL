// Declare the modules
pub mod ast;
pub mod codegen;
pub mod parser;

extern crate clap;
extern crate inkwell;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use clap::Clap;
use codegen::{create_jit_module, JitFunc};
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::OptimizationLevel;
use std::fs;
use std::io::{self, Write};
use std::string::String;

#[derive(Clap)]
#[clap(version = "1.0", author = "Sheepsody")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}
#[derive(Clap)]
enum SubCommand {
    Comp(Compile),
    Jit,
}

#[derive(Clap)]
struct Compile {
    #[clap(short, long)]
    file: String,
    #[clap(short, long)]
    ir: Option<String>,
}

fn compile_file(file: &str, ir: Option<String>) -> Result<f64, &str> {
    match fs::read_to_string(file) {
        Ok(content) => {
            let context = Context::create();
            let module = create_jit_module(&context, &content);

            let execution_engine = module
                .create_jit_execution_engine(OptimizationLevel::None)
                .unwrap();

            if let Some(filename) = ir {
                println!("LLVM IR Code:");
                match module.print_to_file(filename) {
                    Ok(_) => println!("Saved LLVM IR"),
                    Err(_) => eprintln!("Could not save LLVM IR"),
                }
            }

            unsafe {
                let jit_function: JitFunction<JitFunc> =
                    execution_engine.get_function("jit").unwrap();
                Ok(jit_function.call())
            }
        }
        _ => Err("Could not open file."),
    }
}

// FIXME: This kind of jit re-evaluates the while program at each step...
fn jit() -> Result<(), &'static str> {
    println!("言語 JIT\n");
    let mut history = String::new();

    loop {
        let mut s = String::new();
        print!(">>> ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut s)
            .expect("Excepted a correct String");

        if s.starts_with("close") {
            return Ok(());
        }

        let content = history.clone() + &s;

        let context = Context::create();
        let module = create_jit_module(&context, &content);

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let r;
        unsafe {
            let jit_function: JitFunction<JitFunc> = execution_engine.get_function("jit").unwrap();
            r = jit_function.call();
        }

        if r.is_nan() {
            history.push_str(&s);
        } else {
            println!("{}", r);
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Comp(comp) => match compile_file(&comp.file, comp.ir) {
            Ok(r) => println!("Got result : {}", r),
            Err(s) => eprintln!("{}", s),
        },
        SubCommand::Jit => match jit() {
            Ok(_) => (),
            Err(s) => eprintln!("{}", s),
        },
    }
}
