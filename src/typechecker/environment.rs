use std::collections::HashMap;

use crate::ast::{self, AstType, Expression, Param};
use crate::error::Span;

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub specialization_scope: Option<String>,
    pub fields: Vec<(String, AstType)>,
    pub field_defaults: HashMap<String, Expression>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
    // Default type for trailing type params, keyed by param name. Lets a
    // reference omit them: `Vec<i64>` fills `Alloc` from its default.
    pub type_param_defaults: HashMap<String, AstType>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub specialization_scope: Option<String>,
    pub variants: Vec<(String, Option<AstType>)>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
    pub type_param_defaults: HashMap<String, AstType>,
}

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorBound {
    pub behavior: String,
    pub type_args: Vec<AstType>,
}

#[derive(Debug, Clone)]
pub struct BehaviorInfo {
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
    pub methods: Vec<ast::BehaviorMethod>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericBehaviorImplTemplate {
    pub type_name: String,
    pub type_params: Vec<String>,
    pub behavior: String,
    pub behavior_type_args: Vec<AstType>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericFunctionTemplate {
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub body: Expression,
    pub span: Span,
    pub dependencies: SourceModuleDependencies,
}

pub(crate) struct TemplateDependencyState {
    pub(crate) structs: Vec<(String, Option<StructInfo>)>,
    pub(crate) enums: Vec<(String, Option<EnumInfo>)>,
    pub(crate) functions: Vec<(String, Option<FuncInfo>)>,
    pub(crate) generic_functions: Vec<(String, Option<GenericFunctionTemplate>)>,
    pub(crate) methods: Vec<(String, Option<FuncInfo>)>,
    pub(crate) generic_methods: Vec<(String, Option<GenericFunctionTemplate>)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SourceModuleDependencies {
    pub(crate) specialization_scope: Option<String>,
    pub(crate) structs: HashMap<String, StructInfo>,
    pub(crate) enums: HashMap<String, EnumInfo>,
    pub(crate) functions: HashMap<String, FuncInfo>,
    pub(crate) generic_functions: HashMap<String, GenericFunctionTemplate>,
    pub(crate) methods: HashMap<String, FuncInfo>,
    pub(crate) generic_methods: HashMap<String, GenericFunctionTemplate>,
}
