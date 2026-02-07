//! Declaration nodes in the AST

use super::expressions::Expression;
use super::statements::Statement;
use super::types::{AstType, EnumVariant, TypeParameter};
use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub args: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub body: Vec<Statement>,
    pub is_varargs: bool, // For variadic functions like printf
    pub is_public: bool,  // true if marked with 'pub' keyword
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub target_type: AstType,
    pub span: Option<Span>,
}

// For C FFI support
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFunction {
    pub name: String,
    pub args: Vec<AstType>, // Just types, no names for external functions
    pub return_type: AstType,
    pub is_varargs: bool, // For functions like printf
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub fields: Vec<StructField>,
    pub methods: Vec<Function>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_: AstType,
    pub is_mutable: bool,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub methods: Vec<Function>,
    pub required_traits: Vec<String>, // Traits that all variants must implement (.requires())
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: AstType,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<BehaviorMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<TraitMethod>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitImplementation {
    pub type_name: String,
    pub trait_name: String,
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitRequirement {
    pub type_name: String,
    pub trait_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub type_name: String,
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Function(Function),
    ExternalFunction(ExternalFunction),
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Behavior(BehaviorDefinition),
    Trait(TraitDefinition),
    TraitImplementation(TraitImplementation),
    TraitRequirement(TraitRequirement),
    ImplBlock(ImplBlock),
    ComptimeBlock(Vec<Statement>),
    Constant {
        name: String,
        value: Expression,
        type_: Option<AstType>,
        span: Option<Span>,
    },
    ModuleImport {
        alias: String,
        module_path: String,
        span: Option<Span>,
    },
    Export {
        symbols: Vec<String>,
    },
    TypeAlias(TypeAlias),
}

impl Declaration {
    /// Returns the variant name of this declaration as a static string.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Declaration::Function(_) => "Function",
            Declaration::ExternalFunction(_) => "ExternalFunction",
            Declaration::Struct(_) => "Struct",
            Declaration::Enum(_) => "Enum",
            Declaration::Behavior(_) => "Behavior",
            Declaration::Trait(_) => "Trait",
            Declaration::TraitImplementation(_) => "TraitImplementation",
            Declaration::TraitRequirement(_) => "TraitRequirement",
            Declaration::ImplBlock(_) => "ImplBlock",
            Declaration::ComptimeBlock(_) => "ComptimeBlock",
            Declaration::Constant { .. } => "Constant",
            Declaration::ModuleImport { .. } => "ModuleImport",
            Declaration::Export { .. } => "Export",
            Declaration::TypeAlias(_) => "TypeAlias",
        }
    }
}
