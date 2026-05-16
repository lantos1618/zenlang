use std::path::{Path, PathBuf};
use std::process;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("zen compiler v0.8.0");
        eprintln!("Usage: zen <command> [args]");
        eprintln!("Commands:");
        eprintln!("  check <file>   Parse and typecheck a .zen file");
        eprintln!("  build <file>   Compile a .zen file to a binary");
        eprintln!("  build-graph <build.zen>   Compile one target from deterministic build graph");
        eprintln!("  emit  <file>   Emit C source (no compilation)");
        eprintln!("  emit-json ast <file>   Emit resolved AST JSON");
        eprintln!("  emit-json symbols <file>   Emit resolver symbol tables JSON");
        eprintln!("  emit-json typed <file>   Emit checked typed program JSON");
        eprintln!("  emit-json diagnostics <file>   Emit diagnostics JSON");
        eprintln!("  emit-json build-graph <build.zen>   Emit deterministic build graph JSON");
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
        "build-graph" => {
            if args.len() < 3 {
                eprintln!("Usage: zen build-graph <build.zen>");
                process::exit(1);
            }
            cmd_build_graph(&args[2]);
        }
        "emit" => {
            if args.len() < 3 {
                eprintln!("Usage: zen emit <file.zen>");
                process::exit(1);
            }
            cmd_emit(&args[2]);
        }
        "emit-json" => {
            if args.len() < 4 {
                eprintln!("Usage: zen emit-json <ast|symbols|typed|diagnostics> <file.zen>");
                process::exit(1);
            }
            match args[2].as_str() {
                "ast" => cmd_emit_json_ast(&args[3]),
                "symbols" => cmd_emit_json_symbols(&args[3]),
                "typed" => cmd_emit_json_typed(&args[3]),
                "diagnostics" => cmd_emit_json_diagnostics(&args[3]),
                "build-graph" => cmd_emit_json_build_graph(&args[3]),
                _ => {
                    eprintln!(
                        "Usage: zen emit-json <ast|symbols|typed|diagnostics|build-graph> <file.zen>"
                    );
                    process::exit(1);
                }
            }
        }
        arg if arg.ends_with(".zen") => {
            cmd_run_file(arg);
        }
        other => {
            eprintln!("unknown command: {}", other);
            process::exit(1);
        }
    }
}

fn cmd_check(path_str: &str) {
    if is_build_zen_path(path_str) {
        let graph = load_build_graph(path_str);
        println!("  {} build targets — ok", graph.targets().len());
        return;
    }

    let typed = graph_frontend(path_str);
    println!(
        "  {} functions, {} types — ok",
        typed.functions.len(),
        typed.types.len()
    );
}

fn load_module_graph(path_str: &str) -> (zen::module_system::ResolvedModuleGraph, FileTable) {
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

    (graph, files)
}

fn graph_frontend(path_str: &str) -> zen::ast::typed::TypedProgram {
    let (graph, files) = load_module_graph(path_str);

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
            let errors = diags
                .iter()
                .filter(|d| d.severity == zen::error::Severity::Error)
                .count();
            eprintln!("  {} error(s)", errors);
            process::exit(1);
        }
    }
}

fn cmd_emit_json_ast(path_str: &str) {
    reject_build_zen_for_emit_json_mode(path_str);
    let (graph, _files) = load_module_graph(path_str);
    match zen::ir_json::ast_graph_to_json(&graph) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_emit_json_symbols(path_str: &str) {
    reject_build_zen_for_emit_json_mode(path_str);
    let (graph, _files) = load_module_graph(path_str);
    match zen::ir_json::symbols_graph_to_json(&graph) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_emit_json_typed(path_str: &str) {
    reject_build_zen_for_emit_json_mode(path_str);
    let typed = graph_frontend(path_str);
    match zen::ir_json::typed_program_to_json(&typed) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_emit_json_diagnostics(path_str: &str) {
    reject_build_zen_for_emit_json_mode(path_str);

    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("error: file not found: {}", path_str);
        process::exit(1);
    }

    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let mut diagnostics = match module_system.load_module_graph(path, &mut files) {
        Ok(graph) => {
            let mut checker = TypeChecker::new();
            match checker.check_module_graph_entry(&graph) {
                Ok(_) => checker.diagnostics().to_vec(),
                Err(diags) => diags,
            }
        }
        Err(errs) => errs.into_iter().map(Into::into).collect(),
    };

    diagnostics.sort_by_key(|diagnostic| {
        diagnostic
            .span
            .map(|span| (span.file_id, span.start, span.end))
            .unwrap_or((u32::MAX, u32::MAX, u32::MAX))
    });

    let has_errors = diagnostics.iter().any(|diagnostic| diagnostic.is_error());
    match zen::ir_json::diagnostics_to_json(&diagnostics, &files) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }

    if has_errors {
        process::exit(1);
    }
}

fn cmd_emit_json_build_graph(path_str: &str) {
    let graph = load_build_graph(path_str);
    match graph.canonical_json() {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("json emit error: {}", err);
            process::exit(1);
        }
    }
}

