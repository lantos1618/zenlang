use std::path::Path;
use std::process;

mod build_graph_execution;
mod build_graph_loading;
mod build_graph_sources;
mod build_graph_targets;
mod check_emit_commands;
mod compile;
mod diagnostics;
mod execution_commands;
mod frontend;
mod json_emit;
mod usage;

use build_graph_execution::{
    executable_build_targets, single_executable_build_target, test_build_targets,
    BuildGraphExecutionKind,
};
use build_graph_loading::load_build_graph;
use build_graph_sources::{
    check_build_graph_sources, validate_build_graph_sources, validate_non_executed_target_sources,
};
use build_graph_targets::{
    executable_build_target, test_build_target, BuildGraphExecutableTarget, BuildGraphTestTarget,
};
use check_emit_commands::{cmd_check, cmd_emit};
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
