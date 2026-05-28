use std::process;

use zen::error::FileTable;
use zen::typechecker::TypeChecker;

pub(super) fn load_module_graph(
    path_str: &str,
) -> (zen::module_system::ResolvedModuleGraph, FileTable) {
    let path = super::require_existing_path(path_str);
    let mut files = FileTable::default();

    let graph = match zen::module_system::load_module_graph(path, &mut files) {
        Ok(graph) => graph,
        Err(errs) => {
            super::print_errors(&errs, &files);
            process::exit(1);
        }
    };

    (graph, files)
}

pub(super) fn graph_frontend(path_str: &str) -> zen::ast::typed::TypedProgram {
    let (graph, files) = load_module_graph(path_str);

    let mut checker = TypeChecker::new();
    match checker.check_module_graph_entry(&graph) {
        Ok(typed) => typed,
        Err(diags) => {
            for diag in &diags {
                super::print_diagnostic(diag, &files);
            }
            eprintln!("  {} error(s)", diags.len());
            process::exit(1);
        }
    }
}
