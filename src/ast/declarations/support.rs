use crate::ast::expressions::Expression;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TypeDeclarationKeyword {
    Impl,
    Implements,
    Requires,
    Extends,
    Derive,
}

const TYPE_DECLARATION_KEYWORD_SPELLINGS: &[(TypeDeclarationKeyword, &str)] = &[
    (TypeDeclarationKeyword::Impl, "impl"),
    (TypeDeclarationKeyword::Implements, "implements"),
    (TypeDeclarationKeyword::Requires, "requires"),
    (TypeDeclarationKeyword::Extends, "extends"),
    (TypeDeclarationKeyword::Derive, "derive"),
];

crate::static_spelling::impl_static_spelling_display!(
    TypeDeclarationKeyword,
    table = TYPE_DECLARATION_KEYWORD_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    TypeDeclarationKeyword,
    table = TYPE_DECLARATION_KEYWORD_SPELLINGS
);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructField {
    pub name: String,
    pub ty: AstType,
    pub default: Option<Expression>,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<AstType>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BehaviorMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub default_body: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeParam {
    pub name: String,
    pub constraint: Option<String>,
    pub constraint_type_args: Vec<AstType>,
    // Optional default type, used when a reference omits this (trailing)
    // argument: `Vec<T, Alloc: Allocator = Mallocator>` lets `Vec<T>` mean
    // `Vec<T, Mallocator>`.
    pub default: Option<AstType>,
    pub span: Span,
}

pub(crate) fn type_param_names(type_params: &[TypeParam]) -> Vec<String> {
    type_params.iter().map(|param| param.name.clone()).collect()
}
