use std::fmt;

use serde::Serialize;

mod diagnostic;
mod diagnostic_enrichment;
mod file_table;
#[cfg(test)]
mod tests;

pub use diagnostic::{
    ContextFrame, ContextKind, Diagnostic, Label, Severity, SuggestedFix, TextEdit,
};
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
pub const MISSING_BOOL_MATCH_ARM_FIX_KIND: &str = "add_missing_bool_match_arm";
pub const MISSING_BOOL_MATCH_ARM_FIX_TITLE: &str = "Add missing bool match arm";

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
