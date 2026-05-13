use std::path::Path;
use std::process;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::resolver::Resolver;
use zen::typechecker::TypeChecker;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("zen compiler v0.8.0");
        eprintln!("Usage: zen <command> [args]");
        eprintln!("Commands:");
        eprintln!("  check <file>   Parse and typecheck a .zen file");
        eprintln!("  build <file>   Compile a .zen file to a binary");
        eprintln!("  emit  <file>   Emit C source (no compilation)");
        eprintln!("  <file>         Run a .zen file");
        process::exit(1);
    }

    match args[1].as_str() {
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: zen check <file.zen>");
                process::exit(1);
            }
            cmd_check(&args[2]);
        }
        "build" => {
            if args.len() < 3 {
                eprintln!("Usage: zen build <file.zen>");
                process::exit(1);
            }
            cmd_build(&args[2]);
        }
        "emit" => {
            if args.len() < 3 {
                eprintln!("Usage: zen emit <file.zen>");
                process::exit(1);
            }
            cmd_emit(&args[2]);
        }
        arg if arg.ends_with(".zen") => {
            cmd_build(arg);
        }
        other => {
            eprintln!("unknown command: {}", other);
            process::exit(1);
        }
    }
}

/// Parse and typecheck a .zen file, returning the typed program on success.
fn frontend(path_str: &str) -> zen::ast::typed::TypedProgram {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("error: file not found: {}", path_str);
        process::exit(1);
    }

    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();

    let program = match module_system.load_with_imports(path, &mut files) {
        Ok(p) => p,
        Err(errs) => {
            print_errors(&errs, &files);
            process::exit(1);
        }
    };

    let resolver_symbols = match Resolver::new().resolve_program(&program) {
        Ok(symbols) => symbols,
        Err(diags) => {
            for diag in &diags {
                print_diagnostic(diag, &files);
            }
            let errors = diags
                .iter()
                .filter(|d| d.severity == zen::error::Severity::Error)
                .count();
            eprintln!("  {} error(s)", errors);
            process::exit(1);
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check_program_with_symbols(&program, &resolver_symbols) {
        Ok(typed) => {
            let diags = checker.diagnostics();
            for diag in diags {
                print_diagnostic(diag, &files);
            }
            typed
        }
        Err(diags) => {
            for diag in &diags {
                print_diagnostic(diag, &files);
            }
            let errors = diags
                .iter()
                .filter(|d| d.severity == zen::error::Severity::Error)
                .count();
            eprintln!("  {} error(s)", errors);
            process::exit(1);
        }
    }
}

fn cmd_check(path_str: &str) {
    let typed = check_frontend(path_str);
    println!(
        "  {} functions, {} types — ok",
        typed.functions.len(),
        typed.types.len()
    );
}

fn check_frontend(path_str: &str) -> zen::ast::typed::TypedProgram {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("error: file not found: {}", path_str);
        process::exit(1);
    }

    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();

    let graph = match module_system.load_module_graph(path, &mut files) {
        Ok(graph) => graph,
        Err(errs) => {
            print_errors(&errs, &files);
            process::exit(1);
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check_module_graph_entry(&graph) {
        Ok(typed) => {
            for diag in checker.diagnostics() {
                print_diagnostic(diag, &files);
            }
            typed
        }
        Err(diags) => {
            for diag in &diags {
                print_diagnostic(diag, &files);
            }
            for diag in checker.diagnostics() {
                print_diagnostic(diag, &files);
            }
            let errors = diags
                .iter()
                .filter(|d| d.severity == zen::error::Severity::Error)
                .count();
            eprintln!("  {} error(s)", errors);
            process::exit(1);
        }
    }
}

fn cmd_emit(path_str: &str) {
    let typed = frontend(path_str);
    let backend = CBackend;
    match backend.generate(&typed) {
        Ok(c_source) => {
            print!("{}", c_source);
        }
        Err(e) => {
            eprintln!("codegen error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_build(path_str: &str) {
    if is_build_zen_path(path_str) {
        eprintln!(
            "error: build.zen execution is gated until deterministic build graph support exists"
        );
        process::exit(1);
    }

    let typed = frontend(path_str);
    let backend = CBackend;
    let c_source = match backend.generate(&typed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("codegen error: {}", e);
            process::exit(1);
        }
    };

    // Determine output paths
    let stem = Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    let c_path = format!("{}.c", stem);
    let bin_path = stem.to_string();

    // Write C source
    if let Err(e) = std::fs::write(&c_path, &c_source) {
        eprintln!("error writing {}: {}", c_path, e);
        process::exit(1);
    }
    println!("  emitted {}", c_path);

    // Compile with cc
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let status = process::Command::new(&cc)
        .args([&c_path, "-o", &bin_path, "-lm"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("  compiled → {}", bin_path);
        }
        Ok(s) => {
            eprintln!("  {} exited with {}", cc, s);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("  failed to run {}: {}", cc, e);
            eprintln!("  (C source saved to {})", c_path);
            process::exit(1);
        }
    }
}

fn is_build_zen_path(path_str: &str) -> bool {
    Path::new(path_str)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "build.zen")
}

fn print_errors(errs: &[zen::error::CompileError], files: &FileTable) {
    for err in errs {
        let diag: zen::error::Diagnostic = err.clone().into();
        print_diagnostic(&diag, files);
    }
}

fn print_diagnostic(diag: &zen::error::Diagnostic, files: &FileTable) {
    let severity = match diag.severity {
        zen::error::Severity::Error => "error",
        zen::error::Severity::Warning => "warning",
        zen::error::Severity::Info => "info",
        zen::error::Severity::Hint => "hint",
    };

    if let Some(span) = diag.span {
        let path = files.get_path(span.file_id).unwrap_or("<unknown>");
        if let Some((line, col)) = files.line_col(span.file_id, span.start) {
            eprintln!(
                "{}:{}:{}: {}: {}",
                path,
                line + 1,
                col + 1,
                severity,
                diag.message
            );
        } else {
            eprintln!("{}: {}: {}", path, severity, diag.message);
        }
    } else {
        eprintln!("{}: {}", severity, diag.message);
    }

    for label in &diag.labels {
        let path = files.get_path(label.span.file_id).unwrap_or("<unknown>");
        if let Some((line, col)) = files.line_col(label.span.file_id, label.span.start) {
            eprintln!("  --> {}:{}:{}: {}", path, line + 1, col + 1, label.message);
        }
    }
}
