use crate::ast::expressions::{BinaryOp, LoopControlAction, UnaryOp};
use crate::error::Span;
use serde::Serialize;

mod expression_parts;
mod types;

pub use expression_parts::{MatchKind, TypedMatchArm, TypedPattern, TypedStringPart};
pub use types::Type;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExpression {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedExprKind {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Variable(String),

    BinaryOp {
        op: BinaryOp,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<TypedExpression>,
    },

    FunctionCall {
        function: String,
        args: Vec<TypedExpression>,
    },

    /// A compiler intrinsic call (`@builtin.<name>(...)`) lowered directly by
    /// the C backend. `name` is the bare intrinsic spelling (e.g. `libc_write`).
    Intrinsic {
        name: String,
        args: Vec<TypedExpression>,
    },

    FieldAccess {
        object: Box<TypedExpression>,
        field: String,
    },

    IndexAccess {
        object: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },

    StructLiteral {
        type_name: String,
        fields: Vec<(String, TypedExpression)>,
    },

    EnumVariant {
        type_name: String,
        variant: String,
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

    StringInterpolation {
        parts: Vec<TypedStringPart>,
    },

    Assign {
        target: Box<TypedExpression>,
        value: Box<TypedExpression>,
    },

    Block(TypedBlock),

    LoopControl {
        action: LoopControlAction,
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedStatement {
    pub kind: TypedStatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStatementKind {
    VarDecl {
        name: String,
        ty: Type,
        value: TypedExpression,
        mutable: bool,
    },
    Expression(TypedExpression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub expr: Option<Box<TypedExpression>>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedFunction {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Type,
    pub body: TypedBlock,
    pub defers: Vec<TypedExpression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedParam {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedTypeDef {
    pub name: String,
    pub kind: TypeDefKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypeDefKind {
    Struct { fields: Vec<(String, Type)> },
    Enum { variants: Vec<TypedVariant> },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedVariant {
    pub name: String,
    pub tag: u32,
    pub payload: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedGlobal {
    pub name: String,
    pub ty: Type,
    pub value: TypedExpression,
    pub mutable: bool,
    pub span: Span,
}

/// An `extern` C function: a signature with no Zen body. Codegen emits a C
/// prototype; the symbol is resolved at link time from a `link:`-ed library.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExternFunction {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
    pub types: Vec<TypedTypeDef>,
    pub globals: Vec<TypedGlobal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extern_functions: Vec<TypedExternFunction>,
    /// Opaque `@extern` C type names — forward-declared in the generated C.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extern_types: Vec<String>,
    pub entry_point: Option<String>,
}
