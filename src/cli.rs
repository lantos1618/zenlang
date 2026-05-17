use std::path::Path;
use std::process;

mod build_graph_execution;
mod build_graph_loading;
mod compile;
mod diagnostics;
mod frontend;
mod json_emit;
mod usage;

use build_graph_execution::{
    check_build_graph_sources, executable_build_targets, single_executable_build_target,
    test_build_targets, validate_build_graph_sources,
};
use build_graph_loading::load_build_graph;
use compile::{compile_file_to_binary, compile_file_to_c_source, typed_program_to_c_source};
use diagnostics::{print_diagnostic, print_errors};
use frontend::{graph_frontend, load_module_graph};
use json_emit::{
    cmd_emit_json_ast, cmd_emit_json_build_graph, cmd_emit_json_diagnostics, cmd_emit_json_symbols,
    cmd_emit_json_typed,
};
use usage::print_usage;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
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
        "test" => {
            if args.len() < 3 {
                eprintln!("Usage: zen test <build.zen>");
                process::exit(1);
            }
            cmd_test(&args[2]);
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
        let build_path = Path::new(path_str);
        let base_dir = build_path.parent().unwrap_or_else(|| Path::new("."));
        validate_build_graph_sources(base_dir, &graph);
        check_build_graph_sources(base_dir, &graph);
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

fn cmd_test(path_str: &str) {
    if !is_build_zen_path(path_str) {
        eprintln!("error: zen test expects a build.zen file");
        process::exit(1);
    }

    for target in test_build_targets(path_str) {
        if let Err(err) = std::fs::create_dir_all(&target.out_dir) {
            eprintln!("error creating {}: {}", target.out_dir.display(), err);
            process::exit(1);
        }

        let bin_path = compile_file_to_binary(
            target
                .root_path
                .to_str()
                .unwrap_or(&target.root_source_file),
            Some(&target.out_dir),
            Some(&target.name),
        );
        let run = process::Command::new(&bin_path).status();
        match run {
            Ok(status) if status.success() => {
                println!("  test {} passed", target.name);
            }
            Ok(status) => {
                eprintln!("  test {} exited with {}", target.name, status);
                process::exit(1);
            }
            Err(err) => {
                eprintln!("  failed to run test {}: {}", target.name, err);
                process::exit(1);
            }
        }
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
    for target in executable_build_targets(path_str) {
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
