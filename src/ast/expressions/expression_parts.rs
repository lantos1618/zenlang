use super::Expression;
use crate::ast::patterns::Pattern;
use crate::error::Span;
use serde::Serialize;

/// Parts of a string interpolation: `"Hello, ${name}!"` becomes
/// `[Literal("Hello, "), Expr(<name>), Literal("!")]`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StringPart {
    Literal(String),
    Expr(Expression),
}

/// A match arm: `| pattern guard? { body }`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
    pub span: Span,
}
