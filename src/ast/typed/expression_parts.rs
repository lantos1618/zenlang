use super::{Type, TypedBlock, TypedExpression};
use crate::error::Span;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MatchKind {
    ConditionalElse,
    Conditional,
    WhileLoop,
    ControlledLoop { label: std::string::String },
    EnumMatch,
    ValueMatch,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedPattern {
    Bool(bool),
    EnumVariant {
        type_name: std::string::String,
        variant: std::string::String,
        bindings: Vec<(std::string::String, Type)>,
    },
    Wildcard,
    Value(Box<TypedExpression>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStringPart {
    Literal(std::string::String),
    Expr(TypedExpression),
}
