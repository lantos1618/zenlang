use serde::Serialize;

use crate::error::{ContextFrame, Diagnostic, FileTable, Label, Span};

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
    severity: String,
    code: &'a str,
    message: &'a str,
    span: Option<DiagnosticJsonSpan<'a>>,
    labels: Vec<DiagnosticJsonLabel<'a>>,
    notes: &'a [String],
    context: Vec<DiagnosticJsonContext<'a>>,
    suggested_fixes: Vec<DiagnosticJsonSuggestedFix<'a>>,
}

#[derive(Serialize)]
struct DiagnosticJsonLabel<'a> {
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
        format: "zen.diagnostics.v0",
        semantic_status: "diagnostic",
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
                context: diagnostic_json_context(&diagnostic.context, files),
                suggested_fixes: diagnostic_json_suggested_fixes(diagnostic, files),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&graph)
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

fn diagnostic_json_context<'a>(
    context: &'a [ContextFrame],
    files: &'a FileTable,
) -> Vec<DiagnosticJsonContext<'a>> {
    context
        .iter()
        .filter_map(|frame| {
            diagnostic_json_span(frame.span, files).map(|span| DiagnosticJsonContext {
                span,
                kind: frame.kind.as_str(),
                message: frame.message.as_str(),
            })
        })
        .collect()
}

fn diagnostic_json_suggested_fixes<'a>(
    diagnostic: &'a Diagnostic,
    files: &'a FileTable,
) -> Vec<DiagnosticJsonSuggestedFix<'a>> {
    diagnostic
        .suggested_fixes
        .iter()
        .filter_map(|fix| {
            let edits: Vec<_> = fix
                .edits
                .iter()
                .filter_map(|edit| {
                    diagnostic_json_span(edit.span, files).map(|span| DiagnosticJsonTextEdit {
                        span,
                        replacement: edit.replacement.as_str(),
                    })
                })
                .collect();

            if edits.is_empty() {
                None
            } else {
                Some(DiagnosticJsonSuggestedFix {
                    kind: fix.kind.as_str(),
                    title: fix.title.as_str(),
                    edits,
                })
            }
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
