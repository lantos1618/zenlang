use crate::ast::expressions::Expression;
use crate::error::Span;
use serde::Serialize;

/// Pattern — used in match arms and destructuring.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Pattern {
    /// `_` — matches anything, binds nothing.
    Wildcard { span: Span },

    /// A name that binds the matched value: `x`, `value`
    Identifier { name: String, span: Span },

    /// A literal value: `42`, `"hello"`
    Literal { value: Expression, span: Span },

    /// Struct destructuring: `Point { x, y }` or `Warning { message }`
    Struct {
        name: String,
        fields: Vec<(String, Option<Pattern>)>,
        span: Span,
    },

    /// Enum variant: `Ok(value)`, `Some(v)`, `Normal`
    Enum {
        enum_name: String,
        variant: String,
        payload: Option<Box<Pattern>>,
        span: Span,
    },

    /// Or-pattern: `A | B | C`
    Or { patterns: Vec<Pattern>, span: Span },

    /// Range pattern: `1..5` or `1..=5`
    Range {
        start: Expression,
        end: Expression,
        inclusive: bool,
        span: Span,
    },

    /// `true`
    BoolTrue { span: Span },

    /// `false`
    BoolFalse { span: Span },
}

impl Pattern {
    /// Returns the span of this pattern.
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span }
            | Pattern::Identifier { span, .. }
            | Pattern::Literal { span, .. }
            | Pattern::Struct { span, .. }
            | Pattern::Enum { span, .. }
            | Pattern::Or { span, .. }
            | Pattern::Range { span, .. }
            | Pattern::BoolTrue { span }
            | Pattern::BoolFalse { span } => *span,
        }
    }
}
