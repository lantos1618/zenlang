use std::path::Path;
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
use build_graph_targets::{
    executable_build_target, test_build_target, BuildGraphExecutableTarget, BuildGraphTestTarget,
};
use check_emit_commands::{cmd_check, cmd_emit};
use compile::{compile_file_to_binary, compile_file_to_c_source, typed_program_to_c_source};
use diagnostics::{print_diagnostic, print_errors};
use emit_json_mode::{emit_json_usage, EmitJsonMode};
use execution_commands::{cmd_build, cmd_build_graph, cmd_run_file, cmd_test};
use frontend::{graph_frontend, load_module_graph};
use json_emit::{
    cmd_emit_json_ast, cmd_emit_json_build_graph, cmd_emit_json_diagnostics, cmd_emit_json_hir,
    cmd_emit_json_layout, cmd_emit_json_mir, cmd_emit_json_symbols, cmd_emit_json_target_yaml,
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
                eprintln!("{}", emit_json_usage());
                process::exit(1);
            }
            let mode = args[2].as_str();
            match mode.parse::<EmitJsonMode>() {
                Ok(mode) => {
                    if let Some(message) = mode.gate_message() {
                        eprintln!("error: {message}");
                        process::exit(1);
                    }
                    match mode {
                        EmitJsonMode::Ast => cmd_emit_json_ast(&args[3]),
                        EmitJsonMode::Symbols => cmd_emit_json_symbols(&args[3]),
                        EmitJsonMode::Typed => cmd_emit_json_typed(&args[3]),
                        EmitJsonMode::Diagnostics => cmd_emit_json_diagnostics(&args[3]),
                        EmitJsonMode::BuildGraph => cmd_emit_json_build_graph(&args[3]),
                        EmitJsonMode::Hir => cmd_emit_json_hir(&args[3]),
                        EmitJsonMode::Mir => cmd_emit_json_mir(&args[3]),
                        EmitJsonMode::Layout => cmd_emit_json_layout(&args[3]),
                        EmitJsonMode::TargetYaml => cmd_emit_json_target_yaml(&args[3]),
                    }
                }
                Err(()) => {
                    eprintln!("{}", emit_json_usage());
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

#[derive(Clone, Copy)]
enum CompilerOwnedJsonBoundary {
    Ast,
    Typed,
    Symbols,
    Diagnostics,
    BuildGraph,
    Hir,
    Mir,
    Layout,
}

impl CompilerOwnedJsonBoundary {
    fn rejection_message(self) -> &'static str {
        match self {
            Self::Ast => "compiler-owned AST JSON emission rejects hand-authored JSON IR before it can override unchecked syntax trees",
            Self::Typed => "compiler-owned typed JSON emission rejects hand-authored JSON IR before it can override checked types or layouts",
            Self::Symbols => "compiler-owned symbols JSON emission rejects hand-authored resolver IR before it can override symbol metadata",
            Self::Diagnostics => "compiler-owned diagnostics JSON emission rejects hand-authored diagnostic IR before it can override compiler diagnostics",
            Self::BuildGraph => "compiler-owned build graph JSON emission rejects hand-authored graph IR before it can override deterministic build metadata",
            Self::Hir => "compiler-owned IR schemas reject hand-authored HIR JSON before it can override checked types or layouts",
            Self::Mir => "compiler-owned IR schemas reject hand-authored MIR JSON before it can override checked types or layouts",
            Self::Layout => "compiler-owned layout schemas reject hand-authored layout IR before it can override compiler-owned types or layouts",
        }
    }
}

fn reject_hand_authored_json_for_emit(path_str: &str, boundary: CompilerOwnedJsonBoundary) {
    if has_json_extension(path_str) {
        eprintln!("error: {}", boundary.rejection_message());
        process::exit(1);
    }
}

fn reject_hand_authored_json_for_ast_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Ast);
}

fn reject_hand_authored_json_for_typed_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Typed);
}

fn reject_hand_authored_json_for_symbols_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Symbols);
}

fn reject_hand_authored_json_for_diagnostics_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Diagnostics);
}

fn reject_hand_authored_json_for_build_graph_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::BuildGraph);
}

fn reject_hand_authored_json_for_hir_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Hir);
}

fn reject_hand_authored_json_for_mir_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Mir);
}

fn reject_hand_authored_json_for_layout_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Layout);
}

fn has_json_extension(path_str: &str) -> bool {
    Path::new(path_str)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}
