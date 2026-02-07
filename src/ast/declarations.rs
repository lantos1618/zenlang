//! Declaration nodes in the AST

use std::collections::HashMap;

use super::expressions::Expression;
use super::fields::{
    function_arg_field, methods_field, protocol_methods_field, type_params_fields, AstFields,
    FieldValue,
};
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

impl AstFields for Declaration {
    fn ast_fields(&self) -> Vec<(&'static str, FieldValue)> {
        match self {
            Declaration::Function(f) => vec![
                ("name", FieldValue::String(f.name.clone())),
                ("type_params", type_params_fields(&f.type_params)),
                (
                    "args",
                    FieldValue::Array(
                        f.args
                            .iter()
                            .map(|(name, ty)| function_arg_field(name, ty))
                            .collect(),
                    ),
                ),
                ("return_type", FieldValue::ty(&f.return_type)),
                ("body", FieldValue::stmt_array(&f.body)),
                ("is_varargs", FieldValue::Bool(f.is_varargs)),
                ("is_public", FieldValue::Bool(f.is_public)),
            ],
            Declaration::ExternalFunction(ef) => vec![
                ("name", FieldValue::String(ef.name.clone())),
                ("args", FieldValue::type_array(&ef.args)),
                ("return_type", FieldValue::ty(&ef.return_type)),
                ("is_varargs", FieldValue::Bool(ef.is_varargs)),
            ],
            Declaration::Struct(s) => vec![
                ("name", FieldValue::String(s.name.clone())),
                ("type_params", type_params_fields(&s.type_params)),
                (
                    "fields",
                    FieldValue::Array(
                        s.fields
                            .iter()
                            .map(|f| {
                                let mut fields = HashMap::new();
                                fields
                                    .insert("name".to_string(), FieldValue::String(f.name.clone()));
                                fields.insert("field_type".to_string(), FieldValue::ty(&f.type_));
                                fields.insert(
                                    "is_mutable".to_string(),
                                    FieldValue::Bool(f.is_mutable),
                                );
                                fields.insert(
                                    "default_value".to_string(),
                                    match &f.default_value {
                                        Some(e) => FieldValue::expr(e),
                                        None => FieldValue::Null,
                                    },
                                );
                                FieldValue::Struct {
                                    name: "StructField".to_string(),
                                    fields,
                                }
                            })
                            .collect(),
                    ),
                ),
                ("methods", methods_field(&s.methods)),
            ],
            Declaration::Enum(e) => vec![
                ("name", FieldValue::String(e.name.clone())),
                ("type_params", type_params_fields(&e.type_params)),
                (
                    "variants",
                    FieldValue::Array(
                        e.variants
                            .iter()
                            .map(|v| {
                                let mut fields = HashMap::new();
                                fields
                                    .insert("name".to_string(), FieldValue::String(v.name.clone()));
                                fields.insert(
                                    "payload".to_string(),
                                    match &v.payload {
                                        Some(t) => FieldValue::ty(t),
                                        None => FieldValue::Null,
                                    },
                                );
                                FieldValue::Struct {
                                    name: "EnumVariant".to_string(),
                                    fields,
                                }
                            })
                            .collect(),
                    ),
                ),
                ("methods", methods_field(&e.methods)),
                (
                    "required_traits",
                    FieldValue::string_array(&e.required_traits),
                ),
            ],
            Declaration::Behavior(b) => vec![
                ("name", FieldValue::String(b.name.clone())),
                ("type_params", type_params_fields(&b.type_params)),
                (
                    "methods",
                    protocol_methods_field("BehaviorMethod", &b.methods),
                ),
            ],
            Declaration::Trait(t) => vec![
                ("name", FieldValue::String(t.name.clone())),
                ("type_params", type_params_fields(&t.type_params)),
                ("methods", protocol_methods_field("TraitMethod", &t.methods)),
            ],
            Declaration::TraitImplementation(ti) => vec![
                ("type_name", FieldValue::String(ti.type_name.clone())),
                ("trait_name", FieldValue::String(ti.trait_name.clone())),
                ("type_params", type_params_fields(&ti.type_params)),
                ("methods", methods_field(&ti.methods)),
            ],
            Declaration::TraitRequirement(tr) => vec![
                ("type_name", FieldValue::String(tr.type_name.clone())),
                ("trait_name", FieldValue::String(tr.trait_name.clone())),
            ],
            Declaration::ImplBlock(imp) => vec![
                ("type_name", FieldValue::String(imp.type_name.clone())),
                ("type_params", type_params_fields(&imp.type_params)),
                ("methods", methods_field(&imp.methods)),
            ],
            Declaration::ComptimeBlock(stmts) => {
                vec![("statements", FieldValue::stmt_array(stmts))]
            }
            Declaration::Constant {
                name, value, type_, ..
            } => vec![
                ("name", FieldValue::String(name.clone())),
                ("value", FieldValue::expr(value)),
                (
                    "const_type",
                    match type_ {
                        Some(t) => FieldValue::ty(t),
                        None => FieldValue::Null,
                    },
                ),
            ],
            Declaration::ModuleImport {
                alias, module_path, ..
            } => vec![
                ("alias", FieldValue::String(alias.clone())),
                ("module_path", FieldValue::String(module_path.clone())),
            ],
            Declaration::Export { symbols } => {
                vec![("symbols", FieldValue::string_array(symbols))]
            }
            Declaration::TypeAlias(ta) => vec![
                ("name", FieldValue::String(ta.name.clone())),
                ("type_params", type_params_fields(&ta.type_params)),
                ("target_type", FieldValue::ty(&ta.target_type)),
            ],
        }
    }
}
