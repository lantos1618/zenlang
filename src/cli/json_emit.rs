use std::path::Path;
use std::process;

use zen::error::FileTable;
use zen::module_system::ResolvedModuleGraph;
use zen::typechecker::TypeChecker;

use super::EmitJsonMode;

fn print_json_or_exit<E: std::fmt::Display>(result: Result<String, E>) {
    match result {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

fn emit_module_graph_json<E: std::fmt::Display>(
    path_str: &str,
    rejection_message: &'static str,
    serialize: impl FnOnce(&ResolvedModuleGraph) -> Result<String, E>,
) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_emit(path_str, rejection_message);
    let (graph, _files) = super::load_module_graph(path_str);
    print_json_or_exit(serialize(&graph));
}

fn emit_typed_json<E: std::fmt::Display>(
    path_str: &str,
    rejection_message: &'static str,
    serialize: impl FnOnce(&zen::ast::typed::TypedProgram) -> Result<String, E>,
) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_emit(path_str, rejection_message);
    let typed = super::graph_frontend(path_str);
    print_json_or_exit(serialize(&typed));
}

pub(super) fn cmd_emit_json(mode: EmitJsonMode, path_str: &str) {
    match mode {
        EmitJsonMode::Ast => emit_module_graph_json(
            path_str,
            "compiler-owned AST JSON emission rejects hand-authored JSON IR before it can override unchecked syntax trees",
            zen::ir_json::ast_graph_to_json,
        ),
        EmitJsonMode::Symbols => emit_module_graph_json(
            path_str,
            "compiler-owned symbols JSON emission rejects hand-authored resolver IR before it can override symbol metadata",
            zen::ir_json::symbols_graph_to_json,
        ),
        EmitJsonMode::Typed => emit_typed_json(
            path_str,
            "compiler-owned typed JSON emission rejects hand-authored JSON IR before it can override checked types or layouts",
            zen::ir_json::typed_program_to_json,
        ),
        EmitJsonMode::Diagnostics => emit_json_diagnostics(path_str),
        EmitJsonMode::BuildGraph => emit_json_build_graph(path_str),
        EmitJsonMode::Hir => emit_typed_json(
            path_str,
            "compiler-owned IR schemas reject hand-authored HIR JSON before it can override checked types or layouts",
            zen::ir_json::hir_program_to_json,
        ),
        EmitJsonMode::Mir => emit_typed_json(
            path_str,
            "compiler-owned IR schemas reject hand-authored MIR JSON before it can override checked types or layouts",
            zen::ir_json::mir_program_to_json,
        ),
        EmitJsonMode::Layout => emit_typed_json(
            path_str,
            "compiler-owned layout schemas reject hand-authored layout IR before it can override compiler-owned types or layouts",
            zen::ir_json::layout_program_to_json,
        ),
        EmitJsonMode::TargetYaml => emit_json_target_yaml(path_str),
    }
}

fn emit_json_diagnostics(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_emit(
        path_str,
        "compiler-owned diagnostics JSON emission rejects hand-authored diagnostic IR before it can override compiler diagnostics",
    );

    let path = super::require_existing_path(path_str);
    let mut files = FileTable::default();
    let mut diagnostics = match zen::module_system::load_module_graph(path, &mut files) {
        Ok(graph) => {
            let mut checker = TypeChecker::new();
            match checker.check_module_graph_entry(&graph) {
                Ok(_) => Vec::new(),
                Err(diags) => diags,
            }
        }
        Err(errs) => errs.into_iter().map(Into::into).collect(),
    };

    diagnostics.sort_by_key(|diagnostic| {
        diagnostic
            .span
            .map_or((u32::MAX, u32::MAX, u32::MAX), |span| {
                (span.file_id, span.start, span.end)
            })
    });

    let has_errors = !diagnostics.is_empty();
    print_json_or_exit(zen::ir_json::diagnostics_to_json(&diagnostics, &files));
    if has_errors {
        process::exit(1);
    }
}

fn emit_json_build_graph(path_str: &str) {
    super::reject_hand_authored_json_for_emit(
        path_str,
        "compiler-owned build graph JSON emission rejects hand-authored graph IR before it can override deterministic build metadata",
    );
    let graph = super::load_build_graph(path_str);
    print_json_or_exit(graph.canonical_json());
}

fn emit_json_target_yaml(path_str: &str) {
    match zen::target_yaml::target_yaml_file_to_json(Path::new(path_str)) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}
