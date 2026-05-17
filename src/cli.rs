use std::path::Path;
use std::process;

mod build_graph_execution;
mod build_graph_loading;
mod compile;
mod diagnostics;
mod execution_commands;
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
use execution_commands::{cmd_build, cmd_build_graph, cmd_run_file, cmd_test};
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
