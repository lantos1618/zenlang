use std::path::Path;
use std::process;
use std::str::FromStr;

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
use execution_commands::{cmd_build, cmd_build_graph, cmd_run_file, cmd_test};
use frontend::{graph_frontend, load_module_graph};
use json_emit::{
    cmd_emit_json_ast, cmd_emit_json_build_graph, cmd_emit_json_diagnostics, cmd_emit_json_symbols,
    cmd_emit_json_typed,
};
use usage::print_usage;

#[derive(Clone, Copy)]
enum EmitJsonMode {
    Ast,
    Symbols,
    Typed,
    Diagnostics,
    BuildGraph,
    Hir,
    Mir,
    Layout,
    TargetYaml,
}

impl EmitJsonMode {
    const AST: &'static str = "ast";
    const SYMBOLS: &'static str = "symbols";
    const TYPED: &'static str = "typed";
    const DIAGNOSTICS: &'static str = "diagnostics";
    const BUILD_GRAPH: &'static str = "build-graph";
    const HIR: &'static str = "hir";
    const MIR: &'static str = "mir";
    const LAYOUT: &'static str = "layout";
    const TARGET_YAML: &'static str = "target-yaml";

    const ORDERED: [Self; 9] = [
        Self::Ast,
        Self::Symbols,
        Self::Typed,
        Self::Diagnostics,
        Self::BuildGraph,
        Self::Hir,
        Self::Mir,
        Self::Layout,
        Self::TargetYaml,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Ast => Self::AST,
            Self::Symbols => Self::SYMBOLS,
            Self::Typed => Self::TYPED,
            Self::Diagnostics => Self::DIAGNOSTICS,
            Self::BuildGraph => Self::BUILD_GRAPH,
            Self::Hir => Self::HIR,
            Self::Mir => Self::MIR,
            Self::Layout => Self::LAYOUT,
            Self::TargetYaml => Self::TARGET_YAML,
        }
    }

    fn usage() -> String {
        Self::ORDERED
            .iter()
            .map(|mode| mode.as_str())
            .collect::<Vec<_>>()
            .join("|")
    }

    fn gate_message(self) -> Option<&'static str> {
        match self {
            Self::Hir => Some("HIR JSON emission is gated until schema and golden tests exist"),
            Self::Mir => Some("MIR JSON emission is gated until schema and golden tests exist"),
            Self::Layout => Some("type layout JSON emission is gated until ABI layout tests exist"),
            Self::TargetYaml => Some(
                "target YAML validation is gated until schemas and negative validation tests exist",
            ),
            Self::Ast | Self::Symbols | Self::Typed | Self::Diagnostics | Self::BuildGraph => None,
        }
    }
}

impl FromStr for EmitJsonMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            Self::AST => Ok(Self::Ast),
            Self::SYMBOLS => Ok(Self::Symbols),
            Self::TYPED => Ok(Self::Typed),
            Self::DIAGNOSTICS => Ok(Self::Diagnostics),
            Self::BUILD_GRAPH => Ok(Self::BuildGraph),
            Self::HIR => Ok(Self::Hir),
            Self::MIR => Ok(Self::Mir),
            Self::LAYOUT => Ok(Self::Layout),
            Self::TARGET_YAML => Ok(Self::TargetYaml),
            _ => Err(()),
        }
    }
}

fn emit_json_usage() -> String {
    format!(
        "Usage: zen emit-json <{}> <file.zen>",
        EmitJsonMode::usage()
    )
}

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
                        EmitJsonMode::Hir
                        | EmitJsonMode::Mir
                        | EmitJsonMode::Layout
                        | EmitJsonMode::TargetYaml => unreachable!("gated emit-json mode exited"),
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
