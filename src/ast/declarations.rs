use crate::ast::expressions::Expression;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;

mod support;

pub(crate) use support::type_param_names;
pub use support::{BehaviorMethod, EnumVariant, StructField, TypeDeclarationKeyword, TypeParam};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Declaration {
    Function {
        name: String,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Expression,
        public: bool,
        /// An `extern` C function: no Zen body (an empty block placeholder),
        /// not type-checked or emitted as a definition — codegen emits a C
        /// prototype and links the symbol from a `link:`-ed library.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        external: bool,
        span: Span,
    },

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

    Struct {
        name: String,
        type_params: Vec<TypeParam>,
        fields: Vec<StructField>,
        public: bool,
        span: Span,
    },

    Enum {
        name: String,
        type_params: Vec<TypeParam>,
        variants: Vec<EnumVariant>,
        public: bool,
        span: Span,
    },

    Import {
        names: Vec<String>,
        module_path: Vec<String>,
        span: Span,
    },

    Behavior {
        name: String,
        type_params: Vec<TypeParam>,
        methods: Vec<BehaviorMethod>,
        public: bool,
        span: Span,
    },

    ImplBlock {
        type_name: String,
        behavior: Option<String>,
        behavior_type_args: Vec<AstType>,
        type_args: Vec<AstType>,
        methods: Vec<Declaration>,
        span: Span,
    },

    Requires {
        type_name: String,
        behavior: String,
        behavior_type_args: Vec<AstType>,
        span: Span,
    },

    Derive {
        type_name: String,
        behavior: String,
        behavior_type_args: Vec<AstType>,
        span: Span,
    },

    BehaviorExtends {
        behavior: String,
        parent: String,
        parent_type_args: Vec<AstType>,
        span: Span,
    },

    TopLevelExpr {
        expr: Expression,
        span: Span,
    },

    /// `@export({ Name, other })` — the module's public surface. Everything is
    /// private by default; listed names are marked public by the resolver.
    Export {
        names: Vec<String>,
        span: Span,
    },

    /// `@extern Name` — an opaque C type (no Zen body/fields), used behind a
    /// pointer in FFI signatures. Codegen forward-declares it; the real
    /// definition comes from a `headers:` include.
    ExternType {
        name: String,
        span: Span,
    },
}

pub struct CallableDeclaration<'a> {
    pub name: &'a str,
    pub type_params: &'a [TypeParam],
    pub params: &'a [Param],
    pub return_type: &'a Option<AstType>,
    pub body: &'a Expression,
    pub public: bool,
    pub span: Span,
}

impl Declaration {
    pub fn as_callable(&self) -> Option<CallableDeclaration<'_>> {
        match self {
            Declaration::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                public,
                span,
                ..
            }
            | Declaration::Method {
                method_name: name,
                type_params,
                params,
                return_type,
                body,
                public,
                span,
                ..
            } => Some(CallableDeclaration {
                name,
                type_params,
                params,
                return_type,
                body,
                public: *public,
                span: *span,
            }),
            _ => None,
        }
    }

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
            | Declaration::Export { span, .. }
            | Declaration::ExternType { span, .. } => *span,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Declaration::Function { name, .. }
            | Declaration::Struct { name, .. }
            | Declaration::Enum { name, .. }
            | Declaration::Behavior { name, .. }
            | Declaration::ExternType { name, .. } => Some(name),
            Declaration::Method { method_name, .. } => Some(method_name),
            _ => None,
        }
    }

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
