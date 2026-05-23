use std::path::Path;
use std::process;

use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

fn print_json_or_exit<E: std::fmt::Display>(result: Result<String, E>) {
    match result {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

fn emit_typed_json<E: std::fmt::Display>(
    path_str: &str,
    reject_hand_authored: fn(&str),
    serialize: impl FnOnce(&zen::ast::typed::TypedProgram) -> Result<String, E>,
) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    reject_hand_authored(path_str);
    let typed = super::graph_frontend(path_str);
    print_json_or_exit(serialize(&typed));
}

pub(super) fn cmd_emit_json_ast(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_ast_emit(path_str);
    let (graph, _files) = super::load_module_graph(path_str);
    print_json_or_exit(zen::ir_json::ast_graph_to_json(&graph));
}

pub(super) fn cmd_emit_json_symbols(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_symbols_emit(path_str);
    let (graph, _files) = super::load_module_graph(path_str);
    print_json_or_exit(zen::ir_json::symbols_graph_to_json(&graph));
}

pub(super) fn cmd_emit_json_typed(path_str: &str) {
    emit_typed_json(
        path_str,
        super::reject_hand_authored_json_for_typed_emit,
        zen::ir_json::typed_program_to_json,
    );
}

pub(super) fn cmd_emit_json_layout(path_str: &str) {
    emit_typed_json(
        path_str,
        super::reject_hand_authored_json_for_layout_emit,
        zen::ir_json::layout_program_to_json,
    );
}

pub(super) fn cmd_emit_json_hir(path_str: &str) {
    emit_typed_json(
        path_str,
        super::reject_hand_authored_json_for_hir_emit,
        zen::ir_json::hir_program_to_json,
    );
}

pub(super) fn cmd_emit_json_mir(path_str: &str) {
    emit_typed_json(
        path_str,
        super::reject_hand_authored_json_for_mir_emit,
        zen::ir_json::mir_program_to_json,
    );
}

pub(super) fn cmd_emit_json_diagnostics(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_diagnostics_emit(path_str);

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

pub(super) fn cmd_emit_json_build_graph(path_str: &str) {
    super::reject_hand_authored_json_for_build_graph_emit(path_str);
    let graph = super::load_build_graph(path_str);
    match graph.canonical_json() {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("json emit error: {}", err);
            process::exit(1);
        }
    }
}

pub(super) fn cmd_emit_json_target_yaml(path_str: &str) {
    match zen::target_yaml::target_yaml_file_to_json(Path::new(path_str)) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}
