use crate::ast::expressions::Expression;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TypeDeclarationKeyword {
    Impl,
    Implements,
    Requires,
    Extends,
}

impl TypeDeclarationKeyword {
    pub const ALL: &[TypeDeclarationKeyword] = &[
        TypeDeclarationKeyword::Impl,
        TypeDeclarationKeyword::Implements,
        TypeDeclarationKeyword::Requires,
        TypeDeclarationKeyword::Extends,
    ];
    pub const IMPL: &'static str = "impl";
    pub const IMPLEMENTS: &'static str = "implements";
    pub const REQUIRES: &'static str = "requires";
    pub const EXTENDS: &'static str = "extends";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Impl => Self::IMPL,
            Self::Implements => Self::IMPLEMENTS,
            Self::Requires => Self::REQUIRES,
            Self::Extends => Self::EXTENDS,
        }
    }
}

impl fmt::Display for TypeDeclarationKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TypeDeclarationKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}

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

/// Declaration — top-level constructs in a Zen program.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Declaration {
    /// Named function: `add = (a: i32, b: i32) i32 { ... }`
    /// Generic function: `identity<T> = (value: T) T { ... }`
    Function {
        name: String,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Expression,
        public: bool,
        span: Span,
    },

    /// Method: `Point.distance = (self: Ptr<Point>, other: Ptr<Point>) f64 { ... }`
    Method {
        type_name: String,
        method_name: String,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Expression,
        public: bool,
        span: Span,
    },

    /// Struct definition: `Point: { x: f64, y: f64 }`
    Struct {
        name: String,
        type_params: Vec<TypeParam>,
        fields: Vec<StructField>,
        public: bool,
        span: Span,
    },

    /// Enum definition: `Color: Red, Green, Blue`
    Enum {
        name: String,
        type_params: Vec<TypeParam>,
        variants: Vec<EnumVariant>,
        public: bool,
        span: Span,
    },

    /// Import: `{ io } = std`, `{ Channel } = std.sync.channel`
    Import {
        names: Vec<String>,
        module_path: Vec<String>,
        span: Span,
    },

    /// Behavior (trait) definition: `Serializable: behavior { ... }`
    Behavior {
        name: String,
        type_params: Vec<TypeParam>,
        methods: Vec<BehaviorMethod>,
        public: bool,
        span: Span,
    },

    /// Impl block: `Point.impl = { ... }` or `Collector.implements(ActorBehavior, { ... })`
    ImplBlock {
        type_name: String,
        behavior: Option<String>,
        behavior_type_args: Vec<AstType>,
        type_args: Vec<AstType>,
        methods: Vec<Declaration>,
        span: Span,
    },

    /// Compile-time behavior assertion: `SensorReading.requires(Serializable)`
    Requires {
        type_name: String,
        behavior: String,
        behavior_type_args: Vec<AstType>,
        span: Span,
    },

    /// Behavior inheritance: `PrettyPrint.extends(Serializable)`
    BehaviorExtends {
        behavior: String,
        parent: String,
        parent_type_args: Vec<AstType>,
        span: Span,
    },

    /// Top-level expression.
    TopLevelExpr { expr: Expression, span: Span },

    /// Error recovery placeholder — allows parser to continue after errors.
    Error { span: Span },
}

impl Declaration {
    /// Returns the span of this declaration.
    pub fn span(&self) -> Span {
        match self {
            Declaration::Function { span, .. }
            | Declaration::Method { span, .. }
            | Declaration::Struct { span, .. }
            | Declaration::Enum { span, .. }
            | Declaration::Import { span, .. }
            | Declaration::Behavior { span, .. }
            | Declaration::ImplBlock { span, .. }
            | Declaration::Requires { span, .. }
            | Declaration::BehaviorExtends { span, .. }
            | Declaration::TopLevelExpr { span, .. }
            | Declaration::Error { span, .. } => *span,
        }
    }

    /// Returns the name of this declaration, if it has one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Declaration::Function { name, .. }
            | Declaration::Struct { name, .. }
            | Declaration::Enum { name, .. }
            | Declaration::Behavior { name, .. } => Some(name),
            Declaration::Method { method_name, .. } => Some(method_name),
            _ => None,
        }
    }

    /// Whether this declaration is exported from its module.
    pub fn is_public(&self) -> bool {
        match self {
            Declaration::Function { public, .. }
            | Declaration::Method { public, .. }
            | Declaration::Struct { public, .. }
            | Declaration::Enum { public, .. }
            | Declaration::Behavior { public, .. } => *public,
            _ => false,
        }
    }
}
