use inkwell::context::Context;
use std::io::{self, Write, BufRead};
use std::env;

mod ast;
mod codegen;
mod compiler;
mod comptime;
mod error;
mod lexer;
mod lsp;
mod module_system;
mod parser;
mod stdlib;
mod typechecker;
mod type_system;

use zen::compiler::Compiler;
use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::error::{Result, CompileError};

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
        let bytes_read = stdin.read_line(&mut input)?;
        
        // Handle EOF (no bytes read)
        if bytes_read == 0 {
            println!("\nGoodbye! 👋");
            break;
        }
        
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
    // Parse the source
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program()
        .map_err(|e| CompileError::InternalError(format!("Parse error: {}", e), None))?;
    
    if program.declarations.is_empty() {
        return Ok(None);
    }
    
    // Compile the program using LLVM backend
    let llvm_ir = compiler.compile_llvm(&program)?;
    
    // For now, just return the LLVM IR as a string
    // TODO: Implement JIT execution in the future
    Ok(Some(format!("Compiled successfully!\nLLVM IR:\n{}", llvm_ir)))
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