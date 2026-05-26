use serde::Serialize;

use super::{location_for_span, protocol_facts, ProtocolFact, ProtocolLocation, ProtocolRange};
use crate::error::{Diagnostic, FileTable, Severity, Span};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnosticRecord {
    pub uri: String,
    pub diagnostic: LspDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnostic {
    pub range: ProtocolRange,
    pub severity: u8,
    pub code: String,
    pub source: String,
    pub message: String,
    pub code_description: Option<LspCodeDescription>,
    pub related_information: Vec<LspRelatedInformation>,
    pub data: LspDiagnosticData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LspCodeDescription {
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LspRelatedInformation {
    pub location: ProtocolLocation,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LspDiagnosticData {
    pub slug: String,
    pub phase: String,
    pub category: String,
    pub docs_path: String,
    pub facts: Vec<ProtocolFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCodeAction {
    pub title: String,
    pub kind: String,
    pub diagnostics: Vec<LspDiagnostic>,
    pub edits: Vec<LspTextEdit>,
    pub data: LspCodeActionData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspTextEdit {
    pub uri: String,
    pub range: ProtocolRange,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LspCodeActionData {
    pub slug: String,
    pub fix_kind: String,
}

pub fn diagnostics_for_lsp(
    diagnostics: &[Diagnostic],
    files: &FileTable,
) -> Vec<LspDiagnosticRecord> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| lsp_diagnostic_record(diagnostic, files))
        .collect()
}

pub fn code_actions_for_lsp(diagnostics: &[Diagnostic], files: &FileTable) -> Vec<LspCodeAction> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| {
            lsp_diagnostic_record(diagnostic, files).map(|record| (diagnostic, record))
        })
        .flat_map(|(diagnostic, record)| {
            diagnostic.suggested_fixes.iter().filter_map(move |fix| {
                let edits = fix
                    .edits
                    .iter()
                    .filter_map(|edit| lsp_text_edit(edit.span, edit.replacement.as_str(), files))
                    .collect::<Vec<_>>();
                (!edits.is_empty()).then(|| LspCodeAction {
                    title: fix.title.clone(),
                    kind: "quickfix".to_string(),
                    diagnostics: vec![record.diagnostic.clone()],
                    edits,
                    data: LspCodeActionData {
                        slug: diagnostic.slug.clone(),
                        fix_kind: fix.kind.clone(),
                    },
                })
            })
        })
        .collect()
}

fn lsp_diagnostic_record(
    diagnostic: &Diagnostic,
    files: &FileTable,
) -> Option<LspDiagnosticRecord> {
    let location = location_for_span(diagnostic.span?, files)?;
    Some(LspDiagnosticRecord {
        uri: location.uri,
        diagnostic: LspDiagnostic {
            range: location.range,
            severity: lsp_severity(diagnostic.severity),
            code: diagnostic.code.clone(),
            source: "zen".to_string(),
            message: diagnostic.message.clone(),
            code_description: (!diagnostic.docs_path.is_empty()).then(|| LspCodeDescription {
                href: diagnostic.docs_path.clone(),
            }),
            related_information: diagnostic
                .related
                .iter()
                .filter_map(|related| {
                    location_for_span(related.span, files).map(|location| LspRelatedInformation {
                        location,
                        message: related.message.clone(),
                    })
                })
                .collect(),
            data: LspDiagnosticData {
                slug: diagnostic.slug.clone(),
                phase: diagnostic.phase.as_str().to_string(),
                category: diagnostic.category.as_str().to_string(),
                docs_path: diagnostic.docs_path.clone(),
                facts: protocol_facts(diagnostic),
            },
        },
    })
}

fn lsp_text_edit(span: Span, replacement: &str, files: &FileTable) -> Option<LspTextEdit> {
    let location = location_for_span(span, files)?;
    Some(LspTextEdit {
        uri: location.uri,
        range: location.range,
        new_text: replacement.to_string(),
    })
}

fn lsp_severity(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 1,
        Severity::Warning => 2,
        Severity::Info => 3,
        Severity::Hint => 4,
    }
}
