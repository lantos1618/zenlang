use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::targets::{CodeModel, FileType, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

use zen::compiler::Compiler;
use zen::error::{CompileError, Result};
use zen::lexer::Lexer;
use zen::parser::Parser;

fn main() -> std::io::Result<()> {
    // Initialize LLVM
    Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| io::Error::other(format!("LLVM initialization failed: {}", e)))?;

    // CRITICAL: Force MCJIT linkage to prevent LTO dead code elimination.
    // Without this call, LTO removes MCJIT's static constructors that register
    // the JIT backend with LLVM's target registry, causing segfaults.
    ExecutionEngine::link_in_mc_jit();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No arguments - start REPL
            run_repl()?;
        }
        2 => {
            // One argument
            let arg = &args[1];
            if arg == "--help" || arg == "-h" {
                print_usage();
                return Ok(());
            }
            // Compile and run the file
            run_file(arg)?;
        }
        3 | 4 => {
            // Multiple arguments - check for -o flag
            if args.contains(&"-o".to_string()) {
                compile_file(&args)?;
            } else {
                print_usage();
                return Ok(());
            }
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
    println!("  zen                           Start interactive REPL");
    println!("  zen <file.zen>                Compile and run a Zen file");
    println!("  zen <file.zen> -o <output>    Compile to executable (output in target/)");
    println!("  zen -o <output> <file.zen>    Compile to executable (output in target/)");
    println!("  zen --help                    Show this help message");
    println!();
    println!("Examples:");
    println!("  zen                           # Start REPL");
    println!("  zen hello.zen                 # Run hello.zen file");
    println!("  zen hello.zen -o hello        # Compile to target/hello");
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
                // Parse and execute the input
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
    let source = std::fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    let context = Context::create();
    let compiler = Compiler::new(&context);

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::other(format!("Parse error: {}", e)))?;

    let module = compiler
        .get_module(&program)
        .map_err(|e| io::Error::other(format!("Compilation error: {}", e)))?;

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| io::Error::other(format!("Failed to create execution engine: {}", e)))?;

    // Map __c_lib_mkdir to the actual mkdir symbol from libc
    // This is needed because we use __c_lib_mkdir internally to avoid name collision
    // with the Zen stdlib mkdir function
    if let Some(mkdir_fn) = module.get_function("__c_lib_mkdir") {
        let mkdir_ptr = libc::mkdir as *const ();
        execution_engine.add_global_mapping(&mkdir_fn, mkdir_ptr as usize);
    }

    let exit_code = match execution_engine.get_function_value("main") {
        Ok(main_fn) => {
            let main_type = main_fn.get_type();
            let return_type = main_type.get_return_type();

            if let Some(ret_type) = return_type {
                if ret_type.is_int_type() {
                    let result = unsafe { execution_engine.run_function(main_fn, &[]) };
                    result.as_int(true) as i32
                } else if ret_type.is_struct_type() {
                    eprintln!("Warning: main() returns Result<T,E> which is not fully supported in JIT mode");
                    eprintln!("The function will execute but the Result value cannot be extracted");

                    unsafe {
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            execution_engine.run_function(main_fn, &[])
                        })) {
                            Ok(_) => 0,
                            Err(_) => {
                                eprintln!("Error: Cannot execute main() with Result<T,E> return type in JIT mode");
                                eprintln!("Consider using 'void' or 'i32' as the return type");
                                1
                            }
                        }
                    }
                } else if ret_type.is_float_type() {
                    unsafe { execution_engine.run_function(main_fn, &[]) };
                    0
                } else {
                    let _result = unsafe { execution_engine.run_function(main_fn, &[]) };
                    0
                }
            } else {
                0
            }
        }
        Err(_) => {
            eprintln!("Warning: No main function found");
            0
        }
    };

    // Explicitly drop execution engine before context goes out of scope
    // This prevents double-free issues with LLVM module ownership in release builds
    drop(execution_engine);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

fn compile_file(args: &[String]) -> std::io::Result<()> {
    // Parse arguments
    let (input_file, output_file_raw) = if args[1] == "-o" {
        (&args[3], &args[2])
    } else if args[2] == "-o" {
        (&args[1], &args[3])
    } else {
        print_usage();
        return Ok(());
    };

    // Ensure output goes to target directory if no directory specified
    let output_file = if !output_file_raw.contains('/') {
        format!("target/{}", output_file_raw)
    } else {
        output_file_raw.to_string()
    };

    // Ensure target directory exists
    if let Some(parent) = Path::new(&output_file).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| io::Error::other(format!("Failed to create output directory: {}", e)))?;
    }

    // Read the source file
    let source = std::fs::read_to_string(input_file).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    let context = Context::create();
    let compiler = Compiler::new(&context);

    // Parse the source
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::other(format!("Parse error: {}", e)))?;

    // Get the LLVM module
    let module = compiler
        .get_module(&program)
        .map_err(|e| io::Error::other(format!("Compilation error: {}", e)))?;

    // Debug: Print LLVM IR if DEBUG_LLVM is set
    if std::env::var("DEBUG_LLVM").is_ok() {
        eprintln!("LLVM IR:\n{}", module.print_to_string().to_string());
    }

    // Get target machine
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple)
        .map_err(|e| io::Error::other(format!("Failed to get target: {}", e)))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| io::Error::other("Failed to create target machine"))?;

    // Write object file
    let obj_path = format!("{}.o", output_file);
    target_machine
        .write_to_file(&module, FileType::Object, Path::new(&obj_path))
        .map_err(|e| io::Error::other(format!("Failed to write object file: {}", e)))?;

    // Link with system libraries to create executable
    let mut cmd = Command::new("cc");
    cmd.arg(&obj_path)
        .arg("-o")
        .arg(&output_file)
        .arg("-no-pie") // Disable PIE for compatibility
        .arg("-lm"); // Link math library

    let status = cmd
        .status()
        .map_err(|e| io::Error::other(format!("Failed to link: {}", e)))?;

    if !status.success() {
        return Err(io::Error::other("Linking failed"));
    }

    // Clean up object file
    std::fs::remove_file(&obj_path).ok();

    println!("✅ Successfully compiled to: {}", output_file);

    Ok(())
}

fn execute_zen_code(compiler: &mut Compiler, source: &str) -> Result<Option<String>> {
    // Parse the source
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| CompileError::InternalError(format!("Parse error: {}", e), None))?;

    if program.declarations.is_empty() {
        return Ok(None);
    }

    // Compile the program using LLVM backend
    let llvm_ir = compiler.compile_llvm(&program)?;

    // Return just the LLVM IR
    Ok(Some(llvm_ir))
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
