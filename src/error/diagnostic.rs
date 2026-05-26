use std::fmt;

use super::diagnostic_payload::{raw_slug, DiagnosticFact, RelatedDiagnostic};
use super::{DiagnosticCategory, DiagnosticCode, DiagnosticPhase, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Hint => write!(f, "hint"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// A secondary annotation pointing at a span with a message.
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

impl Label {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

/// What kind of context led to this diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextKind {
    FeatureGate,
    InFunction,
    InModule,
    InGenericInstantiation,
    InTraitImpl,
    InImport,
    InMacroExpansion,
}

impl ContextKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FeatureGate => "feature_gate",
            Self::InFunction => "in_function",
            Self::InModule => "in_module",
            Self::InGenericInstantiation => "in_generic_instantiation",
            Self::InTraitImpl => "in_trait_impl",
            Self::InImport => "in_import",
            Self::InMacroExpansion => "in_macro_expansion",
        }
    }
}

/// A frame in the context stack showing how we reached the error.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextFrame {
    pub span: Span,
    pub kind: ContextKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub span: Span,
    pub replacement: String,
}

impl TextEdit {
    pub fn new(span: Span, replacement: impl Into<String>) -> Self {
        Self {
            span,
            replacement: replacement.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SuggestedFix {
    pub kind: String,
    pub title: String,
    pub edits: Vec<TextEdit>,
}

impl SuggestedFix {
    pub fn new(kind: impl Into<String>, title: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            edits,
        }
    }
}

/// A single compiler diagnostic shared across all phases.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub slug: String,
    pub phase: DiagnosticPhase,
    pub category: DiagnosticCategory,
    pub docs_path: String,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub context: Vec<ContextFrame>,
    pub suggested_fixes: Vec<SuggestedFix>,
    pub related: Vec<RelatedDiagnostic>,
    pub facts: Vec<DiagnosticFact>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self::raw(Severity::Error, code.into(), message.into(), Some(span))
    }

    pub fn error_optional(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self::raw(Severity::Error, code.into(), message.into(), span)
    }

    pub fn error_code(
        code: impl Into<DiagnosticCode>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::from_code(code.into(), message.into(), Some(span))
    }

    pub fn error_code_optional(
        code: impl Into<DiagnosticCode>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self::from_code(code.into(), message.into(), span)
    }

    fn from_code(code: DiagnosticCode, message: String, span: Option<Span>) -> Self {
        let descriptor = code.descriptor();
        Self {
            severity: descriptor.severity,
            code: descriptor.number,
            slug: descriptor.slug,
            phase: descriptor.phase,
            category: descriptor.category,
            docs_path: descriptor.docs_path,
            message,
            span,
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
            suggested_fixes: Vec::new(),
            related: Vec::new(),
            facts: Vec::new(),
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self::raw(Severity::Warning, code.into(), message.into(), Some(span))
    }

    fn raw(severity: Severity, code: String, message: String, span: Option<Span>) -> Self {
        Self {
            severity,
            slug: raw_slug(&code),
            phase: DiagnosticPhase::Unknown,
            category: DiagnosticCategory::Unknown,
            docs_path: String::new(),
            code,
            message,
            span,
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
            suggested_fixes: Vec::new(),
            related: Vec::new(),
            facts: Vec::new(),
        }
    }

    /// Add a secondary label.
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::new(span, message));
        self
    }

    /// Add a help/note string.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a context frame showing how we reached this diagnostic.
    pub fn with_context(mut self, frame: ContextFrame) -> Self {
        self.context.push(frame);
        self
    }

    pub fn with_suggested_fix(mut self, fix: SuggestedFix) -> Self {
        self.suggested_fixes.push(fix);
        self
    }

    pub fn with_related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push(RelatedDiagnostic::new(span, message));
        self
    }

    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.facts.push(DiagnosticFact::new(key, value));
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]: {}", self.severity, self.code, self.message)?;
        for frame in &self.context {
            write!(f, "\n   = {}", frame.message)?;
        }
        Ok(())
    }
}
