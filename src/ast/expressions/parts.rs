use super::Expression;
use crate::ast::patterns::Pattern;
use crate::error::Span;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StringPart {
    Literal(String),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
    pub span: Span,
}
