mod agent;
mod dap;
mod lsp;

use serde::Serialize;

use super::{Diagnostic, DiagnosticFact, FileTable, Span};

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

pub type ProtocolFact = DiagnosticFact;

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

pub(super) fn protocol_fixes(
    diagnostic: &Diagnostic,
    files: &FileTable,
) -> Vec<ProtocolSuggestedFix> {
    diagnostic
        .suggested_fixes()
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

pub(super) fn protocol_related(diagnostic: &Diagnostic, files: &FileTable) -> Vec<ProtocolRelated> {
    diagnostic
        .related()
        .iter()
        .filter_map(|related| {
            location_for_span(related.span, files).map(|location| ProtocolRelated {
                location,
                message: related.message.clone(),
            })
        })
        .collect()
}

pub(super) fn location_for_span(span: Span, files: &FileTable) -> Option<ProtocolLocation> {
    let path = files.get_path(span.file_id)?;
    let (start_line, start_character) = files.line_col(span.file_id, span.start)?;
    let (end_line, end_character) = files.line_col(span.file_id, span.end)?;
    let uri_prefix = if path.starts_with("file://") {
        ""
    } else if path.starts_with('/') {
        "file://"
    } else {
        "file:///"
    };
    Some(ProtocolLocation {
        uri: format!("{uri_prefix}{path}"),
        path: path.to_string(),
        range: ProtocolRange {
            start: ProtocolPosition {
                line: start_line,
                character: start_character,
            },
            end: ProtocolPosition {
                line: end_line,
                character: end_character,
            },
        },
    })
}
