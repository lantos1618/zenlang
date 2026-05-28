use std::fmt;

use serde::Serialize;

mod compiler_diagnostic_code;
mod diagnostic;
mod diagnostic_metadata;
mod file_table;
mod protocol;

pub use compiler_diagnostic_code::CompilerDiagnosticCode;
pub use diagnostic::{
    ContextFrame, Diagnostic, DiagnosticFact, RelatedDiagnostic, SuggestedFix, TextEdit,
};
pub use diagnostic_metadata::{DiagnosticCategory, DiagnosticPhase};
pub use file_table::{FileId, FileTable};
pub use protocol::{
    code_actions_for_lsp, dap_launch_failure, diagnostics_for_ai, diagnostics_for_lsp,
    AgentDiagnostic, DapDiagnosticBundle, DapLaunchFailure, DapOutputBody, LspCodeAction,
    LspCodeActionData, LspCodeDescription, LspDiagnostic, LspDiagnosticData, LspDiagnosticRecord,
    LspRelatedInformation, LspTextEdit, ProtocolFact, ProtocolLocation, ProtocolPosition,
    ProtocolRange, ProtocolRelated, ProtocolSuggestedFix, ProtocolTextEdit,
};

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

    pub fn dummy() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn merge(&self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id);
        Self::new(
            self.file_id,
            self.start.min(other.start),
            self.end.max(other.end),
        )
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

#[derive(Debug, Clone)]
pub enum CompileError {
    Syntax(String, Option<Span>),
    Resolution(String, Option<Span>),
    Internal(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Syntax(msg, _) => write!(f, "syntax error: {msg}"),
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
                let diagnostic = Diagnostic::from_code(CompilerDiagnosticCode::E2000, msg, span);
                let diagnostic = if let Some(span) = diagnostic.span {
                    if diagnostic.message == REMOVED_RETURN_KEYWORD_MESSAGE {
                        diagnostic.with_fix(
                            REMOVED_RETURN_FIX_KIND,
                            REMOVED_RETURN_FIX_TITLE,
                            span,
                            "",
                        )
                    } else if diagnostic.message == REMOVED_INFIX_AS_CAST_MESSAGE {
                        diagnostic.with_fix(
                            REMOVED_INFIX_AS_CAST_FIX_KIND,
                            REMOVED_INFIX_AS_CAST_FIX_TITLE,
                            span,
                            REMOVED_INFIX_AS_CAST_REPLACEMENT,
                        )
                    } else {
                        diagnostic
                    }
                } else {
                    diagnostic
                };
                if diagnostic
                    .message
                    .starts_with(GATED_GENERIC_ASSOCIATION_TARGET_MESSAGE_PREFIX)
                {
                    diagnostic.with_feature_gate_context(
                        GATED_GENERIC_ASSOCIATION_TARGET_NOTE,
                        GATED_GENERIC_ASSOCIATION_TARGET_CONTEXT,
                    )
                } else {
                    diagnostic
                }
            }
            CompileError::Resolution(msg, span) => {
                let code = if msg == GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE {
                    CompilerDiagnosticCode::E2000
                } else {
                    CompilerDiagnosticCode::E3500
                };
                let diagnostic = Diagnostic::from_code(code, msg, span);
                if diagnostic.message == GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE {
                    diagnostic.with_feature_gate_context(
                        GATED_GENERATED_BEHAVIOR_DERIVE_NOTE,
                        GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT,
                    )
                } else {
                    diagnostic
                }
            }
            CompileError::Internal(msg) => {
                Diagnostic::from_code(CompilerDiagnosticCode::E9999, msg, None)
            }
        }
    }
}
