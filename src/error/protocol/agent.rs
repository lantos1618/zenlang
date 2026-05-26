use serde::Serialize;

use super::{
    location_for_span, protocol_facts, protocol_fixes, ProtocolFact, ProtocolLocation,
    ProtocolRelated,
};
use crate::error::{Diagnostic, FileTable};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentDiagnostic {
    pub severity: String,
    pub code: String,
    pub slug: String,
    pub phase: String,
    pub category: String,
    pub docs_path: String,
    pub message: String,
    pub location: Option<ProtocolLocation>,
    pub notes: Vec<String>,
    pub suggested_fixes: Vec<super::ProtocolSuggestedFix>,
    pub related: Vec<ProtocolRelated>,
    pub facts: Vec<ProtocolFact>,
}

pub fn diagnostics_for_ai(diagnostics: &[Diagnostic], files: &FileTable) -> Vec<AgentDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| agent_diagnostic(diagnostic, files))
        .collect()
}

fn agent_diagnostic(diagnostic: &Diagnostic, files: &FileTable) -> AgentDiagnostic {
    AgentDiagnostic {
        severity: diagnostic.severity.to_string(),
        code: diagnostic.code.clone(),
        slug: diagnostic.slug.clone(),
        phase: diagnostic.phase.as_str().to_string(),
        category: diagnostic.category.as_str().to_string(),
        docs_path: diagnostic.docs_path.clone(),
        message: diagnostic.message.clone(),
        location: diagnostic
            .span
            .and_then(|span| location_for_span(span, files)),
        notes: diagnostic.notes.clone(),
        suggested_fixes: protocol_fixes(diagnostic, files),
        related: diagnostic
            .related
            .iter()
            .filter_map(|related| {
                location_for_span(related.span, files).map(|location| ProtocolRelated {
                    location,
                    message: related.message.clone(),
                })
            })
            .collect(),
        facts: protocol_facts(diagnostic),
    }
}
