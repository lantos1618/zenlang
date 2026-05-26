mod agent;
mod dap;
mod lsp;

use serde::Serialize;

use super::{Diagnostic, FileTable, Span};

pub use agent::{diagnostics_for_ai, AgentDiagnostic};
pub use dap::{dap_launch_failure, DapDiagnosticBundle, DapLaunchFailure, DapOutputBody};
pub use lsp::{
    code_actions_for_lsp, diagnostics_for_lsp, LspCodeAction, LspCodeActionData,
    LspCodeDescription, LspDiagnostic, LspDiagnosticData, LspDiagnosticRecord,
    LspRelatedInformation, LspTextEdit,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolRange {
    pub start: ProtocolPosition,
    pub end: ProtocolPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolLocation {
    pub uri: String,
    pub path: String,
    pub range: ProtocolRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolFact {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolTextEdit {
    pub location: ProtocolLocation,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolSuggestedFix {
    pub kind: String,
    pub title: String,
    pub edits: Vec<ProtocolTextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolRelated {
    pub location: ProtocolLocation,
    pub message: String,
}

pub(super) fn protocol_facts(diagnostic: &Diagnostic) -> Vec<ProtocolFact> {
    diagnostic
        .facts
        .iter()
        .map(|fact| ProtocolFact {
            key: fact.key.clone(),
            value: fact.value.clone(),
        })
        .collect()
}

pub(super) fn protocol_fixes(
    diagnostic: &Diagnostic,
    files: &FileTable,
) -> Vec<ProtocolSuggestedFix> {
    diagnostic
        .suggested_fixes
        .iter()
        .map(|fix| ProtocolSuggestedFix {
            kind: fix.kind.clone(),
            title: fix.title.clone(),
            edits: fix
                .edits
                .iter()
                .filter_map(|edit| {
                    location_for_span(edit.span, files).map(|location| ProtocolTextEdit {
                        location,
                        replacement: edit.replacement.clone(),
                    })
                })
                .collect(),
        })
        .collect()
}

pub(super) fn location_for_span(span: Span, files: &FileTable) -> Option<ProtocolLocation> {
    let path = files.get_path(span.file_id)?;
    Some(ProtocolLocation {
        uri: file_uri(path),
        path: path.to_string(),
        range: range_for_span(span, files)?,
    })
}

fn range_for_span(span: Span, files: &FileTable) -> Option<ProtocolRange> {
    let (start_line, start_character) = files.line_col(span.file_id, span.start)?;
    let (end_line, end_character) = files.line_col(span.file_id, span.end)?;
    Some(ProtocolRange {
        start: ProtocolPosition {
            line: start_line,
            character: start_character,
        },
        end: ProtocolPosition {
            line: end_line,
            character: end_character,
        },
    })
}

fn file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        return path.to_string();
    }
    let prefix = if path.starts_with('/') {
        "file://"
    } else {
        "file:///"
    };
    format!("{prefix}{path}")
}
