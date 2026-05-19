use std::fmt;

use serde::Serialize;

mod diagnostic_enrichment;
mod file_table;
#[cfg(test)]
mod tests;

pub use file_table::{FileId, FileTable};

// ── Span ───────────────────────────────────────────────────────────

/// Byte range within a specific file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub file_id: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file_id: FileId, start: u32, end: u32) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    /// A zero-width span at file 0 — useful for tests and synthetic nodes.
    pub fn dummy() -> Self {
        Self {
            file_id: 0,
            start: 0,
            end: 0,
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans (must be in the same file). Takes the min start and max end.
    pub fn merge(&self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id);
        Span {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

// ── Severity ───────────────────────────────────────────────────────

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

// ── Label ──────────────────────────────────────────────────────────

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

// ── Context Frames ────────────────────────────────────────────────

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

pub const REMOVED_RETURN_KEYWORD_MESSAGE: &str =
    "return keyword has been removed; use the final expression in the block";
pub const REMOVED_RETURN_FIX_KIND: &str = "replace_removed_return_with_final_expression";
pub const REMOVED_RETURN_FIX_TITLE: &str =
    "Remove `return` and use the value as the final expression";
pub const REMOVED_AS_CAST_MESSAGE: &str =
    "`as` cast syntax has been removed; use cast(value, Type)";
pub const REMOVED_INFIX_AS_CAST_MESSAGE: &str =
    "`as` cast syntax has been removed; use prefix cast(value, Type)";
pub const REMOVED_INFIX_AS_CAST_FIX_KIND: &str = "replace_infix_as_cast_with_prefix_cast";
pub const REMOVED_INFIX_AS_CAST_FIX_TITLE: &str =
    "Rewrite infix `as` cast to prefix `cast(value, Type)`";
pub const REMOVED_INFIX_AS_CAST_REPLACEMENT: &str = "cast(value, Type)";
pub const GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE: &str =
    "generated behavior association `Type.derive(...)` is gated until derive fallback resolution and ambiguity diagnostics are implemented";
pub const GATED_GENERATED_BEHAVIOR_DERIVE_NOTE: &str =
    "Use an explicit `Type.implements(Behavior) { ... }` block until generated fallback derives are implemented";
pub const GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT: &str =
    "reserved generated/fallback behavior association";
pub const GATED_GENERIC_ASSOCIATION_TARGET_MESSAGE_PREFIX: &str =
    "generic association target `Type<T>.";
pub const GATED_GENERIC_ASSOCIATION_TARGET_NOTE: &str =
    "Use a non-generic explicit behavior association until generic behavior target templates are implemented";
pub const GATED_GENERIC_ASSOCIATION_TARGET_CONTEXT: &str =
    "reserved generic behavior association target";

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

// ── Diagnostic ─────────────────────────────────────────────────────

/// A single compiler diagnostic — shared across all phases.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub context: Vec<ContextFrame>,
    pub suggested_fixes: Vec<SuggestedFix>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span: Some(span),
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
            suggested_fixes: Vec::new(),
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span: Some(span),
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
            suggested_fixes: Vec::new(),
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

// ── CompileError ───────────────────────────────────────────────────

/// Simple error enum for use in `Result<T, CompileError>`.
/// For richer reporting, convert to `Diagnostic`.
#[derive(Debug, Clone)]
pub enum CompileError {
    Syntax(String, Option<Span>),
    Type(String, Option<Span>),
    Resolution(String, Option<Span>),
    Internal(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Syntax(msg, _) => write!(f, "syntax error: {msg}"),
            CompileError::Type(msg, _) => write!(f, "type error: {msg}"),
            CompileError::Resolution(msg, _) => write!(f, "resolution error: {msg}"),
            CompileError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<CompileError> for Diagnostic {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::Syntax(msg, span) => {
                let diagnostic = Diagnostic {
                    severity: Severity::Error,
                    code: "E2000".to_string(),
                    message: msg,
                    span,
                    labels: Vec::new(),
                    notes: Vec::new(),
                    context: Vec::new(),
                    suggested_fixes: Vec::new(),
                };
                diagnostic
                    .with_removed_return_fix()
                    .with_removed_infix_as_cast_fix()
                    .with_generic_association_target_gate_context()
            }
            CompileError::Type(msg, span) => Diagnostic {
                severity: Severity::Error,
                code: "E3000".to_string(),
                message: msg,
                span,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
                suggested_fixes: Vec::new(),
            },
            CompileError::Resolution(msg, span) => {
                let code = if msg == GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE {
                    "E2000"
                } else {
                    "E3500"
                };
                Diagnostic {
                    severity: Severity::Error,
                    code: code.to_string(),
                    message: msg,
                    span,
                    labels: Vec::new(),
                    notes: Vec::new(),
                    context: Vec::new(),
                    suggested_fixes: Vec::new(),
                }
                .with_generated_behavior_derive_gate_context()
            }
            CompileError::Internal(msg) => Diagnostic {
                severity: Severity::Error,
                code: "E9999".to_string(),
                message: msg,
                span: None,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
                suggested_fixes: Vec::new(),
            },
        }
    }
}
