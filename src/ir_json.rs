use serde::Serialize;

use crate::ast::typed::TypedProgram;
use crate::ast::Program;
use crate::error::{Diagnostic, FileTable, Label, Span};
use crate::module_system::{ImportBinding, ResolvedModuleGraph};

#[derive(Serialize)]
struct AstJsonGraph<'a> {
    format: &'static str,
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

#[derive(Serialize)]
struct TypedJsonProgram<'a> {
    format: &'static str,
    program: &'a TypedProgram,
}

pub fn ast_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    let mut modules: Vec<_> = graph.modules().values().collect();
    modules.sort_by_key(|module| module.info.id.0);

    let graph = AstJsonGraph {
        format: "zen.ast.v0",
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

pub fn typed_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let graph = TypedJsonProgram {
        format: "zen.typed.v0",
        program,
    };

    serde_json::to_string_pretty(&graph)
}

#[derive(Serialize)]
struct DiagnosticsJson<'a> {
    format: &'static str,
    files: Vec<DiagnosticJsonFile<'a>>,
    diagnostics: Vec<DiagnosticJson<'a>>,
}

#[derive(Serialize)]
struct DiagnosticJsonFile<'a> {
    id: u32,
    path: &'a str,
}

#[derive(Serialize)]
struct DiagnosticJson<'a> {
    severity: String,
    code: &'a str,
    message: &'a str,
    span: Option<DiagnosticJsonSpan<'a>>,
    labels: Vec<DiagnosticJsonLabel<'a>>,
    notes: &'a [String],
}

#[derive(Serialize)]
struct DiagnosticJsonLabel<'a> {
    span: DiagnosticJsonSpan<'a>,
    message: &'a str,
}

#[derive(Serialize)]
struct DiagnosticJsonSpan<'a> {
    file_id: u32,
    path: &'a str,
    start: u32,
    end: u32,
    line: u32,
    column: u32,
}

pub fn diagnostics_to_json(
    diagnostics: &[Diagnostic],
    files: &FileTable,
) -> serde_json::Result<String> {
    let graph = DiagnosticsJson {
        format: "zen.diagnostics.v0",
        files: diagnostic_json_files(files),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| DiagnosticJson {
                severity: diagnostic.severity.to_string(),
                code: diagnostic.code.as_str(),
                message: diagnostic.message.as_str(),
                span: diagnostic
                    .span
                    .and_then(|span| diagnostic_json_span(span, files)),
                labels: diagnostic_json_labels(&diagnostic.labels, files),
                notes: &diagnostic.notes,
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

fn diagnostic_json_files(files: &FileTable) -> Vec<DiagnosticJsonFile<'_>> {
    (0..files.file_count())
        .filter_map(|id| {
            let id = id as u32;
            files
                .get_path(id)
                .map(|path| DiagnosticJsonFile { id, path })
        })
        .collect()
}

fn diagnostic_json_labels<'a>(
    labels: &'a [Label],
    files: &'a FileTable,
) -> Vec<DiagnosticJsonLabel<'a>> {
    labels
        .iter()
        .filter_map(|label| {
            diagnostic_json_span(label.span, files).map(|span| DiagnosticJsonLabel {
                span,
                message: label.message.as_str(),
            })
        })
        .collect()
}

fn diagnostic_json_span(span: Span, files: &FileTable) -> Option<DiagnosticJsonSpan<'_>> {
    let path = files.get_path(span.file_id)?;
    let (line, column) = files.line_col(span.file_id, span.start)?;
    Some(DiagnosticJsonSpan {
        file_id: span.file_id,
        path,
        start: span.start,
        end: span.end,
        line: line + 1,
        column: column + 1,
    })
}
