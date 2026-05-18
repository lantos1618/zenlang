use std::path::Path;
use std::process;

use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

pub(super) fn cmd_emit_json_ast(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_ast_emit(path_str);
    let (graph, _files) = super::load_module_graph(path_str);
    match zen::ir_json::ast_graph_to_json(&graph) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

pub(super) fn cmd_emit_json_symbols(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_symbols_emit(path_str);
    let (graph, _files) = super::load_module_graph(path_str);
    match zen::ir_json::symbols_graph_to_json(&graph) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

pub(super) fn cmd_emit_json_typed(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_typed_emit(path_str);
    let typed = super::graph_frontend(path_str);
    match zen::ir_json::typed_program_to_json(&typed) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
}

pub(super) fn cmd_emit_json_layout(path_str: &str) {
    super::reject_build_zen_for_emit_json_mode(path_str);
    super::reject_hand_authored_json_for_layout_emit(path_str);
    let typed = super::graph_frontend(path_str);
    match zen::ir_json::layout_program_to_json(&typed) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("json emit error: {}", e);
            process::exit(1);
        }
    }
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
