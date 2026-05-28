use serde::Serialize;

use super::{location_for_span, protocol_related, ProtocolFact, ProtocolRange, ProtocolRelated};
use crate::error::{Diagnostic, FileTable};

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

pub type LspRelatedInformation = ProtocolRelated;

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
            diagnostic.suggested_fixes().iter().filter_map(move |fix| {
                let edits = fix
                    .edits
                    .iter()
                    .filter_map(|edit| {
                        let location = location_for_span(edit.span, files)?;
                        Some(LspTextEdit {
                            uri: location.uri,
                            range: location.range,
                            new_text: edit.replacement.clone(),
                        })
                    })
                    .collect::<Vec<_>>();
                (!edits.is_empty()).then(|| LspCodeAction {
                    title: fix.title.clone(),
                    kind: "quickfix".to_string(),
                    diagnostics: vec![record.diagnostic.clone()],
                    edits,
                    data: LspCodeActionData {
                        slug: diagnostic.slug(),
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
            severity: 1,
            code: diagnostic.code(),
            source: "zen".to_string(),
            message: diagnostic.message.clone(),
            code_description: Some(LspCodeDescription {
                href: diagnostic.docs_path().to_string(),
            }),
            related_information: protocol_related(diagnostic, files),
            data: LspDiagnosticData {
                slug: diagnostic.slug(),
                phase: diagnostic.phase().as_str().to_string(),
                category: diagnostic.category().as_str().to_string(),
                docs_path: diagnostic.docs_path().to_string(),
                facts: diagnostic.facts().to_vec(),
            },
        },
    })
}
