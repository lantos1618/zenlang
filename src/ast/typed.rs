use crate::ast::expressions::{BinaryOp, LoopControlAction, UnaryOp};
use crate::error::Span;
use serde::Serialize;

mod declarations;
mod expression_parts;
mod types;

pub use declarations::{
    TypeDefKind, TypedFunction, TypedGlobal, TypedParam, TypedProgram, TypedTypeDef, TypedVariant,
};
pub use expression_parts::{Capture, MatchKind, TypedMatchArm, TypedPattern, TypedStringPart};
pub use types::Type;

// ─── Typed AST Nodes ─────────────────────────────────────────────────────────
// These mirror the untyped AST but every expression carries its resolved Type.
// We keep this minimal for now — it will be fleshed out when building the typechecker.

/// A typed expression: the expression kind + its resolved type + span.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExpression {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: Span,
}

/// Typed expression kinds — mirrors Expression but types are resolved.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedExprKind {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(std::string::String),
    BoolLiteral(bool),
    Variable(std::string::String),

    BinaryOp {
        op: BinaryOp,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<TypedExpression>,
    },

    /// All calls are resolved to concrete (mangled) function names.
    /// No generics, no method lookup — `p.distance(other)` is already
    /// `FunctionCall { function: "Point_distance", args: [p, other] }`.
    FunctionCall {
        function: std::string::String,
        args: Vec<TypedExpression>,
    },

    FieldAccess {
        object: Box<TypedExpression>,
        field: std::string::String,
    },

    IndexAccess {
        object: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },

    StructLiteral {
        type_name: std::string::String,
        fields: Vec<(std::string::String, TypedExpression)>,
    },

    EnumVariant {
        type_name: std::string::String,
        variant: std::string::String,
        payload: Option<Box<TypedExpression>>,
    },

    ArrayLiteral {
        elements: Vec<TypedExpression>,
    },

    Match {
        scrutinee: Box<TypedExpression>,
        arms: Vec<TypedMatchArm>,
        kind: MatchKind,
    },

    Cast {
        expr: Box<TypedExpression>,
        from_type: Type,
        to_type: Type,
    },

    Ref(Box<TypedExpression>),
    MutRef(Box<TypedExpression>),
    Deref(Box<TypedExpression>),

    Closure {
        fn_name: std::string::String,
        env_type: std::string::String,
        captures: Vec<Capture>,
    },

    StringInterpolation {
        parts: Vec<TypedStringPart>,
    },

    Intrinsic {
        name: std::string::String,
        args: Vec<TypedExpression>,
    },

    Assign {
        target: Box<TypedExpression>,
        value: Box<TypedExpression>,
    },

    Block(TypedBlock),

    Break,
    Continue,
    LoopControl {
        action: LoopControlAction,
        label: std::string::String,
    },

    Error,
}

// ─── Typed Statements ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedStatement {
    pub kind: TypedStatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStatementKind {
    VarDecl {
        name: std::string::String,
        ty: Type,
        value: TypedExpression,
        mutable: bool,
    },
    Expression(TypedExpression),
}

// ─── Typed Blocks ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub expr: Option<Box<TypedExpression>>,
    pub ty: Type,
    pub span: Span,
}
