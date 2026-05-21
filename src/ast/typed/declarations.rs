use crate::error::Span;
use serde::Serialize;

use super::{Type, TypedBlock, TypedExpression};

// ─── Typed Declarations ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedFunction {
    pub name: std::string::String,
    pub params: Vec<TypedParam>,
    pub return_type: Type,
    pub body: TypedBlock,
    pub defers: Vec<TypedExpression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedParam {
    pub name: std::string::String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedTypeDef {
    pub name: std::string::String,
    pub kind: TypeDefKind,
    pub methods: Vec<TypedFunction>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypeDefKind {
    Struct {
        fields: Vec<(std::string::String, Type)>,
    },
    Enum {
        variants: Vec<TypedVariant>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedVariant {
    pub name: std::string::String,
    pub tag: u32,
    pub payload: Option<Vec<(std::string::String, Type)>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedGlobal {
    pub name: std::string::String,
    pub ty: Type,
    pub value: TypedExpression,
    pub mutable: bool,
    pub span: Span,
}

// ─── Top-Level Program ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
    pub types: Vec<TypedTypeDef>,
    pub globals: Vec<TypedGlobal>,
    pub entry_point: Option<std::string::String>,
}