fn load_build_graph(path_str: &str) -> zen::build_graph::BuildGraph {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("error: file not found: {}", path_str);
        process::exit(1);
    }
    if !is_build_zen_path(path_str) {
        eprintln!("error: emit-json build-graph expects a build.zen file");
        process::exit(1);
    }

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("error reading {}: {}", path_str, err);
            process::exit(1);
        }
    };

    let mut files = FileTable::new();
    let file_id = files.add_file(path_str.to_string(), source.clone());
    let tokens = match zen::lexer::tokenize(&source, file_id) {
        Ok(tokens) => tokens,
        Err(err) => {
            print_errors(&[err], &files);
            process::exit(1);
        }
    };
    let program = match zen::parser::parse(tokens, file_id) {
        Ok(program) => program,
        Err(errs) => {
            print_errors(&errs, &files);
            process::exit(1);
        }
    };
    let graph = match zen::build_graph::BuildGraph::from_build_program(&program) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("build graph error: {}", err);
            process::exit(1);
        }
    };

    graph
}

fn cmd_emit(path_str: &str) {
    if is_build_zen_path(path_str) {
        let target = single_executable_build_target(path_str);
        print!("{}", compile_file_to_c_source(&target.root_path));
        return;
    }

    let typed = graph_frontend(path_str);
    print!("{}", typed_program_to_c_source(&typed));
}

fn cmd_build(path_str: &str) {
    if is_build_zen_path(path_str) {
        cmd_build_graph(path_str);
    } else {
        compile_file_to_binary(path_str, None, None);
    }
}

fn cmd_run_file(path_str: &str) {
    if is_build_zen_path(path_str) {
        cmd_build_graph(path_str);
    } else {
        compile_file_to_binary(path_str, None, None);
    }
}

fn cmd_build_graph(path_str: &str) {
    let target = single_executable_build_target(path_str);
    if let Err(err) = std::fs::create_dir_all(&target.out_dir) {
        eprintln!("error creating {}: {}", target.out_dir.display(), err);
        process::exit(1);
    }

    compile_file_to_binary(
        target
            .root_path
            .to_str()
            .unwrap_or(&target.root_source_file),
        Some(&target.out_dir),
        Some(&target.name),
    );
}

struct BuildGraphExecutableTarget {
    name: String,
    root_source_file: String,
    root_path: PathBuf,
    out_dir: PathBuf,
}

fn single_executable_build_target(path_str: &str) -> BuildGraphExecutableTarget {
    let graph = load_build_graph(path_str);
    let [target] = graph.targets() else {
        eprintln!(
            "build graph execution supports exactly one target, found {}",
            graph.targets().len()
        );
        process::exit(1);
    };

    let build_path = Path::new(path_str);
    let base_dir = build_path.parent().unwrap_or_else(|| Path::new("."));
    let zen::build_graph::BuildTargetKind::Executable {
        root_source_file,
        out_dir,
    } = target.kind();
    let root_path = base_dir.join(root_source_file);
    if !root_path.exists() {
        eprintln!(
            "build graph target `{}` root source not found: {}",
            target.name(),
            root_source_file
        );
        process::exit(1);
    }
    let out_dir = base_dir.join(out_dir);

    BuildGraphExecutableTarget {
        name: target.name().to_string(),
        root_source_file: root_source_file.clone(),
        root_path,
        out_dir,
    }
}

fn compile_file_to_c_source(path: &Path) -> String {
    let path_str = path.to_str().unwrap_or_else(|| {
        eprintln!("error: non-utf8 source path: {}", path.display());
        process::exit(1);
    });
    let typed = graph_frontend(path_str);
    typed_program_to_c_source(&typed)
}

fn typed_program_to_c_source(typed: &zen::ast::typed::TypedProgram) -> String {
    let backend = CBackend;
    match backend.generate(typed) {
        Ok(c_source) => c_source,
        Err(e) => {
            eprintln!("codegen error: {}", e);
            process::exit(1);
        }
    }
}

fn compile_file_to_binary(path_str: &str, output_dir: Option<&Path>, output_name: Option<&str>) {
    let c_source = compile_file_to_c_source(Path::new(path_str));

    // Determine output paths
    let stem = Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    let output_stem = output_name.unwrap_or(stem);
    let c_path = output_dir
        .map(|dir| dir.join(format!("{output_stem}.c")))
        .unwrap_or_else(|| Path::new(&format!("{output_stem}.c")).to_path_buf());
    let bin_path = output_dir
        .map(|dir| dir.join(output_stem))
        .unwrap_or_else(|| Path::new(output_stem).to_path_buf());

    // Write C source
    if let Err(e) = std::fs::write(&c_path, &c_source) {
        eprintln!("error writing {}: {}", c_path.display(), e);
        process::exit(1);
    }
    println!("  emitted {}", c_path.display());

    // Compile with cc
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let status = process::Command::new(&cc)
        .arg(&c_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-lm")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("  compiled → {}", bin_path.display());
        }
        Ok(s) => {
            eprintln!("  {} exited with {}", cc, s);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("  failed to run {}: {}", cc, e);
            eprintln!("  (C source saved to {})", c_path.display());
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

fn reject_build_zen_for_emit_json_mode(path_str: &str) {
    if is_build_zen_path(path_str) {
        eprintln!(
            "error: this emit-json mode does not support build.zen; use `emit-json build-graph`"
        );
        process::exit(1);
    }
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
