use serde::Serialize;

use super::{
    location_for_span, protocol_fixes, protocol_related, ProtocolFact, ProtocolLocation,
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
        .map(|diagnostic| AgentDiagnostic {
            severity: "error".to_string(),
            code: diagnostic.code(),
            slug: diagnostic.slug(),
            phase: diagnostic.phase().as_str().to_string(),
            category: diagnostic.category().as_str().to_string(),
            docs_path: diagnostic.docs_path().to_string(),
            message: diagnostic.message.clone(),
            location: diagnostic
                .span
                .and_then(|span| location_for_span(span, files)),
            notes: diagnostic.notes().to_vec(),
            suggested_fixes: protocol_fixes(diagnostic, files),
            related: protocol_related(diagnostic, files),
            facts: diagnostic.facts().to_vec(),
        })
        .collect()
}
