use crate::ast::expressions::Expression;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;

mod support;

pub use support::{BehaviorMethod, EnumVariant, StructField, TypeDeclarationKeyword, TypeParam};

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

    /// Generated/fallback behavior association: `Point.derive(Json)`
    Derive {
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
            | Declaration::Derive { span, .. }
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
