use crate::ast::expressions::Expression;
use crate::ast::types::AstType;
use crate::error::Span;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Statement {
    VarDecl {
        name: String,
        ty: Option<AstType>,
        value: Expression,
        mutable: bool,
        constant: bool,
        span: Span,
    },

    Assignment {
        target: Expression,
        value: Expression,
        span: Span,
    },

    Expression {
        expr: Expression,
        span: Span,
    },
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::VarDecl { span, .. }
            | Statement::Assignment { span, .. }
            | Statement::Expression { span, .. } => *span,
        }
    }
}
