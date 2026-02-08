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
use zen::module_system::ModuleSystem;
use zen::parser::Parser;
use zen::typechecker::TypeChecker;

fn main() -> std::io::Result<()> {
    // Initialize LLVM
    Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| io::Error::other(format!("LLVM initialization failed: {}", e)))?;

    // CRITICAL: Force MCJIT linkage to prevent LTO dead code elimination.
    // Without this call, LTO removes MCJIT's static constructors that register
    // the JIT backend with LLVM's target registry, causing segfaults.
    ExecutionEngine::link_in_mc_jit();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_repl()?;
    } else if args[1] == "--help" || args[1] == "-h" {
        print_usage();
    } else if args[1] == "analyze" {
        if args.len() < 3 {
            eprintln!("Error: zen analyze requires a file argument");
            eprintln!("Usage: zen analyze <file.zen> [--json]");
            return Ok(());
        }
        let json_output = args.contains(&"--json".to_string());
        analyze_file(&args[2], json_output)?;
    } else if args[1] == "check" {
        if args.len() < 3 {
            eprintln!("Error: zen check requires a file argument");
            eprintln!("Usage: zen check <file.zen> [--json]");
            return Ok(());
        }
        let json_output = args.contains(&"--json".to_string());
        check_file(&args[2], json_output)?;
    } else if args[1] == "query" {
        if args.len() < 4 {
            eprintln!("Usage:");
            eprintln!("  zen query type <file>:<line>:<col>   Type at position");
            eprintln!("  zen query methods <TypeName> <file>  Methods available on a type");
            return Ok(());
        }
        if args[2] == "type" {
            query_type(&args[3])?;
        } else if args[2] == "methods" {
            if args.len() < 5 {
                eprintln!("Usage: zen query methods <TypeName> <file.zen>");
                return Ok(());
            }
            query_methods(&args[3], &args[4])?;
        } else {
            eprintln!("Unknown query subcommand: {}", args[2]);
            eprintln!("Available: type, methods");
        }
    } else if args[1] == "symbols" {
        if args.len() < 3 {
            eprintln!("Error: zen symbols requires a file or directory argument");
            eprintln!("Usage: zen symbols <file.zen> [--json]");
            eprintln!("       zen symbols --recursive <dir> [--json]");
            return Ok(());
        }
        let json_output = args.contains(&"--json".to_string());
        if args[2] == "--recursive" {
            if args.len() < 4 {
                eprintln!("Usage: zen symbols --recursive <dir> [--json]");
                return Ok(());
            }
            // Find the directory arg (skip flags)
            let dir = args
                .iter()
                .skip(3)
                .find(|a| !a.starts_with("--"))
                .map(|s| s.as_str())
                .unwrap_or(".");
            list_symbols_recursive(dir, json_output)?;
        } else {
            list_symbols(&args[2], json_output)?;
        }
    } else if args.contains(&"-o".to_string()) {
        compile_file(&args)?;
    } else if args.len() == 2 {
        run_file(&args[1])?;
    } else {
        print_usage();
    }

    Ok(())
}

fn print_usage() {
    println!("Zen Language Compiler");
    println!();
    println!("Usage:");
    println!("  zen                                    Start interactive REPL");
    println!("  zen <file.zen>                         Compile and run a Zen file");
    println!("  zen <file.zen> -o <output>             Compile to executable (output in target/)");
    println!("  zen analyze <file.zen> [--json]        Full semantic analysis");
    println!("  zen check <file.zen> [--json]          Type-check and report ALL diagnostics");
    println!("  zen query type <file>:<line>:<col>     Type at position (supports member access)");
    println!("  zen query methods <Type> <file.zen>    List methods available on a type");
    println!("  zen symbols <file.zen> [--json]        List declarations");
    println!("  zen symbols --recursive <dir> [--json] List declarations across all .zen files");
    println!("  zen --help                             Show this help message");
    println!();
    println!("Examples:");
    println!("  zen hello.zen                          # Run hello.zen");
    println!("  zen hello.zen -o hello                 # Compile to target/hello");
    println!("  zen analyze app.zen --json             # JSON type info for AI tools");
    println!("  zen check app.zen --json               # ALL errors as structured JSON");
    println!("  zen query type app.zen:15:5            # Type at line 15, col 5");
    println!("  zen query methods Vec app.zen          # Methods on Vec type");
    println!("  zen symbols app.zen --json             # List all declarations");
    println!("  zen symbols --recursive src/ --json    # All declarations in directory");
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
        // Clean up object file even on linking failure to avoid leaking .o files
        std::fs::remove_file(&obj_path).ok();
        return Err(io::Error::other("Linking failed"));
    }

    // Clean up object file
    std::fs::remove_file(&obj_path).ok();

    println!("✅ Successfully compiled to: {}", output_file);

    Ok(())
}

