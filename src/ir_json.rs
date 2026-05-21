use serde::Serialize;

mod ast_graph;
mod diagnostics;
mod hir;
mod layout;
mod mir;
mod symbols;

use crate::ast::typed::TypedProgram;
use crate::error::{Diagnostic, FileTable};
use crate::module_system::ResolvedModuleGraph;

#[derive(Serialize)]
struct TypedJsonProgram<'a> {
    format: &'static str,
    semantic_status: &'static str,
    program: &'a TypedProgram,
}

pub fn ast_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    ast_graph::ast_graph_to_json(graph)
}

pub fn symbols_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    symbols::symbols_graph_to_json(graph)
}

pub fn typed_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let graph = TypedJsonProgram {
        format: "zen.typed.v0",
        semantic_status: "checked",
        program,
    };

    serde_json::to_string_pretty(&graph)
}

pub fn hir_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    hir::program_to_json(program)
}

pub fn layout_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    layout::program_to_json(program)
}

pub fn mir_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    mir::program_to_json(program)
}

pub fn diagnostics_to_json(
    diagnostics: &[Diagnostic],
    files: &FileTable,
) -> serde_json::Result<String> {
    diagnostics::diagnostics_to_json(diagnostics, files)
}
