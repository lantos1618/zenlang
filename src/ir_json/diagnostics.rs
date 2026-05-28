use serde::Serialize;

use crate::error::{Diagnostic, DiagnosticFact, FileTable, Span};

#[derive(Serialize)]
struct DiagnosticsJson<'a> {
    format: &'static str,
    semantic_status: &'static str,
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
    severity: &'static str,
    code: String,
    slug: String,
    phase: &'static str,
    category: &'static str,
    docs_path: &'static str,
    message: &'a str,
    span: Option<DiagnosticJsonSpan<'a>>,
    labels: Vec<DiagnosticJsonMessageSpan<'a>>,
    notes: &'a [String],
    context: Vec<DiagnosticJsonContext<'a>>,
    suggested_fixes: Vec<DiagnosticJsonSuggestedFix<'a>>,
    related: Vec<DiagnosticJsonMessageSpan<'a>>,
    facts: &'a [DiagnosticFact],
}

#[derive(Serialize)]
struct DiagnosticJsonMessageSpan<'a> {
    span: DiagnosticJsonSpan<'a>,
    message: &'a str,
}

#[derive(Serialize)]
struct DiagnosticJsonContext<'a> {
    span: DiagnosticJsonSpan<'a>,
    kind: &'a str,
    message: &'a str,
}

#[derive(Serialize)]
struct DiagnosticJsonSuggestedFix<'a> {
    kind: &'a str,
    title: &'a str,
    edits: Vec<DiagnosticJsonTextEdit<'a>>,
}

#[derive(Serialize)]
struct DiagnosticJsonTextEdit<'a> {
    span: DiagnosticJsonSpan<'a>,
    replacement: &'a str,
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
        format: "zen.diagnostics.v1",
        semantic_status: "diagnostic",
        files: (0..files.file_count())
            .filter_map(|id| {
                let id = id as u32;
                files
                    .get_path(id)
                    .map(|path| DiagnosticJsonFile { id, path })
            })
            .collect(),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| DiagnosticJson {
                severity: "error",
                code: diagnostic.code(),
                slug: diagnostic.slug(),
                phase: diagnostic.phase().as_str(),
                category: diagnostic.category().as_str(),
                docs_path: diagnostic.docs_path(),
                message: diagnostic.message.as_str(),
                span: diagnostic
                    .span
                    .and_then(|span| diagnostic_json_span(span, files)),
                labels: Vec::new(),
                notes: diagnostic.notes(),
                context: diagnostic_json_spanned(
                    diagnostic.context(),
                    files,
                    |frame| frame.span,
                    |frame, span| DiagnosticJsonContext {
                        span,
                        kind: frame.kind,
                        message: frame.message.as_str(),
                    },
                ),
                suggested_fixes: diagnostic
                    .suggested_fixes()
                    .iter()
                    .filter_map(|fix| {
                        let edits = diagnostic_json_spanned(
                            &fix.edits,
                            files,
                            |edit| edit.span,
                            |edit, span| DiagnosticJsonTextEdit {
                                span,
                                replacement: edit.replacement.as_str(),
                            },
                        );

                        (!edits.is_empty()).then_some(DiagnosticJsonSuggestedFix {
                            kind: fix.kind.as_str(),
                            title: fix.title.as_str(),
                            edits,
                        })
                    })
                    .collect(),
                related: diagnostic_json_spanned(
                    diagnostic.related(),
                    files,
                    |related| related.span,
                    |related, span| DiagnosticJsonMessageSpan {
                        span,
                        message: related.message.as_str(),
                    },
                ),
                facts: diagnostic.facts(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&graph)
}

fn diagnostic_json_spanned<'a, T, U>(
    items: &'a [T],
    files: &'a FileTable,
    item_span: impl Fn(&T) -> Span,
    item_json: impl Fn(&'a T, DiagnosticJsonSpan<'a>) -> U,
) -> Vec<U> {
    items
        .iter()
        .filter_map(|item| {
            diagnostic_json_span(item_span(item), files).map(|span| item_json(item, span))
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
