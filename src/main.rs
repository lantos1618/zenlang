use inkwell::context::Context;
use inkwell::targets::{Target, TargetMachine};
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use std::path::Path;
use std::io::{self, Write, BufRead};
use std::env;

mod ast;
mod compiler;
mod error;

use crate::compiler::{Compiler, lexer::Lexer, parser::Parser};
use crate::error::{Result, CompileError};

fn main() -> std::io::Result<()> {
    // Initialize LLVM
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("LLVM initialization failed: {}", e)))?;
    
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => {
            // No arguments - start REPL
            run_repl()?;
        }
        2 => {
            // One argument - treat as file path
            let file_path = &args[1];
            if file_path == "--help" || file_path == "-h" {
                print_usage();
                return Ok(());
            }
            run_file(file_path)?;
        }
        _ => {
            print_usage();
            return Ok(());
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("Zen Language Compiler");
    println!();
    println!("Usage:");
    println!("  zen                    Start interactive REPL");
    println!("  zen <file.zen>         Compile and run a Zen file");
    println!("  zen --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  zen                    # Start REPL");
    println!("  zen hello.zen          # Run hello.zen file");
}

fn run_repl() -> std::io::Result<()> {
    println!("🎉 Welcome to the Zen REPL!");
    println!("Type Zen code and press Enter to execute.");
    println!("Type 'exit' or 'quit' to exit.");
    println!("Type 'help' for available commands.");
    println!();
    
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    
    loop {
        print!("zen> ");
        stdout.flush()?;
        
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        
        let input = input.trim();
        
        match input {
            "exit" | "quit" => {
                println!("Goodbye! 👋");
                break;
            }
            "help" => {
                print_repl_help();
                continue;
            }
            "clear" => {
                // Clear screen (simple version)
                print!("\x1B[2J\x1B[1;1H");
                stdout.flush()?;
                continue;
            }
            "" => continue,
            _ => {
                // Try to parse and execute the input
                match execute_zen_code(&mut compiler, input) {
                    Ok(result) => {
                        if let Some(value) = result {
                            println!("=> {}", value);
                        }
                    }
                    Err(e) => {
                        println!("❌ Error: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn run_file(file_path: &str) -> std::io::Result<()> {
    println!("📁 Compiling and running: {}", file_path);
    
    // Read the file
    let source = std::fs::read_to_string(file_path)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to read file: {}", e)))?;
    
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    
    match execute_zen_code(&mut compiler, &source) {
        Ok(result) => {
            if let Some(value) = result {
                println!("=> {}", value);
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation error: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn execute_zen_code(compiler: &mut Compiler, source: &str) -> Result<Option<String>> {
    // Reset the compiler for each execution
    let context = compiler.context;
    *compiler = Compiler::new(context);
    
    // Parse the source
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program()
        .map_err(|e| CompileError::InternalError(format!("Parse error: {}", e), None))?;
    
    if program.declarations.is_empty() {
        return Ok(None);
    }
    
    // Compile the program
    compiler.compile_program(&program)?;
    
    // Verify the LLVM IR
    if let Err(errors) = compiler.module.verify() {
        return Err(CompileError::InternalError(
            format!("LLVM IR verification failed: {}", errors.to_string()),
            None
        ));
    }
    
    // JIT execution: run main() if present
    let execution_engine = compiler.module.create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| CompileError::InternalError(format!("Failed to create JIT engine: {}", e), None))?;
    if let Some(main_fn) = compiler.module.get_function("main") {
        let ret_type = main_fn.get_type().get_return_type();
        if let Some(ty) = ret_type {
            match ty {
                inkwell::types::BasicTypeEnum::IntType(int_ty) => {
                    let width = int_ty.get_bit_width();
                    if width == 32 {
                        type MainFuncI32 = unsafe extern "C" fn() -> i32;
                        let result = unsafe {
                            let jit_fn: JitFunction<MainFuncI32> = execution_engine.get_function("main")
                                .map_err(|e| CompileError::InternalError(format!("JIT lookup failed: {}", e), None))?;
                            jit_fn.call()
                        };
                        return Ok(Some(result.to_string()));
                    } else if width == 64 {
                        type MainFuncI64 = unsafe extern "C" fn() -> i64;
                        let result = unsafe {
                            let jit_fn: JitFunction<MainFuncI64> = execution_engine.get_function("main")
                                .map_err(|e| CompileError::InternalError(format!("JIT lookup failed: {}", e), None))?;
                            jit_fn.call()
                        };
                        return Ok(Some(result.to_string()));
                    } else {
                        return Ok(Some("[main returns unsupported int width]".to_string()));
                    }
                }
                _ => {
                    return Ok(Some("[main returns unsupported type]".to_string()));
                }
            }
        } else {
            // void return
            type MainFuncVoid = unsafe extern "C" fn();
            let _ = unsafe {
                let jit_fn: JitFunction<MainFuncVoid> = execution_engine.get_function("main")
                    .map_err(|e| CompileError::InternalError(format!("JIT lookup failed: {}", e), None))?;
                jit_fn.call()
            };
            return Ok(Some("[main returned void]".to_string()));
        }
    }
    Ok(Some("[no main function to execute]".to_string()))
}

fn print_repl_help() {
    println!("Available commands:");
    println!("  help                    Show this help");
    println!("  clear                   Clear the screen");
    println!("  exit, quit              Exit the REPL");
    println!();
    println!("Zen code examples:");
    println!("  main = () i32 {{ 42 }}");
    println!("  add = (a: i32, b: i32) i32 {{ a + b }}");
    println!("  x := 10; y := 20; x + y");
    println!();
} 