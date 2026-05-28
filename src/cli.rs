use std::path::{Path, PathBuf};
use std::process;

mod build_graph_execution;
mod build_graph_loading;
mod build_graph_sources;
mod build_graph_targets;
mod check_emit_commands;
mod compile;
mod diagnostics;
mod emit_json_mode;
mod execution_commands;
mod frontend;
mod json_emit;
mod usage;

use build_graph_execution::{
    executable_build_targets, single_executable_build_target, test_build_targets,
};
use build_graph_loading::load_build_graph;
use build_graph_sources::{
    check_build_graph_sources, validate_build_graph_sources, validate_graph_only_library_sources,
};
use build_graph_targets::{executable_build_target, test_build_target, BuildGraphTarget};
use check_emit_commands::{cmd_check, cmd_emit};
use compile::{compile_file_to_binary, compile_file_to_c_source, typed_program_to_c_source};
use diagnostics::{print_diagnostic, print_errors};
use emit_json_mode::{emit_json_usage, EmitJsonMode};
use execution_commands::{cmd_build, cmd_build_graph, cmd_test};
use frontend::{graph_frontend, load_module_graph};
use json_emit::cmd_emit_json;
use usage::print_usage;

fn require_existing_path(path_str: &str) -> &Path {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("error: file not found: {}", path_str);
        process::exit(1);
    }
    path
}

fn required_arg(args: &[String], index: usize, usage: impl AsRef<str>) -> &str {
    if args.len() <= index {
        eprintln!("{}", usage.as_ref());
        process::exit(1);
    }
    &args[index]
}

fn require_target_source_path(
    base_dir: &Path,
    target: &zen::build_graph::BuildTarget,
    source: &str,
    label: &str,
) -> PathBuf {
    let path = base_dir.join(source);
    if !path.exists() {
        eprintln!(
            "build graph target `{}` {} not found: {}",
            target.name, label, source
        );
        process::exit(1);
    }
    path
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "check" => cmd_check(required_arg(&args, 2, "Usage: zen check <file.zen>")),
        "build" => cmd_build(required_arg(&args, 2, "Usage: zen build <file.zen>")),
        "test" => cmd_test(required_arg(&args, 2, "Usage: zen test <build.zen>")),
        "build-graph" => {
            cmd_build_graph(required_arg(&args, 2, "Usage: zen build-graph <build.zen>"))
        }
        "emit" => cmd_emit(required_arg(&args, 2, "Usage: zen emit <file.zen>")),
        "emit-json" => {
            let usage = emit_json_usage();
            let mode = required_arg(&args, 2, &usage);
            let path = required_arg(&args, 3, &usage);
            match mode.parse::<EmitJsonMode>() {
                Ok(mode) => cmd_emit_json(mode, path),
                Err(()) => {
                    eprintln!("{usage}");
                    process::exit(1);
                }
            }
        }
        arg if arg.ends_with(".zen") => cmd_build(arg),
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

fn reject_hand_authored_json_for_emit(path_str: &str, message: &str) {
    if Path::new(path_str)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        eprintln!("error: {message}");
        process::exit(1);
    }
}
