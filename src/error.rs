use std::fmt;

use serde::Serialize;

mod compiler_diagnostic_code;
mod diagnostic;
mod diagnostic_code;
mod diagnostic_enrichment;
mod diagnostic_metadata;
mod diagnostic_payload;
mod file_table;
mod protocol;
mod resolver_contract_code;
#[cfg(test)]
mod tests;

pub use compiler_diagnostic_code::CompilerDiagnosticCode;
pub use diagnostic::{
    ContextFrame, ContextKind, Diagnostic, Label, Severity, SuggestedFix, TextEdit,
};
pub use diagnostic_code::DiagnosticCode;
pub use diagnostic_metadata::{DiagnosticCategory, DiagnosticDescriptor, DiagnosticPhase};
pub use diagnostic_payload::{DiagnosticFact, RelatedDiagnostic};
pub use file_table::{FileId, FileTable};
pub use protocol::{
    code_actions_for_lsp, dap_launch_failure, diagnostics_for_ai, diagnostics_for_lsp,
    AgentDiagnostic, DapDiagnosticBundle, DapLaunchFailure, DapOutputBody, LspCodeAction,
    LspCodeActionData, LspCodeDescription, LspDiagnostic, LspDiagnosticData, LspDiagnosticRecord,
    LspRelatedInformation, LspTextEdit, ProtocolFact, ProtocolLocation, ProtocolPosition,
    ProtocolRange, ProtocolRelated, ProtocolSuggestedFix, ProtocolTextEdit,
};
pub use resolver_contract_code::ResolverContractCode;

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
                let diagnostic = Diagnostic::error_code_optional(DiagnosticCode::Syntax, msg, span);
                diagnostic
                    .with_removed_return_fix()
                    .with_removed_infix_as_cast_fix()
                    .with_generic_association_target_gate_context()
            }
            CompileError::Type(msg, span) => {
                Diagnostic::error_code_optional(DiagnosticCode::Type, msg, span)
            }
            CompileError::Resolution(msg, span) => {
                let code = if msg == GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE {
                    DiagnosticCode::Syntax
                } else {
                    DiagnosticCode::Resolution
                };
                Diagnostic::error_code_optional(code, msg, span)
                    .with_generated_behavior_derive_gate_context()
            }
            CompileError::Internal(msg) => {
                Diagnostic::error_code_optional(DiagnosticCode::Internal, msg, None)
            }
        }
    }
}
