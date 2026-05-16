//! Typechecker environment data shared across collection, checking, and monomorphization.

use std::collections::HashMap;

use crate::ast::{self, AstType, Expression, Param};
use crate::error::Span;

/// Information about a struct type.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, AstType)>,
    pub field_defaults: HashMap<String, Expression>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<(String, Option<AstType>)>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

/// Information about a function signature.
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
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
    pub name: String,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
    pub methods: Vec<ast::BehaviorMethod>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericFunctionTemplate {
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub body: Expression,
    pub span: Span,
    pub dependency_structs: HashMap<String, StructInfo>,
    pub dependency_enums: HashMap<String, EnumInfo>,
    pub dependency_functions: HashMap<String, FuncInfo>,
    pub dependency_generic_functions: HashMap<String, GenericFunctionTemplate>,
    pub dependency_methods: HashMap<String, FuncInfo>,
    pub dependency_generic_methods: HashMap<String, GenericFunctionTemplate>,
}

impl GenericFunctionTemplate {
    pub(crate) fn new(
        type_params: Vec<String>,
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Expression,
        span: Span,
    ) -> Self {
        Self {
            type_params,
            params,
            return_type,
            body,
            span,
            dependency_structs: HashMap::new(),
            dependency_enums: HashMap::new(),
            dependency_functions: HashMap::new(),
            dependency_generic_functions: HashMap::new(),
            dependency_methods: HashMap::new(),
            dependency_generic_methods: HashMap::new(),
        }
    }

    pub(crate) fn with_dependencies(
        mut self,
        dependency_structs: HashMap<String, StructInfo>,
        dependency_enums: HashMap<String, EnumInfo>,
        dependency_functions: HashMap<String, FuncInfo>,
        dependency_generic_functions: HashMap<String, GenericFunctionTemplate>,
        dependency_methods: HashMap<String, FuncInfo>,
        dependency_generic_methods: HashMap<String, GenericFunctionTemplate>,
    ) -> Self {
        self.dependency_structs = dependency_structs;
        self.dependency_enums = dependency_enums;
        self.dependency_functions = dependency_functions;
        self.dependency_generic_functions = dependency_generic_functions;
        self.dependency_methods = dependency_methods;
        self.dependency_generic_methods = dependency_generic_methods;
        self
    }

    pub(crate) fn with_source_dependencies(self, dependencies: SourceModuleDependencies) -> Self {
        self.with_dependencies(
            dependencies.structs,
            dependencies.enums,
            dependencies.functions,
            dependencies.generic_functions,
            dependencies.methods,
            dependencies.generic_methods,
        )
    }

    pub(crate) fn attach_source_dependencies(&mut self, dependencies: SourceModuleDependencies) {
        self.dependency_structs = dependencies.structs;
        self.dependency_enums = dependencies.enums;
        self.dependency_functions = dependencies.functions;
        self.dependency_generic_functions = dependencies.generic_functions;
        self.dependency_methods = dependencies.methods;
        self.dependency_generic_methods = dependencies.generic_methods;
    }
}

pub(crate) struct TemplateDependencyEntry<T> {
    pub(crate) name: String,
    pub(crate) previous: Option<T>,
}

pub(crate) type TemplateStructDependencyState = Vec<TemplateDependencyEntry<StructInfo>>;
pub(crate) type TemplateEnumDependencyState = Vec<TemplateDependencyEntry<EnumInfo>>;
pub(crate) type TemplateFunctionDependencyState = Vec<TemplateDependencyEntry<FuncInfo>>;
pub(crate) type TemplateGenericDependencyState =
    Vec<TemplateDependencyEntry<GenericFunctionTemplate>>;
pub(crate) type TemplateMethodDependencyState = Vec<TemplateDependencyEntry<FuncInfo>>;
pub(crate) type TemplateGenericMethodDependencyState =
    Vec<TemplateDependencyEntry<GenericFunctionTemplate>>;

pub(crate) struct TemplateDependencyState {
    pub(crate) structs: TemplateStructDependencyState,
    pub(crate) enums: TemplateEnumDependencyState,
    pub(crate) functions: TemplateFunctionDependencyState,
    pub(crate) generic_functions: TemplateGenericDependencyState,
    pub(crate) methods: TemplateMethodDependencyState,
    pub(crate) generic_methods: TemplateGenericMethodDependencyState,
}

#[derive(Clone, Default)]
pub(crate) struct SourceModuleDependencies {
    pub(crate) structs: HashMap<String, StructInfo>,
    pub(crate) enums: HashMap<String, EnumInfo>,
    pub(crate) functions: HashMap<String, FuncInfo>,
    pub(crate) generic_functions: HashMap<String, GenericFunctionTemplate>,
    pub(crate) methods: HashMap<String, FuncInfo>,
    pub(crate) generic_methods: HashMap<String, GenericFunctionTemplate>,
}

impl SourceModuleDependencies {
    pub(crate) fn apply_to_template(
        &self,
        template: GenericFunctionTemplate,
    ) -> GenericFunctionTemplate {
        template.with_source_dependencies(self.clone())
    }
}
