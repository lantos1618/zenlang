use crate::ast::expressions::Expression;
use crate::error::Span;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Pattern {
    Wildcard {
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    Literal {
        value: Expression,
        span: Span,
    },

    Struct {
        name: String,
        fields: Vec<(String, Option<Pattern>)>,
        span: Span,
    },

    Enum {
        enum_name: String,
        variant: String,
        payload: Option<Box<Pattern>>,
        span: Span,
    },

    BoolTrue {
        span: Span,
    },

    BoolFalse {
        span: Span,
    },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span }
            | Pattern::Identifier { span, .. }
            | Pattern::Literal { span, .. }
            | Pattern::Struct { span, .. }
            | Pattern::Enum { span, .. }
            | Pattern::BoolTrue { span }
            | Pattern::BoolFalse { span } => *span,
        }
    }
}
