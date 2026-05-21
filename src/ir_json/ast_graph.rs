use serde::Serialize;

use crate::ast::Program;
use crate::error::Span;
use crate::module_system::{ImportBinding, ResolvedModuleGraph};

#[derive(Serialize)]
struct AstJsonGraph<'a> {
    format: &'static str,
    schema_version: u32,
    semantic_status: &'static str,
    entry_module: u32,
    modules: Vec<AstJsonModule<'a>>,
}

#[derive(Serialize)]
struct AstJsonModule<'a> {
    id: u32,
    package_id: u32,
    canonical_path: &'a str,
    imports: Vec<AstJsonImport<'a>>,
    program: &'a Program,
}

#[derive(Serialize)]
struct AstJsonImport<'a> {
    local_name: &'a str,
    source_module: u32,
    source_symbol: &'a str,
    span: Span,
}

pub(super) fn ast_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    let mut modules: Vec<_> = graph.modules().values().collect();
    modules.sort_by_key(|module| module.info.id.0);

    let graph = AstJsonGraph {
        format: "zen.ast.v0",
        schema_version: 0,
        semantic_status: "unchecked",
        entry_module: graph.entry.0,
        modules: modules
            .into_iter()
            .map(|module| AstJsonModule {
                id: module.info.id.0,
                package_id: module.info.package_id.0,
                canonical_path: module.info.canonical_path.as_str(),
                imports: ast_json_imports(&module.imports),
                program: &module.program,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&graph)
}

fn ast_json_imports(imports: &[ImportBinding]) -> Vec<AstJsonImport<'_>> {
    imports
        .iter()
        .map(|import| AstJsonImport {
            local_name: import.local_name.as_str(),
            source_module: import.source_module.0,
            source_symbol: import.source_symbol.as_str(),
            span: import.span,
        })
        .collect()
}