fn analyze_file(file_path: &str, json_output: bool) -> std::io::Result<()> {
    let source = std::fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::other(format!("Parse error: {}", e)))?;

    // Load imports
    let mut module_system = ModuleSystem::new();
    for decl in &program.declarations {
        if let zen::ast::Declaration::ModuleImport { module_path, .. } = decl {
            let _ = module_system.load_module(module_path);
        }
    }
    let merged = module_system.merge_programs(program);

    // Typecheck (tolerant mode — returns partial TypeContext even on error)
    let mut type_checker = TypeChecker::new();
    let loaded_modules = module_system.get_modules();
    type_checker.with_stdlib_modules(&loaded_modules);

    let (type_ctx, check_error) = type_checker.check_program_tolerant(&merged);
    let has_errors = check_error.is_some();

    if json_output {
        // Structured JSON output for AI tools
        let format_ty = zen::lsp::utils::format_type;

        let functions: serde_json::Map<String, serde_json::Value> = type_ctx
            .functions
            .iter()
            .map(|(name, ft)| {
                let params: Vec<serde_json::Value> = ft
                    .params
                    .iter()
                    .map(
                        |(pname, pty)| serde_json::json!({ "name": pname, "type": format_ty(pty) }),
                    )
                    .collect();
                (
                    name.clone(),
                    serde_json::json!({
                        "params": params,
                        "return_type": format_ty(&ft.return_type),
                        "is_external": ft.is_external
                    }),
                )
            })
            .collect();

        let structs: serde_json::Map<String, serde_json::Value> = type_ctx
            .structs
            .iter()
            .map(|(name, fields)| {
                let fields_arr: Vec<serde_json::Value> = fields
                    .iter()
                    .map(
                        |(fname, fty)| serde_json::json!({ "name": fname, "type": format_ty(fty) }),
                    )
                    .collect();
                (name.clone(), serde_json::json!({ "fields": fields_arr }))
            })
            .collect();

        let enums: serde_json::Map<String, serde_json::Value> = type_ctx
            .enums
            .iter()
            .map(|(name, variants)| {
                let variants_arr: Vec<serde_json::Value> = variants
                    .iter()
                    .map(|(vname, payload)| {
                        serde_json::json!({
                            "name": vname,
                            "payload": payload.as_ref().map(format_ty)
                        })
                    })
                    .collect();
                (
                    name.clone(),
                    serde_json::json!({ "variants": variants_arr }),
                )
            })
            .collect();

        let variables: serde_json::Map<String, serde_json::Value> = type_ctx
            .variables
            .iter()
            .map(|(key, ty)| (key.clone(), serde_json::json!(format_ty(ty))))
            .collect();

        let methods: serde_json::Map<String, serde_json::Value> = type_ctx
            .methods
            .iter()
            .map(|(key, ret_ty)| {
                let params = type_ctx.method_params.get(key);
                let params_arr: Vec<serde_json::Value> = params
                    .map(|p| {
                        p.iter()
                            .map(|(pname, pty)| {
                                serde_json::json!({ "name": pname, "type": format_ty(pty) })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (
                    key.clone(),
                    serde_json::json!({
                        "params": params_arr,
                        "return_type": format_ty(ret_ty)
                    }),
                )
            })
            .collect();

        let type_aliases: serde_json::Map<String, serde_json::Value> = type_ctx
            .type_aliases
            .iter()
            .map(|(name, ty)| (name.clone(), serde_json::json!(format_ty(ty))))
            .collect();

        let mut output = serde_json::json!({
            "success": !has_errors,
            "file": file_path,
            "functions": functions,
            "structs": structs,
            "enums": enums,
            "variables": variables,
            "methods": methods,
            "type_aliases": type_aliases,
            "behavior_impls": type_ctx.behavior_impls,
            "constructors": type_ctx.constructors.keys().collect::<Vec<_>>()
        });
        if let Some(ref err) = check_error {
            output["diagnostics"] = serde_json::json!([compile_error_to_diagnostic(err)]);
        }

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        // Human-readable output
        let format_ty = zen::lsp::utils::format_type;

        println!("=== Analysis: {} ===\n", file_path);

        if !type_ctx.functions.is_empty() {
            println!("Functions:");
            for (name, ft) in &type_ctx.functions {
                let params_str = ft
                    .params
                    .iter()
                    .map(|(pname, pty)| format!("{}: {}", pname, format_ty(pty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ext = if ft.is_external {
                    " [external]"
                } else {
                    ""
                };
                println!(
                    "  {} = ({}) {}{}",
                    name,
                    params_str,
                    format_ty(&ft.return_type),
                    ext
                );
            }
            println!();
        }

        if !type_ctx.structs.is_empty() {
            println!("Structs:");
            for (name, fields) in &type_ctx.structs {
                let fields_str = fields
                    .iter()
                    .map(|(fname, fty)| format!("{}: {}", fname, format_ty(fty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  {} {{ {} }}", name, fields_str);
                if let Some(behaviors) = type_ctx.behavior_impls.get(name) {
                    if !behaviors.is_empty() {
                        println!("    implements {}", behaviors.join(", "));
                    }
                }
            }
            println!();
        }

        if !type_ctx.enums.is_empty() {
            println!("Enums:");
            for (name, variants) in &type_ctx.enums {
                let variants_str = variants
                    .iter()
                    .map(|(vname, payload)| {
                        if let Some(ty) = payload {
                            format!(".{}({})", vname, format_ty(ty))
                        } else {
                            format!(".{}", vname)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  {}: {}", name, variants_str);
            }
            println!();
        }

        if !type_ctx.variables.is_empty() {
            println!("Variables:");
            for (key, ty) in &type_ctx.variables {
                println!("  {}: {}", key, format_ty(ty));
            }
            println!();
        }

        if !type_ctx.methods.is_empty() {
            println!("Methods:");
            for (key, ret_ty) in &type_ctx.methods {
                if let Some(params) = type_ctx.method_params.get(key) {
                    let params_str = params
                        .iter()
                        .map(|(pname, pty)| format!("{}: {}", pname, format_ty(pty)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("  {} = ({}) {}", key, params_str, format_ty(ret_ty));
                } else {
                    println!("  {} -> {}", key, format_ty(ret_ty));
                }
            }
            println!();
        }

        if !type_ctx.type_aliases.is_empty() {
            println!("Type Aliases:");
            for (name, ty) in &type_ctx.type_aliases {
                println!("  {} = {}", name, format_ty(ty));
            }
            println!();
        }

        if let Some(ref err) = check_error {
            eprintln!("\nError: {}", err);
        }

        println!(
            "Summary: {} functions, {} structs, {} enums, {} variables, {} methods{}",
            type_ctx.functions.len(),
            type_ctx.structs.len(),
            type_ctx.enums.len(),
            type_ctx.variables.len(),
            type_ctx.methods.len(),
            if has_errors {
                " (partial — file has errors)"
            } else {
                ""
            },
        );
    }

    Ok(())
}

/// `zen check <file> [--json]` - Type-check and report structured diagnostics
fn check_file(file_path: &str, json_output: bool) -> std::io::Result<()> {
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            if json_output {
                let err_obj = serde_json::json!({
                    "success": false,
                    "file": file_path,
                    "diagnostics": [{
                        "severity": "error",
                        "code": "file-not-found",
                        "message": format!("Failed to read file: {}", e),
                        "line": 0, "column": 0
                    }],
                    "summary": { "errors": 1, "warnings": 0 }
                });
                println!("{}", serde_json::to_string_pretty(&err_obj).unwrap());
            } else {
                eprintln!("Error: Failed to read file: {}", e);
            }
            return Ok(());
        }
    };

    // Parse
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            let diag = compile_error_to_diagnostic(&e);
            if json_output {
                let output = serde_json::json!({
                    "success": false,
                    "file": file_path,
                    "diagnostics": [diag],
                    "summary": { "errors": 1, "warnings": 0 }
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                eprintln!("{}", e);
            }
            return Ok(());
        }
    };

    // Load imports
    let mut module_system = ModuleSystem::new();
    for decl in &program.declarations {
        if let zen::ast::Declaration::ModuleImport { module_path, .. } = decl {
            let _ = module_system.load_module(module_path);
        }
    }
    let merged = module_system.merge_programs(program);

    // Typecheck (collect ALL errors, not just the first)
    let mut type_checker = TypeChecker::new();
    let loaded_modules = module_system.get_modules();
    type_checker.with_stdlib_modules(&loaded_modules);

    let (_type_ctx, errors) = type_checker.check_program_collect_errors(&merged);

    let diagnostics: Vec<serde_json::Value> =
        errors.iter().map(compile_error_to_diagnostic).collect();
    let error_count = diagnostics.len();

    if json_output {
        let output = serde_json::json!({
            "success": error_count == 0,
            "file": file_path,
            "diagnostics": diagnostics,
            "summary": { "errors": error_count, "warnings": 0 }
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if error_count == 0 {
        println!("{}: OK", file_path);
    } else {
        for err in &errors {
            eprintln!("{}", err);
        }
        eprintln!("\n{} error(s) found", error_count);
    }

    Ok(())
}

/// Convert a CompileError into a JSON diagnostic object
fn compile_error_to_diagnostic(err: &CompileError) -> serde_json::Value {
    let code = match err {
        CompileError::SyntaxError(..) => "syntax-error",
        CompileError::UndeclaredVariable(..) => "undeclared-variable",
        CompileError::UndeclaredFunction(..) => "undeclared-function",
        CompileError::TypeMismatch { .. } => "type-mismatch",
        CompileError::InvalidLoopCondition(..) => "invalid-loop-condition",
        CompileError::MissingReturnStatement(..) => "missing-return",
        CompileError::InternalError(..) => "internal-error",
        CompileError::UnsupportedFeature(..) => "unsupported-feature",
        CompileError::TypeError(..) => "type-error",
        CompileError::FileNotFound(..) => "file-not-found",
        CompileError::ParseError(..) => "parse-error",
        CompileError::ComptimeError(..) => "comptime-error",
        CompileError::UnexpectedToken { .. } => "unexpected-token",
        CompileError::InvalidPattern(..) => "invalid-pattern",
        CompileError::ImportError(..) => "import-error",
        CompileError::FFIError(..) => "ffi-error",
        CompileError::InvalidSyntax { .. } => "invalid-syntax",
        CompileError::MissingTypeAnnotation(..) => "missing-type-annotation",
        CompileError::DuplicateDeclaration { .. } => "duplicate-declaration",
        CompileError::BuildError(..) => "build-error",
        CompileError::FileError(..) => "file-error",
        CompileError::CyclicDependency(..) => "cyclic-dependency",
    };

    let (line, column) = err.span().map(|s| (s.line, s.column + 1)).unwrap_or((0, 0));

    let (end_line, end_column) = err
        .span()
        .map(|s| {
            if s.end > s.start {
                // Approximate end position on same line
                (s.line, s.column + 1 + (s.end - s.start))
            } else {
                (s.line, s.column + 2)
            }
        })
        .unwrap_or((line, column));

    serde_json::json!({
        "severity": "error",
        "code": code,
        "message": err.message(),
        "line": line,
        "column": column,
        "end_line": end_line,
        "end_column": end_column
    })
}

/// `zen query type <file>:<line>:<col>` - Point query for type at position
fn query_type(location: &str) -> std::io::Result<()> {
    // Parse "file:line:col"
    let parts: Vec<&str> = location.rsplitn(3, ':').collect();
    if parts.len() < 3 {
        eprintln!("Error: Invalid location format. Expected <file>:<line>:<col>");
        return Ok(());
    }
    let col: usize = parts[0]
        .parse()
        .map_err(|_| io::Error::other("Invalid column number"))?;
    let line: usize = parts[1]
        .parse()
        .map_err(|_| io::Error::other("Invalid line number"))?;
    let file_path = parts[2];

    let source = std::fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::other(format!("Parse error: {}", e)))?;

    // Load imports
    let mut module_system = ModuleSystem::new();
    for decl in &program.declarations {
        if let zen::ast::Declaration::ModuleImport { module_path, .. } = decl {
            let _ = module_system.load_module(module_path);
        }
    }
    let merged = module_system.merge_programs(program);

    // Typecheck (tolerant mode — returns partial TypeContext even on error)
    let mut type_checker = TypeChecker::new();
    let loaded_modules = module_system.get_modules();
    type_checker.with_stdlib_modules(&loaded_modules);

    let (type_ctx, check_error) = type_checker.check_program_tolerant(&merged);

    let format_ty = zen::lsp::utils::format_type;
    let source_lines: Vec<&str> = source.lines().collect();

    // Find the symbol at the given position
    if line == 0 || line > source_lines.len() {
        let output = serde_json::json!({
            "error": format!("Line {} out of range (file has {} lines)", line, source_lines.len())
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    let target_line = source_lines[line - 1];
    let (symbol, receiver) = extract_symbol_with_receiver(target_line, col);

    if symbol.is_empty() {
        let output = serde_json::json!({
            "error": format!("No symbol at {}:{}:{}", file_path, line, col)
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // Determine which function scope the position is in
    let scope = find_scope_at_line(&source_lines, line);

    // If there's a receiver (member access like obj.field), resolve the receiver type first
    if let Some(ref recv_name) = receiver {
        // Resolve the receiver's type
        let recv_type = resolve_symbol_type(&type_ctx, recv_name, &scope);
        if let Some(recv_type_name) = recv_type {
            // Check if symbol is a struct field
            if let Some(field_type) = type_ctx.get_struct_field_type(&recv_type_name, &symbol) {
                let output = serde_json::json!({
                    "symbol": format!("{}.{}", recv_name, symbol),
                    "type": format_ty(&field_type),
                    "kind": "field",
                    "receiver_type": recv_type_name,
                    "scope": scope
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
                return Ok(());
            }
            // Check if symbol is a method on the receiver type
            if let Some(ret_type) = type_ctx.get_method_return_type(&recv_type_name, &symbol) {
                let params = type_ctx.get_method_params(&recv_type_name, &symbol);
                let params_str = params
                    .map(|p| {
                        p.iter()
                            .map(|(pname, pty)| format!("{}: {}", pname, format_ty(pty)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                let output = serde_json::json!({
                    "symbol": format!("{}.{}", recv_name, symbol),
                    "type": format!("({}) {}", params_str, format_ty(&ret_type)),
                    "kind": "method",
                    "receiver_type": recv_type_name,
                    "scope": scope
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
                return Ok(());
            }
            // Check for UFC: a free function whose first param matches the receiver type
            for (fname, ft) in &type_ctx.functions {
                if fname == &symbol && !ft.params.is_empty() {
                    let first_param_type = format_ty(&ft.params[0].1);
                    if first_param_type == recv_type_name {
                        let params_str = ft
                            .params
                            .iter()
                            .map(|(pname, pty)| format!("{}: {}", pname, format_ty(pty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let output = serde_json::json!({
                            "symbol": format!("{}.{}", recv_name, symbol),
                            "type": format!("({}) {}", params_str, format_ty(&ft.return_type)),
                            "kind": "function (UFC)",
                            "receiver_type": recv_type_name,
                            "scope": scope
                        });
                        println!("{}", serde_json::to_string_pretty(&output).unwrap());
                        return Ok(());
                    }
                }
            }
        }
    }

    // No receiver — resolve as a standalone symbol

    // 1. Check variables in the enclosing function scope
    if let Some(ref func_name) = scope {
        if let Some(var_type) = type_ctx.get_variable_type(func_name, &symbol) {
            let output = serde_json::json!({
                "symbol": symbol,
                "type": format_ty(&var_type),
                "kind": "variable",
                "scope": func_name
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            return Ok(());
        }
        // Check if it's a function parameter
        if let Some(func) = type_ctx.functions.get(func_name.as_str()) {
            for (pname, pty) in &func.params {
                if pname == &symbol {
                    let output = serde_json::json!({
                        "symbol": symbol,
                        "type": format_ty(pty),
                        "kind": "parameter",
                        "scope": func_name
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                    return Ok(());
                }
            }
        }
    }

    // 2. Check functions
    if let Some(func) = type_ctx.functions.get(&symbol) {
        let params_str = func
            .params
            .iter()
            .map(|(pname, pty)| format!("{}: {}", pname, format_ty(pty)))
            .collect::<Vec<_>>()
            .join(", ");
        let output = serde_json::json!({
            "symbol": symbol,
            "type": format!("({}) {}", params_str, format_ty(&func.return_type)),
            "kind": "function",
            "scope": serde_json::Value::Null
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // 3. Check structs
    if type_ctx.structs.contains_key(&symbol) {
        let output = serde_json::json!({
            "symbol": symbol,
            "type": "struct",
            "kind": "struct",
            "scope": serde_json::Value::Null
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // 4. Check enums
    if type_ctx.enums.contains_key(&symbol) {
        let output = serde_json::json!({
            "symbol": symbol,
            "type": "enum",
            "kind": "enum",
            "scope": serde_json::Value::Null
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // 5. Check type aliases
    if let Some(alias_type) = type_ctx.type_aliases.get(&symbol) {
        let output = serde_json::json!({
            "symbol": symbol,
            "type": format_ty(alias_type),
            "kind": "type_alias",
            "scope": serde_json::Value::Null
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // 6. Check module imports
    if let Some(module_path) = type_ctx.module_imports.get(&symbol) {
        let output = serde_json::json!({
            "symbol": symbol,
            "type": format!("module \"{}\"", module_path),
            "kind": "module",
            "scope": serde_json::Value::Null
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }

    // Not found — include any typecheck error for context
    let mut result = serde_json::json!({
        "symbol": symbol,
        "type": serde_json::Value::Null,
        "kind": "unknown",
        "scope": scope
    });
    if let Some(err) = check_error {
        result["note"] = serde_json::json!(format!("file has type errors: {}", err));
    }
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}

/// Extract symbol and optional receiver for member access.
/// For `obj.field` at the column of `field`, returns ("field", Some("obj")).
/// For `x` returns ("x", None).
fn extract_symbol_with_receiver(line: &str, col: usize) -> (String, Option<String>) {
    let chars: Vec<char> = line.chars().collect();
    if col == 0 || col > chars.len() {
        return (String::new(), None);
    }
    let idx = col - 1; // Convert to 0-based

    // Check if position is on an identifier character
    if !chars[idx].is_alphanumeric() && chars[idx] != '_' {
        return (String::new(), None);
    }

    // Scan left to find start of identifier
    let mut start = idx;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    // Scan right to find end of identifier
    let mut end = idx;
    while end + 1 < chars.len() && (chars[end + 1].is_alphanumeric() || chars[end + 1] == '_') {
        end += 1;
    }

    let symbol: String = chars[start..=end].iter().collect();

    // Check for receiver: is there a dot before the identifier?
    let receiver = if start > 0 && chars[start - 1] == '.' {
        // Scan left of the dot to find the receiver identifier
        let dot_pos = start - 1;
        if dot_pos > 0
            && (chars[dot_pos - 1].is_alphanumeric()
                || chars[dot_pos - 1] == '_'
                || chars[dot_pos - 1] == ')')
        {
            if chars[dot_pos - 1] == ')' {
                // Method chain like `foo().bar` — extract the function name before ()
                // Walk backwards past the parens
                let mut paren_depth = 1;
                let mut p = dot_pos - 2;
                while p > 0 && paren_depth > 0 {
                    if chars[p] == ')' {
                        paren_depth += 1;
                    }
                    if chars[p] == '(' {
                        paren_depth -= 1;
                    }
                    if paren_depth > 0 {
                        p -= 1;
                    }
                }
                // p is at '(', scan left for the function name
                if p > 0 {
                    let mut rend = p - 1;
                    while rend > 0 && chars[rend].is_whitespace() {
                        rend -= 1;
                    }
                    let mut rstart = rend;
                    while rstart > 0
                        && (chars[rstart - 1].is_alphanumeric() || chars[rstart - 1] == '_')
                    {
                        rstart -= 1;
                    }
                    if rstart <= rend && (chars[rstart].is_alphanumeric() || chars[rstart] == '_') {
                        Some(chars[rstart..=rend].iter().collect())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                // Simple receiver like `obj.field`
                let rend = dot_pos - 1;
                let mut rstart = rend;
                while rstart > 0
                    && (chars[rstart - 1].is_alphanumeric() || chars[rstart - 1] == '_')
                {
                    rstart -= 1;
                }
                Some(chars[rstart..=rend].iter().collect())
            }
        } else {
            None
        }
    } else {
        None
    };

    (symbol, receiver)
}

/// Find the enclosing function scope at a given line number (1-based)
fn find_scope_at_line(lines: &[&str], target_line: usize) -> Option<String> {
    // Walk backwards from target_line to find the nearest function definition
    // Function definitions in Zen look like: name = (...) ReturnType {
    let func_pattern = regex::Regex::new(r"^(\w+)\s*=\s*\(").ok()?;
    let mut brace_depth: i32 = 0;

    for i in (0..target_line).rev() {
        let line = lines[i];
        let trimmed = line.trim();

        // Count braces on this line to track scope
        for ch in trimmed.chars() {
            if ch == '}' {
                brace_depth += 1;
            } else if ch == '{' {
                brace_depth -= 1;
            }
        }

        // If we've closed all scopes back to a function definition
        if brace_depth <= 0 {
            if let Some(caps) = func_pattern.captures(trimmed) {
                return Some(caps[1].to_string());
            }
        }
    }
    None
}

/// `zen symbols <file> [--json]` - List declarations without full type inference
fn list_symbols(file_path: &str, json_output: bool) -> std::io::Result<()> {
    let source = std::fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse only (no typechecking for speed)
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            if json_output {
                let output = serde_json::json!({
                    "file": file_path,
                    "error": format!("{}", e),
                    "symbols": []
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                eprintln!("Parse error: {}", e);
            }
            return Ok(());
        }
    };

    let source_lines: Vec<&str> = source.lines().collect();
    let format_ty = zen::lsp::utils::format_type;
    let mut symbols: Vec<serde_json::Value> = Vec::new();

    for decl in &program.declarations {
        match decl {
            zen::ast::Declaration::Function(f) => {
                let line = find_decl_line(&source_lines, &f.name);
                let params_str = f
                    .args
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, format_ty(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let sig = format!("({}) {}", params_str, format_ty(&f.return_type));
                symbols.push(serde_json::json!({
                    "name": f.name,
                    "kind": "function",
                    "line": line,
                    "signature": sig
                }));
            }
            zen::ast::Declaration::Struct(s) => {
                let line = s
                    .span
                    .as_ref()
                    .map(|sp| sp.line)
                    .unwrap_or_else(|| find_decl_line(&source_lines, &s.name));
                let fields: Vec<String> = s
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, format_ty(&f.type_)))
                    .collect();
                symbols.push(serde_json::json!({
                    "name": s.name,
                    "kind": "struct",
                    "line": line,
                    "fields": fields
                }));
                // List methods defined on the struct
                for m in &s.methods {
                    let mline = find_decl_line(&source_lines, &m.name);
                    let params_str = m
                        .args
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, format_ty(t)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let sig = format!("({}) {}", params_str, format_ty(&m.return_type));
                    symbols.push(serde_json::json!({
                        "name": format!("{}.{}", s.name, m.name),
                        "kind": "method",
                        "line": mline,
                        "signature": sig
                    }));
                }
            }
            zen::ast::Declaration::Enum(e) => {
                let line = e
                    .span
                    .as_ref()
                    .map(|sp| sp.line)
                    .unwrap_or_else(|| find_decl_line(&source_lines, &e.name));
                let variants: Vec<String> = e
                    .variants
                    .iter()
                    .map(|v| {
                        if let Some(ref ty) = v.payload {
                            format!(".{}({})", v.name, format_ty(ty))
                        } else {
                            format!(".{}", v.name)
                        }
                    })
                    .collect();
                symbols.push(serde_json::json!({
                    "name": e.name,
                    "kind": "enum",
                    "line": line,
                    "variants": variants
                }));
            }
            zen::ast::Declaration::TypeAlias(ta) => {
                let line = ta
                    .span
                    .as_ref()
                    .map(|sp| sp.line)
                    .unwrap_or_else(|| find_decl_line(&source_lines, &ta.name));
                symbols.push(serde_json::json!({
                    "name": ta.name,
                    "kind": "type_alias",
                    "line": line,
                    "target": format_ty(&ta.target_type)
                }));
            }
            zen::ast::Declaration::Behavior(b) => {
                let line = find_decl_line(&source_lines, &b.name);
                let methods: Vec<String> = b.methods.iter().map(|m| m.name.clone()).collect();
                symbols.push(serde_json::json!({
                    "name": b.name,
                    "kind": "behavior",
                    "line": line,
                    "methods": methods
                }));
            }
            zen::ast::Declaration::Trait(t) => {
                let line = t
                    .span
                    .as_ref()
                    .map(|sp| sp.line)
                    .unwrap_or_else(|| find_decl_line(&source_lines, &t.name));
                let methods: Vec<String> = t.methods.iter().map(|m| m.name.clone()).collect();
                symbols.push(serde_json::json!({
                    "name": t.name,
                    "kind": "trait",
                    "line": line,
                    "methods": methods
                }));
            }
            zen::ast::Declaration::ImplBlock(ib) => {
                let _line = find_decl_line(&source_lines, &format!("impl {}", ib.type_name));
                for m in &ib.methods {
                    let mline = find_decl_line(&source_lines, &m.name);
                    let params_str = m
                        .args
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, format_ty(t)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let sig = format!("({}) {}", params_str, format_ty(&m.return_type));
                    symbols.push(serde_json::json!({
                        "name": format!("{}.{}", ib.type_name, m.name),
                        "kind": "method",
                        "line": mline,
                        "signature": sig
                    }));
                }
            }
            zen::ast::Declaration::Constant { name, span, .. } => {
                let line = span
                    .as_ref()
                    .map(|sp| sp.line)
                    .unwrap_or_else(|| find_decl_line(&source_lines, name));
                symbols.push(serde_json::json!({
                    "name": name,
                    "kind": "constant",
                    "line": line
                }));
            }
            zen::ast::Declaration::ExternalFunction(ef) => {
                symbols.push(serde_json::json!({
                    "name": ef.name,
                    "kind": "external_function",
                    "line": 0,
                    "signature": format!("(...) {}", format_ty(&ef.return_type))
                }));
            }
            _ => {} // Skip imports, exports, comptime blocks, trait impls, etc.
        }
    }

    if json_output {
        let output = serde_json::json!({
            "file": file_path,
            "symbols": symbols
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("=== Symbols: {} ===\n", file_path);
        for sym in &symbols {
            let kind = sym["kind"].as_str().unwrap_or("?");
            let name = sym["name"].as_str().unwrap_or("?");
            let line = sym["line"].as_u64().unwrap_or(0);
            match kind {
                "function" => {
                    let sig = sym["signature"].as_str().unwrap_or("");
                    println!("  fn {} {} [line {}]", name, sig, line);
                }
                "struct" => {
                    let fields = sym["fields"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!("  struct {} {{ {} }} [line {}]", name, fields, line);
                }
                "enum" => {
                    let variants = sym["variants"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!("  enum {} {{ {} }} [line {}]", name, variants, line);
                }
                "method" => {
                    let sig = sym["signature"].as_str().unwrap_or("");
                    println!("  method {} {} [line {}]", name, sig, line);
                }
                "type_alias" => {
                    let target = sym["target"].as_str().unwrap_or("?");
                    println!("  type {} = {} [line {}]", name, target, line);
                }
                "behavior" => {
                    let methods = sym["methods"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!("  behavior {} {{ {} }} [line {}]", name, methods, line);
                }
                "trait" => {
                    let methods = sym["methods"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!("  trait {} {{ {} }} [line {}]", name, methods, line);
                }
                "constant" => {
                    println!("  const {} [line {}]", name, line);
                }
                "external_function" => {
                    let sig = sym["signature"].as_str().unwrap_or("");
                    println!("  extern {} {} [line {}]", name, sig, line);
                }
                _ => {
                    println!("  {} {} [line {}]", kind, name, line);
                }
            }
        }
        println!("\n{} symbols found", symbols.len());
    }

    Ok(())
}

/// Find the line number (1-based) of a declaration by name
fn find_decl_line(lines: &[&str], name: &str) -> usize {
    for (i, line) in lines.iter().enumerate() {
        if line.contains(name) {
            return i + 1;
        }
    }
    0
}

/// Resolve a symbol name to its type name string using the TypeContext.
/// Used for member access resolution — given "obj", returns "Point" if obj: Point.
fn resolve_symbol_type(
    type_ctx: &zen::type_context::TypeContext,
    name: &str,
    scope: &Option<String>,
) -> Option<String> {
    let format_ty = zen::lsp::utils::format_type;

    // Check variable in scope
    if let Some(ref func_name) = scope {
        if let Some(var_type) = type_ctx.get_variable_type(func_name, name) {
            return Some(format_ty(&var_type));
        }
        // Check function parameter
        if let Some(func) = type_ctx.functions.get(func_name.as_str()) {
            for (pname, pty) in &func.params {
                if pname == name {
                    return Some(format_ty(pty));
                }
            }
        }
    }
    // Check if it's a struct name (for static method calls like Point.new())
    if type_ctx.structs.contains_key(name) {
        return Some(name.to_string());
    }
    // Check if it's an enum name
    if type_ctx.enums.contains_key(name) {
        return Some(name.to_string());
    }
    None
}

/// `zen query methods <TypeName> <file>` - List all methods available on a type
fn query_methods(type_name: &str, file_path: &str) -> std::io::Result<()> {
    let source = std::fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse
    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::other(format!("Parse error: {}", e)))?;

    // Load imports
    let mut module_system = ModuleSystem::new();
    for decl in &program.declarations {
        if let zen::ast::Declaration::ModuleImport { module_path, .. } = decl {
            let _ = module_system.load_module(module_path);
        }
    }
    let merged = module_system.merge_programs(program);

    // Typecheck (tolerant)
    let mut type_checker = TypeChecker::new();
    let loaded_modules = module_system.get_modules();
    type_checker.with_stdlib_modules(&loaded_modules);
    let (type_ctx, _) = type_checker.check_program_tolerant(&merged);

    let format_ty = zen::lsp::utils::format_type;

    let mut methods: Vec<serde_json::Value> = Vec::new();

    // 1. Collect methods from TypeContext methods map (from struct methods + impl blocks)
    for (key, ret_ty) in &type_ctx.methods {
        if let Some(rest) = key.strip_prefix(&format!("{}.", type_name)) {
            let params = type_ctx.method_params.get(key);
            let params_arr: Vec<serde_json::Value> = params
                .map(|p| {
                    p.iter()
                        .map(|(pname, pty)| serde_json::json!({"name": pname, "type": format_ty(pty)}))
                        .collect()
                })
                .unwrap_or_default();
            methods.push(serde_json::json!({
                "name": rest,
                "kind": "method",
                "params": params_arr,
                "return_type": format_ty(ret_ty)
            }));
        }
    }

    // 2. Collect constructors
    for (key, ret_ty) in &type_ctx.constructors {
        if let Some(rest) = key.strip_prefix(&format!("{}.", type_name)) {
            methods.push(serde_json::json!({
                "name": rest,
                "kind": "constructor",
                "params": [],
                "return_type": format_ty(ret_ty)
            }));
        }
    }

    // 3. Collect UFC-compatible free functions (first param type matches)
    for (fname, ft) in &type_ctx.functions {
        if !ft.params.is_empty() {
            let first_param_type = format_ty(&ft.params[0].1);
            if first_param_type == type_name {
                let params_arr: Vec<serde_json::Value> = ft
                    .params
                    .iter()
                    .map(|(pname, pty)| serde_json::json!({"name": pname, "type": format_ty(pty)}))
                    .collect();
                methods.push(serde_json::json!({
                    "name": fname,
                    "kind": "function (UFC)",
                    "params": params_arr,
                    "return_type": format_ty(&ft.return_type)
                }));
            }
        }
    }

    // 4. Struct fields (useful context for "what can I do with this type")
    let fields: Vec<serde_json::Value> = type_ctx
        .get_struct_fields(type_name)
        .map(|f| {
            f.iter()
                .map(|(fname, fty)| serde_json::json!({"name": fname, "type": format_ty(fty)}))
                .collect()
        })
        .unwrap_or_default();

    // 5. Behaviors implemented
    let behaviors: Vec<&String> = type_ctx
        .behavior_impls
        .get(type_name)
        .map(|b| b.iter().collect())
        .unwrap_or_default();

    let output = serde_json::json!({
        "type": type_name,
        "fields": fields,
        "methods": methods,
        "behaviors": behaviors
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}

/// `zen symbols --recursive <dir>` - List symbols across all .zen files in a directory
fn list_symbols_recursive(dir_path: &str, json_output: bool) -> std::io::Result<()> {
    let format_ty = zen::lsp::utils::format_type;
    let mut all_files: Vec<serde_json::Value> = Vec::new();
    let mut total_symbols = 0;

    // Collect all .zen files recursively
    let mut zen_files: Vec<std::path::PathBuf> = Vec::new();
    collect_zen_files(Path::new(dir_path), &mut zen_files)?;
    zen_files.sort();

    for file_path in &zen_files {
        let file_str = file_path.to_string_lossy().to_string();
        let source = match std::fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let lexer = Lexer::new(&source);
        let mut parser = Parser::new(lexer);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(_) => continue, // Skip files with parse errors
        };

        let source_lines: Vec<&str> = source.lines().collect();
        let mut symbols: Vec<serde_json::Value> = Vec::new();

        for decl in &program.declarations {
            match decl {
                zen::ast::Declaration::Function(f) => {
                    let line = find_decl_line(&source_lines, &f.name);
                    let params_str = f
                        .args
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, format_ty(t)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    symbols.push(serde_json::json!({
                        "name": f.name,
                        "kind": "function",
                        "line": line,
                        "signature": format!("({}) {}", params_str, format_ty(&f.return_type))
                    }));
                }
                zen::ast::Declaration::Struct(s) => {
                    let line = s
                        .span
                        .as_ref()
                        .map(|sp| sp.line)
                        .unwrap_or_else(|| find_decl_line(&source_lines, &s.name));
                    let fields: Vec<String> = s
                        .fields
                        .iter()
                        .map(|f| format!("{}: {}", f.name, format_ty(&f.type_)))
                        .collect();
                    symbols.push(serde_json::json!({
                        "name": s.name, "kind": "struct", "line": line, "fields": fields
                    }));
                    for m in &s.methods {
                        symbols.push(serde_json::json!({
                            "name": format!("{}.{}", s.name, m.name),
                            "kind": "method",
                            "line": find_decl_line(&source_lines, &m.name)
                        }));
                    }
                }
                zen::ast::Declaration::Enum(e) => {
                    let line = e
                        .span
                        .as_ref()
                        .map(|sp| sp.line)
                        .unwrap_or_else(|| find_decl_line(&source_lines, &e.name));
                    let variants: Vec<String> = e
                        .variants
                        .iter()
                        .map(|v| {
                            if let Some(ref ty) = v.payload {
                                format!(".{}({})", v.name, format_ty(ty))
                            } else {
                                format!(".{}", v.name)
                            }
                        })
                        .collect();
                    symbols.push(serde_json::json!({
                        "name": e.name, "kind": "enum", "line": line, "variants": variants
                    }));
                }
                zen::ast::Declaration::TypeAlias(ta) => {
                    let line = ta
                        .span
                        .as_ref()
                        .map(|sp| sp.line)
                        .unwrap_or_else(|| find_decl_line(&source_lines, &ta.name));
                    symbols.push(serde_json::json!({
                        "name": ta.name, "kind": "type_alias", "line": line,
                        "target": format_ty(&ta.target_type)
                    }));
                }
                zen::ast::Declaration::Behavior(b) => {
                    symbols.push(serde_json::json!({
                        "name": b.name, "kind": "behavior",
                        "line": find_decl_line(&source_lines, &b.name),
                        "methods": b.methods.iter().map(|m| &m.name).collect::<Vec<_>>()
                    }));
                }
                zen::ast::Declaration::Trait(t) => {
                    let line = t
                        .span
                        .as_ref()
                        .map(|sp| sp.line)
                        .unwrap_or_else(|| find_decl_line(&source_lines, &t.name));
                    symbols.push(serde_json::json!({
                        "name": t.name, "kind": "trait", "line": line,
                        "methods": t.methods.iter().map(|m| &m.name).collect::<Vec<_>>()
                    }));
                }
                zen::ast::Declaration::ImplBlock(ib) => {
                    for m in &ib.methods {
                        symbols.push(serde_json::json!({
                            "name": format!("{}.{}", ib.type_name, m.name),
                            "kind": "method",
                            "line": find_decl_line(&source_lines, &m.name)
                        }));
                    }
                }
                zen::ast::Declaration::Constant { name, span, .. } => {
                    let line = span
                        .as_ref()
                        .map(|sp| sp.line)
                        .unwrap_or_else(|| find_decl_line(&source_lines, name));
                    symbols.push(
                        serde_json::json!({ "name": name, "kind": "constant", "line": line }),
                    );
                }
                zen::ast::Declaration::ExternalFunction(ef) => {
                    symbols.push(serde_json::json!({
                        "name": ef.name, "kind": "external_function", "line": 0,
                        "signature": format!("(...) {}", format_ty(&ef.return_type))
                    }));
                }
                _ => {}
            }
        }

        total_symbols += symbols.len();
        if !symbols.is_empty() {
            all_files.push(serde_json::json!({
                "file": file_str,
                "symbols": symbols
            }));
        }
    }

    if json_output {
        let output = serde_json::json!({
            "directory": dir_path,
            "files": all_files,
            "total_files": zen_files.len(),
            "total_symbols": total_symbols
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        for file_entry in &all_files {
            let file = file_entry["file"].as_str().unwrap_or("?");
            let syms = file_entry["symbols"].as_array();
            println!("=== {} ===", file);
            if let Some(syms) = syms {
                for sym in syms {
                    let kind = sym["kind"].as_str().unwrap_or("?");
                    let name = sym["name"].as_str().unwrap_or("?");
                    let line = sym["line"].as_u64().unwrap_or(0);
                    println!("  {} {} [line {}]", kind, name, line);
                }
            }
            println!();
        }
        println!("{} symbols across {} files", total_symbols, zen_files.len());
    }

    Ok(())
}

/// Recursively collect all .zen files in a directory
fn collect_zen_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", dir.display()),
        ));
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_zen_files(&path, files)?;
        } else if path.extension().map(|e| e == "zen").unwrap_or(false) {
            files.push(path);
        }
    }
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

    // If DEBUG_LLVM is set, print the IR instead of executing
    if std::env::var("DEBUG_LLVM").is_ok() {
        let llvm_ir = compiler.compile_llvm(&program)?;
        return Ok(Some(llvm_ir));
    }

    // Compile and execute via JIT
    let module = compiler.get_module(&program)?;

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| {
            CompileError::InternalError(format!("Failed to create execution engine: {}", e), None)
        })?;

    // Map __c_lib_mkdir if needed
    if let Some(mkdir_fn) = module.get_function("__c_lib_mkdir") {
        let mkdir_ptr = libc::mkdir as *const ();
        execution_engine.add_global_mapping(&mkdir_fn, mkdir_ptr as usize);
    }

    // Look for main function and execute it
    match execution_engine.get_function_value("main") {
        Ok(main_fn) => {
            let main_type = main_fn.get_type();
            let return_type = main_type.get_return_type();

            if let Some(ret_type) = return_type {
                if ret_type.is_int_type() {
                    let result = unsafe { execution_engine.run_function(main_fn, &[]) };
                    let exit_code = result.as_int(true) as i32;
                    drop(execution_engine);
                    if exit_code != 0 {
                        return Ok(Some(format!("Exit code: {}", exit_code)));
                    }
                    return Ok(None);
                }
            }

            // void or other return type - just run it
            unsafe { execution_engine.run_function(main_fn, &[]) };
            drop(execution_engine);
            Ok(None)
        }
        Err(_) => {
            drop(execution_engine);
            Ok(Some("(no main function found)".to_string()))
        }
    }
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
