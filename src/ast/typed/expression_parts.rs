use super::{Type, TypedBlock, TypedExpression};
use crate::error::Span;
use serde::Serialize;

/// Sema resolves which kind of control flow `?` represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MatchKind {
    /// `expr ? | true { } | false { }` -> if/else
    ConditionalElse,
    /// `expr ? | true { }` -> one-shot conditional
    Conditional,
    /// `expr ? { body }` -> while loop
    WhileLoop,
    /// `loop((l) { ... })` with generated `l.done()` / `l.next()` labels.
    ControlledLoop { label: std::string::String },
    /// `enum ? | Variant {} ...` -> switch on tag
    EnumMatch,
    /// `val ? | X {} | Y {}` -> if/else chain on values
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
pub struct Capture {
    pub name: std::string::String,
    pub ty: Type,
    pub by_ref: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStringPart {
    Literal(std::string::String),
    Expr(TypedExpression),
}
