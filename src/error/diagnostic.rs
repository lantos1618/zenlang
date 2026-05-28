use std::fmt;

use super::{CompilerDiagnosticCode, DiagnosticCategory, DiagnosticPhase, Span};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticFact {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextFrame {
    pub span: Span,
    pub kind: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct SuggestedFix {
    pub kind: String,
    pub title: String,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: CompilerDiagnosticCode,
    pub message: String,
    pub span: Option<Span>,
    payload: Box<DiagnosticPayload>,
}

#[derive(Debug, Clone, Default)]
struct DiagnosticPayload {
    notes: Vec<String>,
    context: Vec<ContextFrame>,
    suggested_fixes: Vec<SuggestedFix>,
    related: Vec<RelatedDiagnostic>,
    facts: Vec<DiagnosticFact>,
}

impl Diagnostic {
    pub fn error_code(
        code: CompilerDiagnosticCode,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::from_code(code, message.into(), Some(span))
    }

    pub(super) fn from_code(
        code: CompilerDiagnosticCode,
        message: String,
        span: Option<Span>,
    ) -> Self {
        Self {
            code,
            message,
            span,
            payload: Box::default(),
        }
    }

    pub fn code(&self) -> String {
        format!("E{:04}", self.code as u16)
    }

    pub fn slug(&self) -> String {
        self.code.slug()
    }

    pub fn phase(&self) -> DiagnosticPhase {
        self.code.phase()
    }

    pub fn category(&self) -> DiagnosticCategory {
        self.code.category()
    }

    pub fn docs_path(&self) -> &'static str {
        self.code.docs_path()
    }

    pub fn notes(&self) -> &[String] {
        &self.payload.notes
    }

    pub fn context(&self) -> &[ContextFrame] {
        &self.payload.context
    }

    pub fn suggested_fixes(&self) -> &[SuggestedFix] {
        &self.payload.suggested_fixes
    }

    pub fn related(&self) -> &[RelatedDiagnostic] {
        &self.payload.related
    }

    pub fn facts(&self) -> &[DiagnosticFact] {
        &self.payload.facts
    }

    pub fn with_feature_gate_context(
        mut self,
        note: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        let Some(span) = self.span else {
            return self;
        };

        self.payload.notes.push(note.into());
        self.payload.context.push(ContextFrame {
            span,
            kind: "feature_gate",
            message: context.into(),
        });
        self
    }

    pub fn with_fix(
        mut self,
        kind: impl Into<String>,
        title: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.payload.suggested_fixes.push(SuggestedFix {
            kind: kind.into(),
            title: title.into(),
            edits: vec![TextEdit {
                span,
                replacement: replacement.into(),
            }],
        });
        self
    }

    pub fn with_related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.payload.related.push(RelatedDiagnostic {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.payload.facts.push(DiagnosticFact {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}]: {}", self.code(), self.message)?;
        for frame in self.context() {
            write!(f, "\n   = {}", frame.message)?;
        }
        Ok(())
    }
}
