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

impl TypeDeclarationKeyword {
    pub const ALL: &[TypeDeclarationKeyword] = &[
        TypeDeclarationKeyword::Impl,
        TypeDeclarationKeyword::Implements,
        TypeDeclarationKeyword::Requires,
        TypeDeclarationKeyword::Extends,
        TypeDeclarationKeyword::Derive,
    ];
    pub const IMPL: &'static str = "impl";
    pub const IMPLEMENTS: &'static str = "implements";
    pub const REQUIRES: &'static str = "requires";
    pub const EXTENDS: &'static str = "extends";
    pub const DERIVE: &'static str = "derive";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Impl => Self::IMPL,
            Self::Implements => Self::IMPLEMENTS,
            Self::Requires => Self::REQUIRES,
            Self::Extends => Self::EXTENDS,
            Self::Derive => Self::DERIVE,
        }
    }
}

crate::static_spelling::impl_static_spelling_display!(
    TypeDeclarationKeyword,
    as_str = TypeDeclarationKeyword::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    TypeDeclarationKeyword,
    variants = TypeDeclarationKeyword::ALL,
    as_str = TypeDeclarationKeyword::as_str
);

/// A field in a struct definition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructField {
    pub name: String,
    pub ty: AstType,
    pub default: Option<Expression>,
    pub mutable: bool,
    pub span: Span,
}

/// A variant in an enum definition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<AstType>,
    pub span: Span,
}

/// A method signature in a behavior (trait) definition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BehaviorMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub default_body: Option<Expression>,
    pub span: Span,
}

/// Generic type parameter, optionally constrained by a behavior.
/// e.g. `T` or `T: Serializable`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeParam {
    pub name: String,
    pub constraint: Option<String>,
    pub constraint_type_args: Vec<AstType>,
    pub span: Span,
}
