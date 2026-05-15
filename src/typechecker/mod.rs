//! Typechecker — transforms untyped AST → TypedProgram.
//!
//! Pipeline:
//! 1. **Collect**: Register all struct/enum/function/behavior signatures
//! 2. **Resolve**: Resolve type references (Named("Foo") → Struct fields)
//! 3. **Check**: Type-check function bodies, produce TypedExpression
//!
//! The typechecker NEVER defaults unknown types to I32. If a type can't be
//! resolved, it's an error.

mod closures;
mod expressions;
mod monomorphize;
mod patterns;
mod resolve;
mod statements;

use std::collections::{HashMap, HashSet};

use crate::ast::typed::*;
use crate::ast::{self, AstType, Declaration, EnumVariant, Expression, Param, StructField};
use crate::error::{Diagnostic, Span};
use crate::module_system::{ResolvedModule, ResolvedModuleGraph};
use crate::resolver::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace,
    SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

// ── Type Environment ──────────────────────────────────────────────

/// Information about a struct type.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, AstType)>,
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
    fn new(
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

    fn with_dependencies(
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
}

pub(crate) type TemplateStructDependencyState = Vec<(String, Option<StructInfo>)>;
pub(crate) type TemplateEnumDependencyState = Vec<(String, Option<EnumInfo>)>;
pub(crate) type TemplateFunctionDependencyState = Vec<(String, Option<FuncInfo>)>;
pub(crate) type TemplateGenericDependencyState = Vec<(String, Option<GenericFunctionTemplate>)>;
pub(crate) type TemplateMethodDependencyState = Vec<(String, Option<FuncInfo>)>;
pub(crate) type TemplateGenericMethodDependencyState =
    Vec<(String, Option<GenericFunctionTemplate>)>;
pub(crate) type TemplateDependencyState = (
    TemplateStructDependencyState,
    TemplateEnumDependencyState,
    TemplateFunctionDependencyState,
    TemplateGenericDependencyState,
    TemplateMethodDependencyState,
    TemplateGenericMethodDependencyState,
);

struct DefaultBehaviorMethod {
    name: String,
    params: Vec<Param>,
    return_type: Option<AstType>,
    body: Expression,
    span: Span,
}

struct ExpectedValueSignature {
    parameter_names: Vec<String>,
    parameter_types: Vec<AstType>,
    parameter_type_names: Vec<String>,
    return_type: AstType,
    return_type_name: String,
    type_parameter_count: usize,
    type_parameter_names: Vec<String>,
    type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
    type_parameter_bound_refs: Vec<TypeParameterBoundRefMetadata>,
}

struct ExpectedTypeLikeSymbol {
    type_parameter_count: usize,
    type_parameter_names: Vec<String>,
    type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
    type_parameter_bound_refs: Vec<TypeParameterBoundRefMetadata>,
    is_public: Option<bool>,
}

struct ImportedMethodSignature<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
}

struct ImportedMethodDependencies<'a> {
    structs: &'a HashMap<String, StructInfo>,
    enums: &'a HashMap<String, EnumInfo>,
    functions: &'a HashMap<String, FuncInfo>,
    generic_functions: &'a HashMap<String, GenericFunctionTemplate>,
    methods: &'a HashMap<String, FuncInfo>,
    generic_methods: &'a HashMap<String, GenericFunctionTemplate>,
}

#[derive(Default)]
struct SourceModuleDependencies {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    generic_functions: HashMap<String, GenericFunctionTemplate>,
    methods: HashMap<String, FuncInfo>,
    generic_methods: HashMap<String, GenericFunctionTemplate>,
}

#[derive(Debug, Clone)]
struct BehaviorParentRef {
    behavior: String,
    type_args: Vec<AstType>,
    key: String,
}

#[derive(Default)]
struct ResolverScopeCursor {
    next_scope_id: u32,
}

impl ResolverScopeCursor {
    fn new_scope(&mut self) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::new(self.next_scope_id)
    }

    fn child_scope(&mut self, parent: &ResolverLocalScope) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::with_parent(self.next_scope_id, parent)
    }
}

#[derive(Clone)]
struct ResolverLocalScope {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ResolverLocalScope {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ResolverLocalScope) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}

/// Scope for variable types.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub ty: Type,
    pub mutable: bool,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

fn type_param_bounds(type_params: &[ast::TypeParam]) -> HashMap<String, BehaviorBound> {
    type_params
        .iter()
        .filter_map(|param| {
            param.constraint.as_ref().map(|bound| {
                (
                    param.name.clone(),
                    BehaviorBound {
                        behavior: bound.clone(),
                        type_args: param.constraint_type_args.clone(),
                    },
                )
            })
        })
        .collect()
}

fn type_param_bounds_from_resolver_refs(
    bounds: &[TypeParameterBoundRefMetadata],
) -> HashMap<String, BehaviorBound> {
    bounds
        .iter()
        .map(|bound| {
            (
                bound.type_parameter.clone(),
                BehaviorBound {
                    behavior: bound.behavior.clone(),
                    type_args: bound.type_args.clone(),
                },
            )
        })
        .collect()
}

fn type_param_bound_display(type_param: &ast::TypeParam) -> Option<String> {
    type_param.constraint.as_ref().map(|constraint| {
        if type_param.constraint_type_args.is_empty() {
            constraint.clone()
        } else {
            format!(
                "{}<{}>",
                constraint,
                type_param
                    .constraint_type_args
                    .iter()
                    .map(AstType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })
}

fn type_param_name_set(type_params: &[ast::TypeParam]) -> HashSet<String> {
    type_params.iter().map(|param| param.name.clone()).collect()
}

fn ast_type_references_type_param(
    ast_type: &AstType,
    scoped_type_params: &HashSet<String>,
) -> bool {
    match ast_type {
        AstType::Named(name) => scoped_type_params.contains(name),
        AstType::Generic { type_args, .. } => type_args
            .iter()
            .any(|arg| ast_type_references_type_param(arg, scoped_type_params)),
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => {
            ast_type_references_type_param(inner, scoped_type_params)
        }
        AstType::Function { params, ret } => {
            params
                .iter()
                .any(|param| ast_type_references_type_param(param, scoped_type_params))
                || ast_type_references_type_param(ret, scoped_type_params)
        }
        _ => false,
    }
}

fn collect_ast_type_names(ast_type: &AstType, names: &mut HashSet<String>) {
    match ast_type {
        AstType::Named(name) => {
            names.insert(name.clone());
        }
        AstType::Generic { name, type_args } => {
            names.insert(name.clone());
            for type_arg in type_args {
                collect_ast_type_names(type_arg, names);
            }
        }
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => collect_ast_type_names(inner, names),
        AstType::Function { params, ret } => {
            for param in params {
                collect_ast_type_names(param, names);
            }
            collect_ast_type_names(ret, names);
        }
        _ => {}
    }
}

fn concrete_self_ast_type(ast_type: &AstType, self_type_name: &str) -> AstType {
    match ast_type {
        AstType::SelfType => AstType::Named(self_type_name.to_string()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(concrete_self_ast_type(elem, self_type_name)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| concrete_self_ast_type(param, self_type_name))
                .collect(),
            ret: Box::new(concrete_self_ast_type(ret, self_type_name)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| concrete_self_ast_type(arg, self_type_name))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_behavior_ast_type(arg, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .map(monomorphize::type_to_ast)
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_bound_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_bound_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_bound_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: substitute_behavior_bound_type_args(type_args, substitutions),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_type_args(
    type_args: &[AstType],
    substitutions: &HashMap<String, Type>,
) -> Vec<AstType> {
    type_args
        .iter()
        .map(|arg| substitute_behavior_bound_ast_type(arg, substitutions))
        .collect()
}

fn behavior_bound_display(bound: &BehaviorBound, substitutions: &HashMap<String, Type>) -> String {
    let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
    if type_args.is_empty() {
        bound.behavior.clone()
    } else {
        format!(
            "{}<{}>",
            bound.behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!(
            "{}<{}>",
            behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn behavior_method_signatures_match(
    left: &ast::BehaviorMethod,
    right: &ast::BehaviorMethod,
) -> bool {
    left.return_type == right.return_type
        && left.params.len() == right.params.len()
        && left
            .params
            .iter()
            .zip(&right.params)
            .all(|(left, right)| left.mutable == right.mutable && left.ty == right.ty)
}

// ── TypeChecker ───────────────────────────────────────────────────

pub struct TypeChecker {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    methods: HashMap<String, FuncInfo>, // key: "TypeName.method_name"
    behaviors: HashMap<String, BehaviorInfo>,
    behavior_extends: HashMap<String, Vec<BehaviorParentRef>>,
    behavior_extends_spans: HashMap<String, Span>,
    behavior_impls: HashSet<(String, String)>,
    generic_functions: HashMap<String, GenericFunctionTemplate>,
    generic_methods: HashMap<String, GenericFunctionTemplate>,
    specialized_functions: Vec<TypedFunction>,
    specializations_seen: HashSet<String>,
    specialized_types: Vec<TypedTypeDef>,
    specialized_types_seen: HashSet<String>,
    type_substitutions: Vec<HashMap<String, Type>>,
    imports: HashMap<String, Vec<String>>, // imported name -> source module path
    scopes: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Type>,
    current_self_type: Option<Type>,
    pending_defers: Vec<TypedExpression>,
    resolver_backed_collection: bool,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
            behaviors: HashMap::new(),
            behavior_extends: HashMap::new(),
            behavior_extends_spans: HashMap::new(),
            behavior_impls: HashSet::new(),
            generic_functions: HashMap::new(),
            generic_methods: HashMap::new(),
            specialized_functions: Vec::new(),
            specializations_seen: HashSet::new(),
            specialized_types: Vec::new(),
            specialized_types_seen: HashSet::new(),
            type_substitutions: Vec::new(),
            imports: HashMap::new(),
            scopes: vec![Scope::new()], // global scope
            diagnostics: Vec::new(),
            current_return_type: None,
            current_self_type: None,
            pending_defers: Vec::new(),
            resolver_backed_collection: false,
        }
    }

    /// Type-check a program and produce a TypedProgram.
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);
        self.validate_collected_declaration_semantics(&program.declarations);
        self.check_program_after_collection(program)
    }

    fn check_program_after_collection(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 2: Check function bodies and produce typed AST
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut globals = Vec::new();
        let mut entry_point = None;

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    if name == "main" {
                        entry_point = Some(name.clone());
                    }
                    match self.check_function(name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    let full_name = format!("{}.{}", type_name, method_name);
                    // Set Self type for method body
                    self.current_self_type =
                        Some(self.resolve_type(&AstType::Named(type_name.clone())));
                    match self.check_function(&full_name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                    self.current_self_type = None;
                }
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    let resolved_fields: Vec<(String, Type)> = fields
                        .iter()
                        .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.clone(),
                        kind: TypeDefKind::Struct {
                            fields: resolved_fields,
                        },
                        methods: Vec::new(),
                        span: *span,
                    });
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    let typed_variants: Vec<TypedVariant> = variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| TypedVariant {
                            name: v.name.clone(),
                            tag: i as u32,
                            payload: v
                                .payload
                                .as_ref()
                                .map(|ty| vec![("payload".to_string(), self.resolve_type(ty))]),
                        })
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.clone(),
                        kind: TypeDefKind::Enum {
                            variants: typed_variants,
                        },
                        methods: Vec::new(),
                        span: *span,
                    });
                }
                Declaration::TopLevelExpr { expr, span } => {
                    // Top-level expressions like main() call
                    match self.check_expr(expr) {
                        Ok(typed_expr) => {
                            globals.push(TypedGlobal {
                                name: "__top_level__".into(),
                                ty: typed_expr.ty.clone(),
                                value: typed_expr,
                                mutable: false,
                                span: *span,
                            });
                        }
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Import { .. } => {
                    // Imports are handled by the module system, not the typechecker
                }
                Declaration::Behavior { .. } => {}
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            if !type_params.is_empty() {
                                continue;
                            }
                            let full_name = format!("{}.{}", type_name, name);
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(&full_name, params, return_type, body, span) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }

                    if let Some(behavior) = behavior {
                        for default in self.behavior_default_methods_for_impl(
                            type_name,
                            behavior,
                            behavior_type_args,
                            methods,
                        ) {
                            let full_name = format!("{}.{}", type_name, default.name);
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(
                                &full_name,
                                &default.params,
                                &default.return_type,
                                &default.body,
                                &default.span,
                            ) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }
                }
                _ => {}
            }
        }

        let errors: Vec<_> = self
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .cloned()
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }

        functions.append(&mut self.specialized_functions);
        types.append(&mut self.specialized_types);

        Ok(TypedProgram {
            functions,
            types,
            globals,
            entry_point,
        })
    }

    pub fn check_program_with_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.validate_resolver_symbols(program, symbols);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }
        self.collect_resolver_imports(symbols);
        self.collect_declarations_with_symbols(&program.declarations, symbols);
        self.check_program_after_collection(program)
    }

    pub fn check_module_graph_entry(
        &mut self,
        graph: &ResolvedModuleGraph,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        let Some(entry) = graph.module(graph.entry) else {
            self.diagnostics.push(Diagnostic::error(
                "E0232",
                format!("module graph missing entry module {:?}", graph.entry),
                Span::dummy(),
            ));
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        };

        let mut modules = graph.modules().values().collect::<Vec<_>>();
        modules.sort_by_key(|module| module.info.id.0);

        let mut dependency_programs = Vec::new();
        for module in modules {
            if module.info.id == graph.entry {
                continue;
            }

            let mut checker = TypeChecker::new();
            match checker.check_module_graph_module(graph, module) {
                Ok(typed) => dependency_programs.push(typed),
                Err(diags) => self.diagnostics.extend(diags),
            }
        }

        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        let mut typed = self.check_module_graph_module(graph, entry)?;
        for mut dependency in dependency_programs {
            typed.functions.append(&mut dependency.functions);
            typed.types.append(&mut dependency.types);
            typed.globals.append(&mut dependency.globals);
        }
        Ok(typed)
    }

    fn check_module_graph_module(
        &mut self,
        graph: &ResolvedModuleGraph,
        module: &ResolvedModule,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.validate_resolver_symbols(&module.program, &module.symbols);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        self.collect_module_graph_imports(graph, module);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        self.check_program(&module.program)
    }

    /// Get all diagnostics (errors + warnings).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    // ── Phase 1: Collect ──────────────────────────────────────────

    fn collect_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            if let Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } = decl
            {
                self.behaviors.insert(
                    name.clone(),
                    BehaviorInfo {
                        name: name.clone(),
                        type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                        methods: methods.clone(),
                    },
                );
            }
        }

        for decl in decls {
            if let Declaration::Behavior { type_params, .. } = decl {
                self.validate_generic_bounds(type_params);
            }
        }

        self.validate_self_type_contexts(decls);

        for decl in decls {
            if let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            {
                self.check_behavior_extends(behavior, parent, parent_type_args, *span);
            }
        }
        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();

        for decl in decls {
            match decl {
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    ..
                } => {
                    self.validate_generic_bounds(type_params);
                    self.structs.insert(
                        name.clone(),
                        StructInfo {
                            name: name.clone(),
                            fields: fields
                                .iter()
                                .map(|f| (f.name.clone(), f.ty.clone()))
                                .collect(),
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                            type_param_bounds: type_param_bounds(type_params),
                        },
                    );
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    ..
                } => {
                    self.validate_generic_bounds(type_params);
                    self.enums.insert(
                        name.clone(),
                        EnumInfo {
                            name: name.clone(),
                            variants: variants
                                .iter()
                                .map(|v| (v.name.clone(), v.payload.clone()))
                                .collect(),
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                            type_param_bounds: type_param_bounds(type_params),
                        },
                    );
                }
                Declaration::Import {
                    names, module_path, ..
                } => {
                    for name in names {
                        self.imports.insert(name.clone(), module_path.clone());
                    }
                }
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_generic_bounds(type_params);
                    let ret = return_type.clone().unwrap_or(AstType::Void);
                    let collected_type_params: Vec<String> =
                        type_params.iter().map(|tp| tp.name.clone()).collect();
                    let type_param_bounds = type_param_bounds(type_params);
                    self.functions.insert(
                        name.clone(),
                        FuncInfo {
                            name: name.clone(),
                            params: params
                                .iter()
                                .map(|p| (p.name.clone(), p.ty.clone()))
                                .collect(),
                            return_type: ret,
                            type_params: collected_type_params.clone(),
                            type_param_bounds: type_param_bounds.clone(),
                        },
                    );
                    if !collected_type_params.is_empty() {
                        self.generic_functions.insert(
                            name.clone(),
                            GenericFunctionTemplate::new(
                                collected_type_params,
                                params.clone(),
                                return_type.clone(),
                                body.clone(),
                                *span,
                            ),
                        );
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_generic_bounds(type_params);
                    let key = format!("{}.{}", type_name, method_name);
                    let ret = return_type.clone().unwrap_or(AstType::Void);
                    let collected_type_params: Vec<String> =
                        type_params.iter().map(|tp| tp.name.clone()).collect();
                    let type_param_bounds = type_param_bounds(type_params);
                    self.methods.insert(
                        key.clone(),
                        FuncInfo {
                            name: key,
                            params: params
                                .iter()
                                .map(|p| (p.name.clone(), p.ty.clone()))
                                .collect(),
                            return_type: ret,
                            type_params: collected_type_params.clone(),
                            type_param_bounds: type_param_bounds.clone(),
                        },
                    );
                    if !collected_type_params.is_empty() {
                        self.generic_methods.insert(
                            format!("{}.{}", type_name, method_name),
                            GenericFunctionTemplate::new(
                                collected_type_params,
                                params.clone(),
                                return_type.clone(),
                                body.clone(),
                                *span,
                            ),
                        );
                    }
                }
                Declaration::Behavior { type_params, .. } => {
                    self.validate_generic_bounds(type_params);
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        self.collect_impl_method_signature(type_name, method);
                    }

                    if let Some(behavior) = behavior {
                        self.collect_behavior_default_method_signatures(
                            type_name,
                            behavior,
                            behavior_type_args,
                            methods,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_declarations_with_symbols(&mut self, decls: &[Declaration], symbols: &SymbolTable) {
        self.collect_declarations(decls);

        for decl in decls {
            match decl {
                Declaration::Function { name, .. } => {
                    self.collect_resolver_value_signature(symbols, name);
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } => {
                    self.collect_resolver_value_signature(
                        symbols,
                        &format!("{type_name}.{method_name}"),
                    );
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    for method in methods {
                        if let Declaration::Function { name, .. } = method {
                            self.collect_resolver_value_signature(
                                symbols,
                                &format!("{type_name}.{name}"),
                            );
                        }
                    }
                }
                Declaration::Struct { name, .. } => {
                    self.collect_resolver_struct_fields(symbols, name);
                }
                Declaration::Enum { name, .. } => {
                    self.collect_resolver_enum_variants(symbols, name);
                }
                Declaration::Behavior { name, .. } => {
                    self.collect_resolver_behavior_methods(symbols, name);
                    self.collect_resolver_behavior_parents(symbols, name);
                }
                _ => {}
            }
        }

        for decl in decls {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                ..
            } = decl
            {
                self.collect_behavior_default_method_signatures(
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                );
            }
        }

        self.resolver_backed_collection = true;
        self.validate_collected_declaration_semantics(decls);
        self.resolver_backed_collection = false;

        for decl in decls {
            match decl {
                Declaration::Struct { name, .. } | Declaration::Enum { name, .. } => {
                    self.collect_resolver_type_behavior_impls(symbols, name);
                }
                _ => {}
            }
        }
    }

    fn validate_collected_declaration_semantics(&mut self, decls: &[Declaration]) {
        for decl in decls {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                span,
                ..
            } = decl
            {
                self.check_behavior_impl(type_name, behavior, behavior_type_args, methods, *span);
            }
        }

        for decl in decls {
            if let Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } = decl
            {
                self.check_behavior_requires(type_name, behavior, behavior_type_args, *span);
            }
        }

        self.validate_generic_type_references(decls);
    }

    fn collect_resolver_value_signature(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            return;
        };
        let (Some(parameter_names), Some(parameter_types), Some(return_type)) = (
            symbol.parameter_names.as_ref(),
            symbol.parameter_types.as_ref(),
            symbol.return_type.as_ref(),
        ) else {
            return;
        };
        let ast_type_param_bounds = self
            .functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .map(|info| info.type_param_bounds.clone())
            .unwrap_or_default();
        let type_param_bounds = symbol
            .type_parameter_bound_refs
            .as_deref()
            .map(type_param_bounds_from_resolver_refs)
            .unwrap_or(ast_type_param_bounds);

        let info = FuncInfo {
            name: name.to_string(),
            params: parameter_names
                .iter()
                .cloned()
                .zip(parameter_types.iter().cloned())
                .collect(),
            return_type: return_type.clone(),
            type_params: symbol.type_parameter_names.clone().unwrap_or_default(),
            type_param_bounds,
        };
        self.functions.remove(name);
        self.methods.remove(name);
        if name.contains('.') {
            self.methods.insert(name.to_string(), info);
        } else {
            self.functions.insert(name.to_string(), info);
        }
        self.collect_resolver_generic_template_signature(
            name,
            symbol.type_parameter_names.as_deref().unwrap_or(&[]),
            parameter_types,
            return_type,
        );
    }

    fn collect_resolver_generic_template_signature(
        &mut self,
        name: &str,
        type_parameter_names: &[String],
        parameter_types: &[AstType],
        return_type: &AstType,
    ) {
        let template = if name.contains('.') {
            self.generic_methods.get_mut(name)
        } else {
            self.generic_functions.get_mut(name)
        };
        let Some(template) = template else {
            return;
        };
        template.type_params = type_parameter_names.to_vec();
        if template.params.len() != parameter_types.len() {
            return;
        }
        for (param, ty) in template
            .params
            .iter_mut()
            .zip(parameter_types.iter().cloned())
        {
            param.ty = ty;
        }
        if template.return_type.is_some() {
            template.return_type = Some(return_type.clone());
        }
    }

    fn collect_resolver_struct_fields(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };
        let Some(field_types) = symbol.field_types.as_ref() else {
            return;
        };

        let ast_type_param_bounds = self
            .structs
            .get(name)
            .map(|info| info.type_param_bounds.clone())
            .unwrap_or_default();
        let type_param_bounds = symbol
            .type_parameter_bound_refs
            .as_deref()
            .map(type_param_bounds_from_resolver_refs)
            .unwrap_or(ast_type_param_bounds);
        self.structs.insert(
            name.to_string(),
            StructInfo {
                name: name.to_string(),
                fields: field_types.clone(),
                type_params: symbol.type_parameter_names.clone().unwrap_or_default(),
                type_param_bounds,
            },
        );
    }

    fn collect_resolver_enum_variants(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };
        let Some(variant_names) = symbol.variant_names.as_ref() else {
            return;
        };

        let variants = variant_names
            .iter()
            .map(|variant_name| {
                (
                    variant_name.clone(),
                    symbols
                        .lookup_variant(name, variant_name)
                        .and_then(|variant| variant.variant_payload_type.clone()),
                )
            })
            .collect();
        let ast_type_param_bounds = self
            .enums
            .get(name)
            .map(|info| info.type_param_bounds.clone())
            .unwrap_or_default();
        let type_param_bounds = symbol
            .type_parameter_bound_refs
            .as_deref()
            .map(type_param_bounds_from_resolver_refs)
            .unwrap_or(ast_type_param_bounds);
        self.enums.insert(
            name.to_string(),
            EnumInfo {
                name: name.to_string(),
                variants,
                type_params: symbol.type_parameter_names.clone().unwrap_or_default(),
                type_param_bounds,
            },
        );
    }

    fn collect_resolver_behavior_methods(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Behavior, name) else {
            return;
        };
        let Some(method_types) = symbol.behavior_method_types.as_ref() else {
            return;
        };

        let Some(existing) = self.behaviors.get(name).cloned() else {
            return;
        };
        let methods = existing
            .methods
            .into_iter()
            .map(|method| {
                let Some(metadata) = method_types
                    .iter()
                    .find(|metadata| metadata.name == method.name)
                else {
                    return method;
                };
                if method.params.len() != metadata.parameter_types.len() {
                    return method;
                }
                let params = method
                    .params
                    .into_iter()
                    .zip(metadata.parameter_types.iter().cloned())
                    .map(|(mut param, ty)| {
                        param.ty = ty;
                        param
                    })
                    .collect();
                let return_type = method
                    .return_type
                    .as_ref()
                    .map(|_| metadata.return_type.clone());
                ast::BehaviorMethod {
                    params,
                    return_type,
                    ..method
                }
            })
            .collect();
        self.behaviors.insert(
            name.to_string(),
            BehaviorInfo {
                name: name.to_string(),
                type_params: symbol.type_parameter_names.clone().unwrap_or_default(),
                type_param_bounds: symbol
                    .type_parameter_bound_refs
                    .as_deref()
                    .map(type_param_bounds_from_resolver_refs)
                    .unwrap_or_else(|| existing.type_param_bounds.clone()),
                methods,
            },
        );
    }

    fn collect_resolver_behavior_parents(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Behavior, name) else {
            return;
        };
        let Some(parent_refs) = symbol.behavior_parent_refs.as_ref() else {
            return;
        };

        let parents = parent_refs
            .iter()
            .map(|parent| self.behavior_parent_ref_from_metadata(parent))
            .collect();
        self.behavior_extends.insert(name.to_string(), parents);
    }

    fn collect_resolver_type_behavior_impls(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };
        let Some(impl_refs) = symbol.behavior_impl_refs.as_ref() else {
            return;
        };

        self.behavior_impls
            .retain(|(type_name, _)| type_name != name);
        for behavior in impl_refs {
            self.behavior_impls.insert((
                name.to_string(),
                self.behavior_reference_key(&behavior.name, &behavior.type_args),
            ));
        }
    }

    fn behavior_parent_ref_from_metadata(
        &self,
        metadata: &BehaviorRefMetadata,
    ) -> BehaviorParentRef {
        BehaviorParentRef {
            behavior: metadata.name.clone(),
            type_args: metadata.type_args.clone(),
            key: self.behavior_reference_key(&metadata.name, &metadata.type_args),
        }
    }

    fn collect_impl_method_signature(&mut self, type_name: &str, method: &Declaration) {
        let Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = method
        else {
            return;
        };

        self.validate_generic_bounds(type_params);
        let key = format!("{}.{}", type_name, name);
        let collected_type_params: Vec<String> =
            type_params.iter().map(|tp| tp.name.clone()).collect();
        self.methods.insert(
            key.clone(),
            FuncInfo {
                name: key.clone(),
                params: params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_type: return_type.clone().unwrap_or(AstType::Void),
                type_params: collected_type_params.clone(),
                type_param_bounds: type_param_bounds(type_params),
            },
        );
        if !collected_type_params.is_empty() {
            self.generic_methods.insert(
                key,
                GenericFunctionTemplate::new(
                    collected_type_params,
                    params.to_vec(),
                    return_type.clone(),
                    body.clone(),
                    *span,
                ),
            );
        }
    }

    fn collect_behavior_default_method_signatures(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        for default in
            self.behavior_default_methods_for_impl(type_name, behavior, behavior_type_args, methods)
        {
            let key = format!("{}.{}", type_name, default.name);
            self.methods.insert(
                key.clone(),
                FuncInfo {
                    name: key,
                    params: default
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_type: default.return_type.unwrap_or(AstType::Void),
                    type_params: Vec::new(),
                    type_param_bounds: HashMap::new(),
                },
            );
        }
    }

    fn behavior_reference_key(&self, behavior: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            behavior.to_string()
        } else {
            self.mangle_generic_type_name(behavior, type_args)
        }
    }

    fn behavior_type_arg_substitutions(
        &mut self,
        behavior: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) -> Option<HashMap<String, AstType>> {
        let Some(info) = self.behaviors.get(behavior).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return None;
        };

        if info.type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic behavior `{}` expects {} type arguments, found {}",
                    behavior,
                    info.type_params.len(),
                    type_args.len()
                ),
                span,
            ));
            return None;
        }

        let ast_substitutions: HashMap<String, AstType> = info
            .type_params
            .iter()
            .cloned()
            .zip(type_args.iter().cloned())
            .collect();
        let type_substitutions: HashMap<String, Type> = info
            .type_params
            .iter()
            .zip(type_args.iter())
            .filter_map(|(param, arg)| {
                if ast_type_references_type_param(arg, scoped_type_params) {
                    None
                } else {
                    Some((param.clone(), self.resolve_type(arg)))
                }
            })
            .collect();
        let error_count = self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count();
        self.check_generic_bounds(&info.type_param_bounds, &type_substitutions, span);
        if self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count()
            > error_count
        {
            return None;
        }

        Some(ast_substitutions)
    }

    fn check_behavior_requires(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(_) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if !self.type_implements_behavior(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6007",
                format!(
                    "type `{}` does not implement required behavior `{}`",
                    type_name, behavior_key
                ),
                span,
            ));
        }
    }

    fn check_behavior_extends(
        &mut self,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
    ) {
        if !self.behaviors.contains_key(behavior) {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return;
        }

        let scoped_type_params: HashSet<String> = self
            .behaviors
            .get(behavior)
            .map(|info| info.type_params.iter().cloned().collect())
            .unwrap_or_default();
        let Some(_) = self.behavior_type_arg_substitutions(
            parent,
            parent_type_args,
            &scoped_type_params,
            span,
        ) else {
            return;
        };

        let parent_key = self.behavior_reference_key(parent, parent_type_args);
        let parent_display = behavior_ref_display(parent, parent_type_args);
        let parents = self
            .behavior_extends
            .entry(behavior.to_string())
            .or_default();
        if parents.iter().any(|existing| existing.key == parent_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6011",
                format!("duplicate behavior inheritance `{behavior}.extends({parent_display})`"),
                span,
            ));
            return;
        }

        parents.push(BehaviorParentRef {
            behavior: parent.to_string(),
            type_args: parent_type_args.to_vec(),
            key: parent_key,
        });
        self.behavior_extends_spans
            .entry(behavior.to_string())
            .or_insert(span);
    }

    fn validate_behavior_extends_cycles(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        for behavior in behaviors {
            let mut visiting = HashSet::new();
            let mut visited = HashSet::new();
            if self.behavior_extends_has_cycle(&behavior, &mut visiting, &mut visited) {
                let span = self
                    .behavior_extends_spans
                    .get(&behavior)
                    .copied()
                    .unwrap_or_else(Span::dummy);
                self.diagnostics.push(Diagnostic::error(
                    "E6008",
                    format!("behavior inheritance cycle involving `{}`", behavior),
                    span,
                ));
            }
        }
    }

    fn behavior_extends_has_cycle(
        &self,
        behavior: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visiting.contains(behavior) {
            return true;
        }
        if !visited.insert(behavior.to_string()) {
            return false;
        }

        visiting.insert(behavior.to_string());
        let has_cycle = self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents
                .iter()
                .any(|parent| self.behavior_extends_has_cycle(&parent.key, visiting, visited))
        });
        visiting.remove(behavior);
        has_cycle
    }

    fn validate_behavior_method_coherence(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        let mut diagnostics = Vec::new();

        for behavior in behaviors {
            let mut seen_behaviors = HashSet::new();
            let mut seen_methods = HashMap::new();
            self.collect_behavior_method_coherence_errors(
                &behavior,
                &behavior,
                &HashMap::new(),
                &mut seen_behaviors,
                &mut seen_methods,
                &mut diagnostics,
            );
        }

        self.diagnostics.extend(diagnostics);
    }

    fn collect_behavior_method_coherence_errors(
        &self,
        behavior: &str,
        root_behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen_behaviors: &mut HashSet<String>,
        seen_methods: &mut HashMap<String, ast::BehaviorMethod>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let behavior_seen_key = self.behavior_seen_key(behavior, substitutions);
        if !seen_behaviors.insert(behavior_seen_key) {
            return;
        }

        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_type_args: Vec<AstType> = parent
                    .type_args
                    .iter()
                    .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
                    .collect();
                let parent_substitutions = self
                    .behaviors
                    .get(&parent.behavior)
                    .map(|info| {
                        info.type_params
                            .iter()
                            .cloned()
                            .zip(parent_type_args)
                            .collect::<HashMap<_, _>>()
                    })
                    .unwrap_or_default();
                self.collect_behavior_method_coherence_errors(
                    &parent.behavior,
                    root_behavior,
                    &parent_substitutions,
                    seen_behaviors,
                    seen_methods,
                    diagnostics,
                );
            }
        }

        if let Some(info) = self.behaviors.get(behavior) {
            for method in &info.methods {
                let mut method = method.clone();
                for param in &mut method.params {
                    param.ty = substitute_behavior_ast_type(&param.ty, substitutions);
                }
                if let Some(return_type) = &mut method.return_type {
                    *return_type = substitute_behavior_ast_type(return_type, substitutions);
                }

                if let Some(previous) = seen_methods.get(&method.name) {
                    if !behavior_method_signatures_match(previous, &method) {
                        diagnostics.push(Diagnostic::error(
                            "E6009",
                            format!(
                                "conflicting behavior method `{}` inherited by `{}`",
                                method.name, root_behavior
                            ),
                            method.span,
                        ));
                    }
                } else {
                    seen_methods.insert(method.name.clone(), method);
                }
            }
        }
    }

    fn type_implements_behavior(&self, type_name: &str, behavior: &str) -> bool {
        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior.to_string()))
        {
            return true;
        }

        self.behavior_impls
            .iter()
            .any(|(implemented_type, implemented_behavior)| {
                implemented_type == type_name
                    && self.behavior_inherits_from(implemented_behavior, behavior)
            })
    }

    fn behavior_inherits_from(&self, behavior: &str, parent: &str) -> bool {
        self.behavior_inherits_from_inner(behavior, parent, &mut HashSet::new())
    }

    fn behavior_inherits_from_inner(
        &self,
        behavior: &str,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(behavior.to_string()) {
            return false;
        }

        self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents.iter().any(|candidate| {
                candidate.key == parent
                    || self.behavior_inherits_from_inner(&candidate.key, parent, seen)
            })
        })
    }

    fn check_behavior_impl(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(behavior_substitutions) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior_key.clone()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for type `{}`",
                    behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        if let Some(existing) = self.find_overlapping_behavior_impl(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6010",
                format!(
                    "overlapping implementations of behaviors `{}` and `{}` for type `{}`",
                    existing, behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        self.behavior_impls
            .insert((type_name.to_string(), behavior_key.clone()));
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());

        for method in methods {
            if let Declaration::Function { name, span, .. } = method {
                if !required_methods
                    .iter()
                    .any(|required| required.name == *name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            name, behavior_key
                        ),
                        *span,
                    ));
                }
            }
        }

        for required in &required_methods {
            let Some(actual) = methods.iter().find_map(|decl| match decl {
                Declaration::Function {
                    name,
                    params,
                    return_type,
                    span,
                    ..
                } if name == &required.name => Some((params, return_type, *span)),
                _ => None,
            }) else {
                if required.default_body.is_some() {
                    continue;
                }
                self.diagnostics.push(Diagnostic::error(
                    "E6001",
                    format!(
                        "type `{}` implementation of `{}` is missing required method `{}`",
                        type_name, behavior_key, required.name
                    ),
                    span,
                ));
                continue;
            };

            let (actual_params, actual_return_type, actual_span) = actual;
            let method_key = format!("{}.{}", type_name, required.name);
            let collected_signature = self
                .resolver_backed_collection
                .then(|| self.methods.get(&method_key))
                .flatten();
            let actual_param_types: Vec<AstType> = collected_signature
                .map(|info| {
                    info.params
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| actual_params.iter().map(|param| param.ty.clone()).collect());
            let actual_return = collected_signature
                .map(|info| info.return_type.clone())
                .unwrap_or_else(|| actual_return_type.clone().unwrap_or(AstType::Void));

            if actual_param_types.len() != required.params.len() {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects {} parameters, found {}",
                        required.name,
                        behavior_key,
                        required.params.len(),
                        actual_param_types.len()
                    ),
                    actual_span,
                ));
                continue;
            }

            for (idx, (expected, actual_ty)) in required
                .params
                .iter()
                .zip(actual_param_types.iter())
                .enumerate()
            {
                if !self.impl_ast_types_compatible(&expected.ty, actual_ty, type_name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E6002",
                        format!(
                            "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
                            idx + 1,
                            required.name,
                            behavior_key,
                            self.impl_type_display(&expected.ty, type_name),
                            actual_ty.display_name()
                        ),
                        actual_params
                            .get(idx)
                            .map(|param| param.span)
                            .unwrap_or(actual_span),
                    ));
                }
            }

            let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
            if !self.impl_ast_types_compatible(expected_return, &actual_return, type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                        required.name,
                        behavior_key,
                        self.impl_type_display(expected_return, type_name),
                        actual_return.display_name()
                    ),
                    actual_span,
                ));
            }
        }
    }

    fn find_overlapping_behavior_impl(&self, type_name: &str, behavior: &str) -> Option<String> {
        self.behavior_impls
            .iter()
            .filter(|(implemented_type, _)| implemented_type == type_name)
            .map(|(_, implemented_behavior)| implemented_behavior)
            .find(|implemented_behavior| {
                self.behavior_inherits_from(implemented_behavior, behavior)
                    || self.behavior_inherits_from(behavior, implemented_behavior)
            })
            .cloned()
    }

    fn reject_unspecialized_generic_type(&mut self, type_name: &str, span: Span) -> bool {
        let type_param_count = self
            .structs
            .get(type_name)
            .map(|info| info.type_params.len())
            .or_else(|| self.enums.get(type_name).map(|info| info.type_params.len()))
            .unwrap_or(0);
        if type_param_count == 0 {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E6013",
            format!(
                "generic type `{}` expects {} type arguments, found 0",
                type_name, type_param_count
            ),
            span,
        ));
        true
    }

    fn behavior_default_methods_for_impl(
        &self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) -> Vec<DefaultBehaviorMethod> {
        let behavior_substitutions = self
            .behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .cloned()
                    .zip(behavior_type_args.iter().cloned())
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .iter()
            .filter(|required| {
                required.default_body.is_some()
                    && !methods.iter().any(|decl| {
                        matches!(decl, Declaration::Function { name, .. } if name == &required.name)
                    })
            })
            .filter_map(|required| {
                let body = required.default_body.clone()?;
                Some(DefaultBehaviorMethod {
                    name: required.name.clone(),
                    params: required
                        .params
                        .iter()
                        .map(|param| Param {
                            name: param.name.clone(),
                            ty: concrete_self_ast_type(&param.ty, type_name),
                            mutable: param.mutable,
                            span: param.span,
                        })
                        .collect(),
                    return_type: required
                        .return_type
                        .as_ref()
                        .map(|ty| concrete_self_ast_type(ty, type_name)),
                    body,
                    span: required.span,
                })
            })
            .collect()
    }

    fn behavior_methods_with_inherited_substituted(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        let behavior_seen_key = self.behavior_seen_key(behavior, substitutions);
        if !seen.insert(behavior_seen_key) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_type_args: Vec<AstType> = parent
                    .type_args
                    .iter()
                    .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
                    .collect();
                let parent_substitutions = self
                    .behaviors
                    .get(&parent.behavior)
                    .map(|info| {
                        info.type_params
                            .iter()
                            .cloned()
                            .zip(parent_type_args)
                            .collect::<HashMap<_, _>>()
                    })
                    .unwrap_or_default();
                methods.extend(self.behavior_methods_with_inherited_substituted(
                    &parent.behavior,
                    &parent_substitutions,
                    seen,
                ));
            }
        }
        if let Some(info) = self.behaviors.get(behavior) {
            methods.extend(info.methods.iter().cloned().map(|mut method| {
                for param in &mut method.params {
                    param.ty = substitute_behavior_ast_type(&param.ty, substitutions);
                }
                if let Some(return_type) = &mut method.return_type {
                    *return_type = substitute_behavior_ast_type(return_type, substitutions);
                }
                method
            }));
        }
        methods
    }

    fn behavior_seen_key(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
    ) -> String {
        let type_args = self
            .behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .filter_map(|param| substitutions.get(param).cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        behavior_ref_display(behavior, &type_args)
    }

    fn behavior_methods_for_impl(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        self.behavior_methods_with_inherited_substituted(behavior, substitutions, seen)
    }

    fn impl_ast_types_compatible(
        &self,
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
    ) -> bool {
        match expected {
            AstType::SelfType => matches!(actual, AstType::Named(name) if name == self_type_name),
            _ => expected == actual,
        }
    }

    fn impl_type_display(&self, ty: &AstType, self_type_name: &str) -> String {
        match ty {
            AstType::SelfType => self_type_name.to_string(),
            _ => ty.display_name(),
        }
    }

    fn validate_generic_bounds(&mut self, type_params: &[ast::TypeParam]) {
        for param in type_params {
            if let Some(bound) = &param.constraint {
                if !self.behaviors.contains_key(bound) {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
                        format!(
                            "generic bound `{}` on type parameter `{}` references undefined behavior",
                            bound, param.name
                        ),
                        param.span,
                    ));
                } else {
                    let expected = self
                        .behaviors
                        .get(bound)
                        .map(|info| info.type_params.len())
                        .unwrap_or(0);
                    let found = param.constraint_type_args.len();
                    if expected != found {
                        self.diagnostics.push(Diagnostic::error(
                            "E6012",
                            format!(
                                "generic behavior `{}` expects {} type arguments, found {}",
                                bound, expected, found
                            ),
                            param.span,
                        ));
                    }
                }
            }
        }
    }

    fn validate_generic_type_references(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                    }
                }
                Declaration::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                Declaration::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                    ..
                }
                | Declaration::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for param in params {
                        self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                    }
                    if let Some(return_type) = return_type {
                        self.validate_generic_type_ref_bounds(return_type, &scoped, Span::dummy());
                    }
                    self.validate_generic_expr_type_references(body, &scoped);
                }
                Declaration::Behavior {
                    type_params,
                    methods,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in methods {
                        for param in &method.params {
                            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                        }
                        if let Some(return_type) = &method.return_type {
                            self.validate_generic_type_ref_bounds(
                                return_type,
                                &scoped,
                                method.span,
                            );
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_generic_expr_type_references(default_body, &scoped);
                        }
                    }
                }
                Declaration::ImplBlock { methods, .. } => {
                    for method in methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            let scoped = type_param_name_set(type_params);
                            for param in params {
                                self.validate_generic_type_ref_bounds(
                                    &param.ty, &scoped, param.span,
                                );
                            }
                            if let Some(return_type) = return_type {
                                self.validate_generic_type_ref_bounds(
                                    return_type,
                                    &scoped,
                                    method.span(),
                                );
                            }
                            self.validate_generic_expr_type_references(body, &scoped);
                        }
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
                _ => {}
            }
        }
    }

    fn validate_self_type_contexts(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Struct { fields, .. } => {
                    for field in fields {
                        self.validate_self_type_ref(&field.ty, field.span, false);
                        if let Some(default) = &field.default {
                            self.validate_self_type_expr(default, false);
                        }
                    }
                }
                Declaration::Enum { variants, .. } => {
                    for variant in variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_self_type_ref(payload, variant.span, false);
                        }
                    }
                }
                Declaration::Function {
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_self_type_params(params, false);
                    if let Some(return_type) = return_type {
                        self.validate_self_type_ref(return_type, *span, false);
                    }
                    self.validate_self_type_expr(body, false);
                }
                Declaration::Method {
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_self_type_params(params, true);
                    if let Some(return_type) = return_type {
                        self.validate_self_type_ref(return_type, *span, true);
                    }
                    self.validate_self_type_expr(body, true);
                }
                Declaration::Behavior { methods, .. } => {
                    for method in methods {
                        self.validate_self_type_params(&method.params, true);
                        if let Some(return_type) = &method.return_type {
                            self.validate_self_type_ref(return_type, method.span, true);
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_self_type_expr(default_body, true);
                        }
                    }
                }
                Declaration::ImplBlock {
                    behavior_type_args,
                    methods,
                    span,
                    ..
                } => {
                    for type_arg in behavior_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                    for method in methods {
                        if let Declaration::Function {
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            self.validate_self_type_params(params, true);
                            if let Some(return_type) = return_type {
                                self.validate_self_type_ref(return_type, *span, true);
                            }
                            self.validate_self_type_expr(body, true);
                        }
                    }
                }
                Declaration::Requires {
                    behavior_type_args,
                    span,
                    ..
                } => {
                    for type_arg in behavior_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                }
                Declaration::BehaviorExtends {
                    parent_type_args,
                    span,
                    ..
                } => {
                    for type_arg in parent_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.validate_self_type_expr(expr, false);
                }
                Declaration::Import { .. } | Declaration::Error { .. } => {}
            }
        }
    }

    fn validate_self_type_params(&mut self, params: &[Param], allow_self_type: bool) {
        for param in params {
            self.validate_self_type_ref(&param.ty, param.span, allow_self_type);
        }
    }

    fn validate_self_type_ref(&mut self, ast_type: &AstType, span: Span, allow_self_type: bool) {
        match ast_type {
            AstType::SelfType => {
                if !allow_self_type {
                    self.diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            AstType::Generic { type_args, .. } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, span, allow_self_type);
                }
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_self_type_ref(inner, span, allow_self_type);
            }
            AstType::Array { elem, .. } => {
                self.validate_self_type_ref(elem, span, allow_self_type);
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_self_type_ref(param, span, allow_self_type);
                }
                self.validate_self_type_ref(ret, span, allow_self_type);
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::Named(_)
            | AstType::Inferred => {}
        }
    }

    fn validate_self_type_expr(&mut self, expr: &Expression, allow_self_type: bool) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_self_type_expr(receiver, allow_self_type);
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_self_type_expr(left, allow_self_type);
                self.validate_self_type_expr(right, allow_self_type);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_self_type_expr(operand, allow_self_type);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
                self.validate_self_type_expr(index, allow_self_type);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for (_, value) in fields {
                    self.validate_self_type_expr(value, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: None,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: Some(payload),
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                self.validate_self_type_expr(payload, allow_self_type);
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_self_type_expr(element, allow_self_type);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_self_type_expr(scrutinee, allow_self_type);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_self_type_expr(guard, allow_self_type);
                    }
                    self.validate_self_type_expr(&arm.body, allow_self_type);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Loop { body, .. } => {
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(then_body, allow_self_type);
                if let Some(else_body) = else_body {
                    self.validate_self_type_expr(else_body, allow_self_type);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
                if let Some(expr) = expr {
                    self.validate_self_type_expr(expr, allow_self_type);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_self_type_expr(value, allow_self_type);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                self.validate_self_type_params(params, allow_self_type);
                if let Some(return_type) = return_type {
                    self.validate_self_type_ref(return_type, *span, allow_self_type);
                }
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_self_type_expr(expr, allow_self_type);
                self.validate_self_type_ref(target_type, *span, allow_self_type);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_self_type_expr(expr, allow_self_type);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_self_type_expr(start, allow_self_type);
                self.validate_self_type_expr(end, allow_self_type);
            }
            Expression::Defer { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_self_type_statement(&mut self, statement: &ast::Statement, allow_self_type: bool) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_self_type_ref(ty, *span, allow_self_type);
                }
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_self_type_expr(target, allow_self_type);
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
            }
        }
    }

    fn validate_generic_type_ref_bounds(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_ref_bounds_allow_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_ref_bounds_with_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if scoped_type_params.contains(name) {
                    return;
                }

                if !self.is_known_named_type(name) {
                    if reject_unknown {
                        self.diagnostics.push(Diagnostic::error(
                            "E0201",
                            format!("unknown type symbol '{name}'"),
                            span,
                        ));
                    }
                    return;
                }

                let generic = self
                    .structs
                    .get(name)
                    .map(|info| ("struct", info.type_params.len()))
                    .or_else(|| {
                        self.enums
                            .get(name)
                            .map(|info| ("enum", info.type_params.len()))
                    });
                if let Some((kind, type_param_count)) = generic {
                    if type_param_count > 0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E5001",
                            format!(
                                "generic {} `{}` expects {} type arguments, found 0",
                                kind, name, type_param_count
                            ),
                            span,
                        ));
                    }
                }
            }
            AstType::Generic { name, type_args } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds_with_unknowns(
                        type_arg,
                        scoped_type_params,
                        span,
                        reject_unknown,
                    );
                }

                if scoped_type_params.contains(name) {
                    return;
                }

                let (kind, type_params, type_param_bounds) =
                    if let Some(info) = self.structs.get(name) {
                        (
                            "struct",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else if let Some(info) = self.enums.get(name) {
                        (
                            "enum",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else {
                        if reject_unknown && !self.imports.contains_key(name) {
                            self.diagnostics.push(Diagnostic::error(
                                "E0201",
                                format!("unknown type symbol '{name}'"),
                                span,
                            ));
                        }
                        return;
                    };

                if type_params.len() != type_args.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5001",
                        format!(
                            "generic {} `{}` expects {} type arguments, found {}",
                            kind,
                            name,
                            type_params.len(),
                            type_args.len()
                        ),
                        span,
                    ));
                    return;
                }

                let substitutions: HashMap<String, Type> = type_params
                    .iter()
                    .zip(type_args.iter())
                    .filter_map(|(param, arg)| {
                        if ast_type_references_type_param(arg, scoped_type_params) {
                            None
                        } else {
                            Some((param.clone(), self.resolve_type(arg)))
                        }
                    })
                    .collect();
                self.check_generic_bounds(&type_param_bounds, &substitutions, span);
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    inner,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Array { elem, .. } => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    elem,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_generic_type_ref_bounds_with_unknowns(
                        param,
                        scoped_type_params,
                        span,
                        reject_unknown,
                    );
                }
                self.validate_generic_type_ref_bounds_with_unknowns(
                    ret,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            _ => {}
        }
    }

    fn is_known_named_type(&self, name: &str) -> bool {
        self.structs.contains_key(name)
            || self.enums.contains_key(name)
            || self.imports.contains_key(name)
    }

    fn validate_generic_expr_type_references(
        &mut self,
        expr: &Expression,
        scoped_type_params: &HashSet<String>,
    ) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_generic_expr_type_references(receiver, scoped_type_params);
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_generic_expr_type_references(left, scoped_type_params);
                self.validate_generic_expr_type_references(right, scoped_type_params);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_generic_expr_type_references(operand, scoped_type_params);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
                self.validate_generic_expr_type_references(index, scoped_type_params);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for (_, value) in fields {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                if let Some(payload) = payload {
                    self.validate_generic_expr_type_references(payload, scoped_type_params);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_generic_expr_type_references(element, scoped_type_params);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_generic_expr_type_references(scrutinee, scoped_type_params);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_generic_expr_type_references(guard, scoped_type_params);
                    }
                    self.validate_generic_expr_type_references(&arm.body, scoped_type_params);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Loop { body, .. } => {
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(then_body, scoped_type_params);
                if let Some(else_body) = else_body {
                    self.validate_generic_expr_type_references(else_body, scoped_type_params);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
                if let Some(expr) = expr {
                    self.validate_generic_expr_type_references(expr, scoped_type_params);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                for param in params {
                    self.validate_generic_type_ref_bounds(
                        &param.ty,
                        scoped_type_params,
                        param.span,
                    );
                }
                if let Some(return_type) = return_type {
                    self.validate_generic_type_ref_bounds(return_type, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
                self.validate_generic_type_ref_bounds(target_type, scoped_type_params, *span);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_generic_expr_type_references(expr, scoped_type_params);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_generic_expr_type_references(start, scoped_type_params);
                self.validate_generic_expr_type_references(end, scoped_type_params);
            }
            Expression::Defer { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_generic_statement_type_references(
        &mut self,
        statement: &ast::Statement,
        scoped_type_params: &HashSet<String>,
    ) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_generic_type_ref_bounds(ty, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_generic_expr_type_references(target, scoped_type_params);
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
            }
        }
    }

    pub(crate) fn check_generic_bounds(
        &mut self,
        bounds: &HashMap<String, BehaviorBound>,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        for (param, bound) in bounds {
            let Some(concrete) = substitutions.get(param) else {
                continue;
            };
            let behavior_key = self.behavior_bound_key(bound, substitutions);
            let behavior_display = behavior_bound_display(bound, substitutions);
            let Some(type_name) = self.behavior_bound_type_name(concrete) else {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        concrete.display_name(),
                        behavior_display,
                        param
                    ),
                    span,
                ));
                continue;
            };
            if !self.type_implements_behavior(&type_name, &behavior_key) {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        type_name, behavior_display, param
                    ),
                    span,
                ));
            }
        }
    }

    fn behavior_bound_key(
        &self,
        bound: &BehaviorBound,
        substitutions: &HashMap<String, Type>,
    ) -> String {
        let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
        self.behavior_reference_key(&bound.behavior, &type_args)
    }

    fn behavior_bound_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
                Some(name.clone())
            }
            _ => None,
        }
    }

    // ── Scope Management ──────────────────────────────────────────

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn define_var(&mut self, name: &str, ty: Type) {
        self.define_var_with_mutability(name, ty, false);
    }

    pub(crate) fn define_var_with_mutability(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name.to_string(), VarInfo { ty, mutable });
        }
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<Type> {
        self.lookup_var_info(name).map(|info| info.ty.clone())
    }

    pub(crate) fn lookup_var_info(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.vars.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub(crate) fn is_import(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    pub(crate) fn is_root_std_import(&self, name: &str) -> bool {
        self.imports
            .get(name)
            .is_some_and(|path| path == &["std".to_string()] || path == &["@std".to_string()])
    }

    fn validate_resolver_symbols(&mut self, program: &ast::Program, symbols: &SymbolTable) {
        let mut scope_cursor = ResolverScopeCursor::default();
        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    params,
                    return_type,
                    type_params,
                    public,
                    span,
                    body,
                    ..
                } => {
                    self.require_resolver_value_symbol(
                        symbols,
                        name,
                        expected_value_signature(params, return_type, type_params),
                        *public,
                        *span,
                    );
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_parameter_locals(symbols, params, &mut locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        body,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    params,
                    return_type,
                    type_params,
                    public,
                    span,
                    body,
                    ..
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    self.require_resolver_value_symbol(
                        symbols,
                        &format!("{type_name}.{method_name}"),
                        expected_value_signature(params, return_type, type_params),
                        *public,
                        *span,
                    );
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_parameter_locals(symbols, params, &mut locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        body,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    public,
                    span,
                    ..
                } => {
                    let Some(symbol) = self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Type,
                        name,
                        expected_type_like_symbol(type_params, Some(*public)),
                        *span,
                    ) else {
                        continue;
                    };
                    self.validate_resolver_field_count(
                        symbol,
                        Namespace::Type,
                        name,
                        fields.len(),
                        *span,
                    );
                    self.validate_resolver_field_types(
                        symbol,
                        Namespace::Type,
                        name,
                        expected_field_types(fields),
                        expected_field_type_names(fields),
                        *span,
                    );
                    self.validate_resolver_struct_absent_enum_metadata(symbol, name, *span);
                    for field in fields {
                        if let Some(default) = &field.default {
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_expr_locals(
                                symbols,
                                default,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    public,
                    span,
                    ..
                } => {
                    if let Some(symbol) = self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Type,
                        name,
                        expected_type_like_symbol(type_params, Some(*public)),
                        *span,
                    ) {
                        self.validate_resolver_variant_names(
                            symbol,
                            name,
                            expected_variant_names(variants),
                            *span,
                        );
                        self.validate_resolver_enum_absent_struct_metadata(symbol, name, *span);
                    }
                    for variant in variants {
                        let Some(symbol) = symbols.lookup_variant(name, &variant.name) else {
                            if let Some(symbol) = symbols.lookup(Namespace::Variant, &variant.name)
                            {
                                self.validate_resolver_variant_owner_name(
                                    symbol,
                                    &variant.name,
                                    name,
                                    variant.span,
                                );
                                continue;
                            }
                            self.require_resolver_symbol(
                                symbols,
                                Namespace::Variant,
                                &variant.name,
                                variant.span,
                            );
                            continue;
                        };
                        self.validate_resolver_variant_payload_count(
                            symbol,
                            &variant.name,
                            usize::from(variant.payload.is_some()),
                            variant.span,
                        );
                        self.validate_resolver_variant_owner_name(
                            symbol,
                            &variant.name,
                            name,
                            variant.span,
                        );
                        self.validate_resolver_variant_visibility(
                            symbol,
                            &variant.name,
                            *public,
                            variant.span,
                        );
                        self.validate_resolver_variant_payload_type(
                            symbol,
                            &variant.name,
                            variant.payload.clone(),
                            expected_variant_payload_type_name(&variant.payload),
                            variant.span,
                        );
                        self.validate_resolver_variant_absent_other_metadata(
                            symbol,
                            &variant.name,
                            variant.span,
                        );
                    }
                }
                Declaration::Behavior {
                    name,
                    type_params,
                    methods,
                    public,
                    span,
                    ..
                } => {
                    let Some(symbol) = self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Behavior,
                        name,
                        expected_type_like_symbol(type_params, Some(*public)),
                        *span,
                    ) else {
                        continue;
                    };
                    self.validate_resolver_behavior_method_signatures(
                        symbol,
                        name,
                        expected_behavior_method_signatures(methods),
                        expected_behavior_method_types(methods),
                        *span,
                    );
                    self.validate_resolver_behavior_absent_type_metadata(symbol, name, *span);
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_parameter_locals(
                                symbols,
                                &method.params,
                                &mut locals,
                            );
                            self.require_resolver_expr_locals(
                                symbols,
                                default_body,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Import {
                    names,
                    module_path,
                    span,
                } => {
                    self.require_resolver_module_symbol(symbols, &module_path.join("."), *span);
                    for name in names {
                        self.require_resolver_import_symbol(
                            symbols,
                            name,
                            &module_path.join("."),
                            *span,
                        );
                    }
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    span,
                    ..
                } => {
                    let type_symbol = symbols.lookup(Namespace::Type, type_name);
                    if type_symbol.is_none() {
                        self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    }
                    if let Some(behavior) = behavior {
                        self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                        if let Some(symbol) = type_symbol {
                            self.validate_resolver_behavior_impl_names(
                                symbol,
                                type_name,
                                &behavior_ref_display(behavior, behavior_type_args),
                                &BehaviorRefMetadata {
                                    name: behavior.clone(),
                                    type_args: behavior_type_args.clone(),
                                },
                                *span,
                            );
                        }
                    }
                    for type_arg in behavior_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            params,
                            return_type,
                            type_params,
                            public,
                            span,
                            body,
                            ..
                        } = method
                        {
                            self.require_resolver_value_symbol(
                                symbols,
                                &format!("{type_name}.{name}"),
                                expected_value_signature(params, return_type, type_params),
                                *public,
                                *span,
                            );
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_parameter_locals(symbols, params, &mut locals);
                            self.require_resolver_expr_locals(
                                symbols,
                                body,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    span,
                } => {
                    let type_symbol = symbols.lookup(Namespace::Type, type_name);
                    if type_symbol.is_none() {
                        self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    }
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    if let Some(symbol) = type_symbol {
                        self.validate_resolver_behavior_required_names(
                            symbol,
                            type_name,
                            &behavior_ref_display(behavior, behavior_type_args),
                            &BehaviorRefMetadata {
                                name: behavior.clone(),
                                type_args: behavior_type_args.clone(),
                            },
                            *span,
                        );
                    }
                    for type_arg in behavior_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    span,
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    self.require_resolver_symbol(symbols, Namespace::Behavior, parent, *span);
                    for type_arg in parent_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                    if let Some(symbol) = symbols.lookup(Namespace::Behavior, behavior) {
                        self.validate_resolver_behavior_parent_names(
                            symbol,
                            behavior,
                            &behavior_ref_display(parent, parent_type_args),
                            &BehaviorRefMetadata {
                                name: parent.clone(),
                                type_args: parent_type_args.clone(),
                            },
                            *span,
                        );
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_expr_locals(
                        symbols,
                        expr,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Error { .. } => {}
            }
        }
        self.validate_no_extra_resolver_declaration_symbols(program, symbols);
        self.validate_no_extra_resolver_local_symbols(program, symbols);
        self.validate_resolver_behavior_association_lists(program, symbols);
        self.validate_resolver_behavior_parent_lists(program, symbols);
        self.validate_stripped_resolver_import_symbols(program, symbols);
    }

    fn validate_no_extra_resolver_declaration_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_resolver_declaration_symbols(program);
        let validate_imports = program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Declaration::Import { .. }));
        for symbol in symbols.symbols() {
            if !validate_imports
                && matches!(symbol.namespace, Namespace::Module | Namespace::Import)
            {
                continue;
            }
            if !matches!(
                symbol.namespace,
                Namespace::Value
                    | Namespace::Type
                    | Namespace::Behavior
                    | Namespace::Variant
                    | Namespace::Module
                    | Namespace::Import
            ) {
                continue;
            }
            if !expected.contains(&(symbol.namespace, symbol.name.clone())) {
                self.diagnostics.push(Diagnostic::error(
                    "E0243",
                    format!(
                        "resolver symbol table has extra {} symbol '{}'",
                        symbol.namespace.diagnostic_name(),
                        symbol.name
                    ),
                    symbol.definition_span,
                ));
            }
        }
    }

    fn validate_no_extra_resolver_local_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_resolver_local_symbols(program);
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Local {
                continue;
            }
            if !expected.contains(&(symbol.name.clone(), symbol.scope_id)) {
                self.diagnostics.push(Diagnostic::error(
                    "E0244",
                    format!(
                        "resolver symbol table has extra local symbol '{}'",
                        symbol.name
                    ),
                    symbol.definition_span,
                ));
            }
        }
    }

    fn validate_resolver_behavior_association_lists(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let (expected_impls, expected_requires) = expected_behavior_associations(program);
        let (expected_impl_refs, expected_required_refs) =
            expected_behavior_association_refs(program);
        for decl in &program.declarations {
            let (Declaration::Struct { name, span, .. } | Declaration::Enum { name, span, .. }) =
                decl
            else {
                continue;
            };
            let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
                continue;
            };
            self.validate_resolver_behavior_impl_list(
                symbol,
                name,
                expected_impls.get(name).map(Vec::as_slice).unwrap_or(&[]),
                expected_impl_refs
                    .get(name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                *span,
            );
            self.validate_resolver_behavior_required_list(
                symbol,
                name,
                expected_requires
                    .get(name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                expected_required_refs
                    .get(name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                *span,
            );
        }
    }

    fn validate_resolver_behavior_parent_lists(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected_parents = expected_behavior_parent_associations(program);
        let expected_parent_refs = expected_behavior_parent_ref_associations(program);
        for decl in &program.declarations {
            let Declaration::Behavior { name, span, .. } = decl else {
                continue;
            };
            let Some(symbol) = symbols.lookup(Namespace::Behavior, name) else {
                continue;
            };
            self.validate_resolver_behavior_parent_list(
                symbol,
                name,
                expected_parents.get(name).map(Vec::as_slice).unwrap_or(&[]),
                expected_parent_refs
                    .get(name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                *span,
            );
        }
    }

    fn require_resolver_module_symbol(
        &mut self,
        symbols: &SymbolTable,
        expected_module: &str,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Module, expected_module) else {
            self.require_resolver_symbol(symbols, Namespace::Module, expected_module, span);
            return;
        };

        if symbol.is_public {
            self.diagnostics.push(Diagnostic::error(
                "E0229",
                format!(
                    "resolver module symbol '{expected_module}' has visibility public, expected private"
                ),
                span,
            ));
        }

        if let Some(actual) = symbol.import_source.as_deref() {
            self.diagnostics.push(Diagnostic::error(
                "E0230",
                format!("resolver module symbol '{expected_module}' has source '{actual}', expected none"),
                span,
            ));
        }

        if symbol.parameter_count.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0265",
                format!(
                    "resolver module symbol '{expected_module}' has parameter count metadata, expected none"
                ),
                span,
            ));
        }

        if symbol.return_type_name.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0266",
                format!(
                    "resolver module symbol '{expected_module}' has return type metadata, expected none"
                ),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.parameter_names.is_some(), "E0267", "parameter names"),
            (
                symbol.parameter_type_names.is_some(),
                "E0268",
                "parameter types",
            ),
            (
                symbol.parameter_types.is_some(),
                "E0371",
                "typed parameter types",
            ),
            (symbol.return_type.is_some(), "E0372", "typed return type"),
            (
                symbol.type_parameter_count.is_some(),
                "E0269",
                "type parameter count",
            ),
            (
                symbol.type_parameter_names.is_some(),
                "E0348",
                "type parameter names",
            ),
            (
                symbol.type_parameter_bounds.is_some(),
                "E0270",
                "type parameter bounds",
            ),
            (
                symbol.type_parameter_bound_refs.is_some(),
                "E0373",
                "typed type parameter bound refs",
            ),
            (symbol.field_count.is_some(), "E0271", "field count"),
            (symbol.field_type_names.is_some(), "E0272", "field types"),
            (symbol.field_types.is_some(), "E0374", "typed field types"),
            (symbol.variant_names.is_some(), "E0273", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0274",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0275",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0276",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0375",
                "typed variant payload type",
            ),
            (
                symbol.behavior_method_signatures.is_some(),
                "E0277",
                "behavior methods",
            ),
            (
                symbol.behavior_method_types.is_some(),
                "E0376",
                "typed behavior methods",
            ),
            (
                symbol.behavior_parent_names.is_some(),
                "E0278",
                "behavior parents",
            ),
            (
                symbol.behavior_parent_refs.is_some(),
                "E0377",
                "typed behavior parents",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0279",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0378",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0280",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0379",
                "typed behavior requires",
            ),
            (symbol.is_mutable.is_some(), "E0345", "mutability"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!(
                        "resolver module symbol '{expected_module}' has {label} metadata, expected none"
                    ),
                    span,
                ));
            }
        }
    }

    fn validate_stripped_resolver_import_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        if program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Declaration::Import { .. }))
        {
            return;
        }

        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            if symbol.is_public {
                self.diagnostics.push(Diagnostic::error(
                    "E0245",
                    format!(
                        "resolver import symbol '{}' has visibility public, expected private",
                        symbol.name
                    ),
                    symbol.definition_span,
                ));
            }
            if symbol.import_source.is_none() {
                self.diagnostics.push(Diagnostic::error(
                    "E0246",
                    format!(
                        "resolver import symbol '{}' has source 'unknown', expected a module source",
                        symbol.name
                    ),
                    symbol.definition_span,
                ));
            } else if let Some(source) = symbol.import_source.as_deref() {
                self.require_resolver_module_symbol(symbols, source, symbol.definition_span);
            }
            self.validate_resolver_import_absent_declaration_metadata(
                symbol,
                &symbol.name,
                symbol.definition_span,
            );
        }
    }

    fn collect_resolver_imports(&mut self, symbols: &SymbolTable) {
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            let Some(source) = &symbol.import_source else {
                continue;
            };
            self.imports
                .entry(symbol.name.clone())
                .or_insert_with(|| source.split('.').map(str::to_string).collect());
        }
    }

    fn collect_module_graph_imports(
        &mut self,
        graph: &ResolvedModuleGraph,
        entry: &ResolvedModule,
    ) {
        for binding in &entry.imports {
            let Some(source_module) = graph.module(binding.source_module) else {
                self.diagnostics.push(Diagnostic::error(
                    "E0233",
                    format!(
                        "module graph import '{}' points at missing module {:?}",
                        binding.local_name, binding.source_module
                    ),
                    binding.span,
                ));
                continue;
            };

            let Some(decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                self.diagnostics.push(Diagnostic::error(
                    "E0234",
                    format!(
                        "module graph import '{}' points at missing symbol '{}'",
                        binding.local_name, binding.source_symbol
                    ),
                    binding.span,
                ));
                continue;
            };

            self.seed_module_graph_import(binding.local_name.as_str(), decl);
            self.seed_imported_callable_signature_type_dependencies(decl, source_module, graph);
            self.seed_imported_generic_function_dependencies(
                binding.local_name.as_str(),
                decl,
                source_module,
                graph,
            );
            if matches!(decl, Declaration::Behavior { .. }) {
                self.seed_behavior_extends_for_imported_behavior(
                    binding.local_name.as_str(),
                    binding.source_symbol.as_str(),
                    source_module,
                    graph,
                );
            }
            self.seed_public_methods_for_imported_type(
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
            self.seed_behavior_impls_for_imported_type(
                binding.local_name.as_str(),
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
        }
    }

    fn seed_imported_callable_signature_type_dependencies(
        &mut self,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let mut type_names = HashSet::new();
        match decl {
            Declaration::Function {
                params,
                return_type,
                ..
            }
            | Declaration::Method {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    collect_ast_type_names(&param.ty, &mut type_names);
                }
                if let Some(return_type) = return_type {
                    collect_ast_type_names(return_type, &mut type_names);
                }
            }
            _ => return,
        }

        for type_name in type_names {
            self.seed_imported_type_dependency(&type_name, source_module, graph);
        }
    }

    fn seed_imported_type_dependency(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(type_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(type_name))
        {
            if !matches!(
                type_decl,
                Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
            ) {
                return;
            }
            self.seed_module_graph_import(type_name, type_decl);
            self.seed_public_methods_for_imported_type(type_name, source_module, graph);
            self.seed_behavior_impls_for_imported_type(type_name, type_name, source_module, graph);
            return;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == type_name)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };
        let Some(type_decl) = imported_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
        else {
            return;
        };
        if !matches!(
            type_decl,
            Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
        ) {
            return;
        }
        self.seed_module_graph_import(type_name, type_decl);
        self.seed_public_methods_for_imported_type(
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
        self.seed_behavior_impls_for_imported_type(
            type_name,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }

    fn seed_imported_generic_function_dependencies(
        &mut self,
        local_name: &str,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Declaration::Function { type_params, .. } = decl else {
            return;
        };
        if type_params.is_empty() {
            return;
        }
        let dependencies = Self::source_module_dependencies(source_module, graph);
        let Some(template) = self.generic_functions.get_mut(local_name) else {
            return;
        };
        Self::attach_template_dependencies(template, dependencies);
    }

    fn attach_template_dependencies(
        template: &mut GenericFunctionTemplate,
        dependencies: SourceModuleDependencies,
    ) {
        template.dependency_structs = dependencies.structs;
        template.dependency_enums = dependencies.enums;
        template.dependency_functions = dependencies.functions;
        template.dependency_generic_functions = dependencies.generic_functions;
        template.dependency_methods = dependencies.methods;
        template.dependency_generic_methods = dependencies.generic_methods;
    }

    fn seed_module_graph_import(&mut self, local_name: &str, decl: &Declaration) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.structs.insert(
                    local_name.to_string(),
                    StructInfo {
                        name: local_name.to_string(),
                        fields: fields
                            .iter()
                            .map(|field| (field.name.clone(), field.ty.clone()))
                            .collect(),
                        type_params: type_params.iter().map(|param| param.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    EnumInfo {
                        name: local_name.to_string(),
                        variants: variants
                            .iter()
                            .map(|variant| (variant.name.clone(), variant.payload.clone()))
                            .collect(),
                        type_params: type_params.iter().map(|param| param.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    BehaviorInfo {
                        name: local_name.to_string(),
                        type_params: type_params.iter().map(|param| param.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                        methods: methods.clone(),
                    },
                );
            }
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                let collected_type_params: Vec<String> =
                    type_params.iter().map(|param| param.name.clone()).collect();
                self.functions.insert(
                    local_name.to_string(),
                    FuncInfo {
                        name: local_name.to_string(),
                        params: params
                            .iter()
                            .map(|param| (param.name.clone(), param.ty.clone()))
                            .collect(),
                        return_type: return_type.clone().unwrap_or(AstType::Void),
                        type_params: collected_type_params.clone(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
                if !collected_type_params.is_empty() {
                    self.generic_functions.insert(
                        local_name.to_string(),
                        GenericFunctionTemplate::new(
                            collected_type_params,
                            params.clone(),
                            return_type.clone(),
                            body.clone(),
                            *span,
                        ),
                    );
                }
            }
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                let key = format!("{}.{}", type_name, method_name);
                let collected_type_params: Vec<String> =
                    type_params.iter().map(|param| param.name.clone()).collect();
                self.methods.insert(
                    key.clone(),
                    FuncInfo {
                        name: key.clone(),
                        params: params
                            .iter()
                            .map(|param| (param.name.clone(), param.ty.clone()))
                            .collect(),
                        return_type: return_type.clone().unwrap_or(AstType::Void),
                        type_params: collected_type_params.clone(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
                if !collected_type_params.is_empty() {
                    self.generic_methods.insert(
                        key,
                        GenericFunctionTemplate::new(
                            collected_type_params,
                            params.clone(),
                            return_type.clone(),
                            body.clone(),
                            *span,
                        ),
                    );
                }
            }
            _ => {}
        }
    }

    fn seed_behavior_extends_for_imported_behavior(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        self.seed_behavior_extends_for_imported_behavior_inner(
            local_name,
            source_name,
            source_module,
            graph,
            &mut HashSet::new(),
        );
    }

    fn seed_behavior_extends_for_imported_behavior_inner(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        seen: &mut HashSet<String>,
    ) {
        if !seen.insert(source_name.to_string()) {
            return;
        }

        for decl in &source_module.program.declarations {
            let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            else {
                continue;
            };
            if behavior != source_name {
                continue;
            }

            if let Some(parent_decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(parent.as_str()))
            {
                self.seed_module_graph_import(parent, parent_decl);
                self.seed_behavior_extends_for_imported_behavior_inner(
                    parent,
                    parent,
                    source_module,
                    graph,
                    seen,
                );
            } else if let Some(binding) = source_module
                .imports
                .iter()
                .find(|binding| binding.local_name == *parent)
            {
                if let Some(parent_module) = graph.module(binding.source_module) {
                    if let Some(parent_decl) = parent_module
                        .program
                        .declarations
                        .iter()
                        .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
                    {
                        self.seed_module_graph_import(parent, parent_decl);
                        self.seed_behavior_extends_for_imported_behavior_inner(
                            parent,
                            binding.source_symbol.as_str(),
                            parent_module,
                            graph,
                            seen,
                        );
                    }
                }
            }

            let parent_key = self.behavior_reference_key(parent, parent_type_args);
            let parents = self
                .behavior_extends
                .entry(local_name.to_string())
                .or_default();
            if parents.iter().any(|existing| existing.key == parent_key) {
                continue;
            }

            parents.push(BehaviorParentRef {
                behavior: parent.clone(),
                type_args: parent_type_args.clone(),
                key: parent_key,
            });
            self.behavior_extends_spans
                .entry(local_name.to_string())
                .or_insert(*span);
        }
    }

    fn seed_public_methods_for_imported_type(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let dependencies = Self::source_module_dependencies(source_module, graph);

        for decl in &source_module.program.declarations {
            let Declaration::Method {
                type_name: method_type,
                public,
                ..
            } = decl
            else {
                continue;
            };

            if method_type == type_name && *public {
                self.seed_imported_method_with_dependencies(type_name, decl, &dependencies);
            }
        }
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name: impl_type,
                behavior: None,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if impl_type != type_name {
                continue;
            }
            for method in methods {
                self.seed_imported_impl_method(type_name, method, true, &dependencies);
            }
        }
    }

    fn seed_behavior_impls_for_imported_type(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if type_name != source_name {
                continue;
            }
            if !self.imported_behavior_impl_is_public(behavior, source_module, graph) {
                continue;
            }

            self.seed_behavior_decl_for_imported_impl(behavior, behavior, source_module, graph);
            self.seed_behavior_decl_for_imported_impl_from_imports(behavior, source_module, graph);

            let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);
            self.behavior_impls
                .insert((local_name.to_string(), behavior_key));

            let dependencies = Self::source_module_dependencies(source_module, graph);
            for method in methods {
                self.seed_imported_impl_method(local_name, method, false, &dependencies);
            }
            for default in self.behavior_default_methods_for_impl(
                local_name,
                behavior,
                behavior_type_args,
                methods,
            ) {
                let key = format!("{}.{}", local_name, default.name);
                self.methods.insert(
                    key.clone(),
                    FuncInfo {
                        name: key,
                        params: default
                            .params
                            .iter()
                            .map(|param| (param.name.clone(), param.ty.clone()))
                            .collect(),
                        return_type: default.return_type.unwrap_or(AstType::Void),
                        type_params: Vec::new(),
                        type_param_bounds: HashMap::new(),
                    },
                );
            }
        }
    }

    fn imported_behavior_impl_is_public(
        &self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> bool {
        if let Some(Declaration::Behavior { public, .. }) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(behavior))
        {
            return *public;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return false;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return false;
        };
        matches!(
            imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str())),
            Some(Declaration::Behavior { public: true, .. })
        )
    }

    fn seed_behavior_decl_for_imported_impl(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(behavior_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(source_name))
        {
            self.seed_module_graph_import(local_name, behavior_decl);
            self.seed_behavior_extends_for_imported_behavior(
                local_name,
                source_name,
                source_module,
                graph,
            );
        }
    }

    fn seed_behavior_decl_for_imported_impl_from_imports(
        &mut self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };

        self.seed_behavior_decl_for_imported_impl(
            behavior,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }

    fn source_module_dependencies(
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> SourceModuleDependencies {
        let mut dependencies = SourceModuleDependencies::default();
        for binding in &source_module.imports {
            let Some(imported_module) = graph.module(binding.source_module) else {
                continue;
            };
            let Some(decl) = imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                continue;
            };
            Self::insert_source_import_dependency(&binding.local_name, decl, &mut dependencies);
            if matches!(decl, Declaration::Struct { .. } | Declaration::Enum { .. }) {
                Self::insert_source_import_type_method_dependencies(
                    &binding.local_name,
                    binding.source_symbol.as_str(),
                    imported_module,
                    graph,
                    &mut dependencies,
                );
            } else if matches!(
                decl,
                Declaration::Function { type_params, .. } if !type_params.is_empty()
            ) {
                let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
                if let Some(template) = dependencies
                    .generic_functions
                    .get_mut(binding.local_name.as_str())
                {
                    Self::attach_template_dependencies(template, nested_dependencies);
                }
            }
        }

        for decl in &source_module.program.declarations {
            match decl {
                Declaration::Struct { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Enum { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Function { name, .. } => {
                    Self::insert_source_function_dependency(
                        name,
                        decl,
                        &mut dependencies.functions,
                        &mut dependencies.generic_functions,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } => {
                    Self::insert_source_method_dependency(
                        &format!("{type_name}.{method_name}"),
                        decl,
                        &mut dependencies.methods,
                        &mut dependencies.generic_methods,
                    );
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    for method in methods {
                        if let Declaration::Function { name, .. } = method {
                            Self::insert_source_method_dependency(
                                &format!("{type_name}.{name}"),
                                method,
                                &mut dependencies.methods,
                                &mut dependencies.generic_methods,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        dependencies
    }

    fn insert_source_import_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
    ) {
        match decl {
            Declaration::Struct { .. } | Declaration::Enum { .. } => {
                Self::insert_source_type_dependency(local_name, decl, dependencies);
            }
            Declaration::Function { .. } => {
                Self::insert_source_function_dependency(
                    local_name,
                    decl,
                    &mut dependencies.functions,
                    &mut dependencies.generic_functions,
                );
            }
            _ => {}
        }
    }

    fn insert_source_type_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                dependencies.structs.insert(
                    local_name.to_string(),
                    StructInfo {
                        name: local_name.to_string(),
                        fields: fields
                            .iter()
                            .map(|field| (field.name.clone(), field.ty.clone()))
                            .collect(),
                        type_params: type_params.iter().map(|param| param.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                dependencies.enums.insert(
                    local_name.to_string(),
                    EnumInfo {
                        name: local_name.to_string(),
                        variants: variants
                            .iter()
                            .map(|variant| (variant.name.clone(), variant.payload.clone()))
                            .collect(),
                        type_params: type_params.iter().map(|param| param.name.clone()).collect(),
                        type_param_bounds: type_param_bounds(type_params),
                    },
                );
            }
            _ => {}
        }
    }

    fn insert_source_import_type_method_dependencies(
        local_name: &str,
        source_name: &str,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        for decl in &imported_module.program.declarations {
            match decl {
                Declaration::Method {
                    type_name,
                    method_name,
                    public,
                    ..
                } if type_name == source_name && *public => {
                    Self::insert_source_method_dependency(
                        &format!("{local_name}.{method_name}"),
                        decl,
                        &mut dependencies.methods,
                        &mut dependencies.generic_methods,
                    );
                    let key = format!("{local_name}.{method_name}");
                    if let Some(template) = dependencies.generic_methods.get_mut(&key) {
                        let nested_dependencies =
                            Self::source_module_dependencies(imported_module, graph);
                        Self::attach_template_dependencies(template, nested_dependencies);
                    }
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: None,
                    methods,
                    ..
                } if type_name == source_name => {
                    for method in methods {
                        let Declaration::Function { name, public, .. } = method else {
                            continue;
                        };
                        if !*public {
                            continue;
                        }
                        Self::insert_source_method_dependency(
                            &format!("{local_name}.{name}"),
                            method,
                            &mut dependencies.methods,
                            &mut dependencies.generic_methods,
                        );
                        let key = format!("{local_name}.{name}");
                        if let Some(template) = dependencies.generic_methods.get_mut(&key) {
                            let nested_dependencies =
                                Self::source_module_dependencies(imported_module, graph);
                            Self::attach_template_dependencies(template, nested_dependencies);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn insert_source_function_dependency(
        key: &str,
        decl: &Declaration,
        functions: &mut HashMap<String, FuncInfo>,
        generic_functions: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        let Declaration::Function {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return;
        };

        Self::insert_source_callable_dependency(
            ImportedMethodSignature {
                name: key,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            },
            functions,
            generic_functions,
        );
    }

    fn insert_source_method_dependency(
        key: &str,
        decl: &Declaration,
        methods: &mut HashMap<String, FuncInfo>,
        generic_methods: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        match decl {
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            }
            | Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => Self::insert_source_callable_dependency(
                ImportedMethodSignature {
                    name: key,
                    type_params,
                    params,
                    return_type,
                    body,
                    span: *span,
                },
                methods,
                generic_methods,
            ),
            _ => {}
        }
    }

    fn insert_source_callable_dependency(
        signature: ImportedMethodSignature<'_>,
        callables: &mut HashMap<String, FuncInfo>,
        generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        let collected_type_params: Vec<String> = signature
            .type_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        callables.insert(
            signature.name.to_string(),
            FuncInfo {
                name: signature.name.to_string(),
                params: signature
                    .params
                    .iter()
                    .map(|param| (param.name.clone(), param.ty.clone()))
                    .collect(),
                return_type: signature.return_type.clone().unwrap_or(AstType::Void),
                type_params: collected_type_params.clone(),
                type_param_bounds: type_param_bounds(signature.type_params),
            },
        );
        if !collected_type_params.is_empty() {
            generic_callables.insert(
                signature.name.to_string(),
                GenericFunctionTemplate::new(
                    collected_type_params,
                    signature.params.to_vec(),
                    signature.return_type.clone(),
                    signature.body.clone(),
                    signature.span,
                ),
            );
        }
    }

    fn seed_imported_method_with_dependencies(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Method {
            method_name,
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = method
        else {
            return;
        };

        self.seed_imported_method_signature(
            local_type_name,
            ImportedMethodSignature {
                name: method_name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            },
            ImportedMethodDependencies {
                structs: &dependencies.structs,
                enums: &dependencies.enums,
                functions: &dependencies.functions,
                generic_functions: &dependencies.generic_functions,
                methods: &dependencies.methods,
                generic_methods: &dependencies.generic_methods,
            },
        );
    }

    fn seed_imported_impl_method(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        public_only: bool,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
            public,
            ..
        } = method
        else {
            return;
        };
        if public_only && !*public {
            return;
        }

        self.seed_imported_method_signature(
            local_type_name,
            ImportedMethodSignature {
                name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            },
            ImportedMethodDependencies {
                structs: &dependencies.structs,
                enums: &dependencies.enums,
                functions: &dependencies.functions,
                generic_functions: &dependencies.generic_functions,
                methods: &dependencies.methods,
                generic_methods: &dependencies.generic_methods,
            },
        );
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        signature: ImportedMethodSignature<'_>,
        dependencies: ImportedMethodDependencies<'_>,
    ) {
        let key = format!("{}.{}", local_type_name, signature.name);
        let collected_type_params: Vec<String> = signature
            .type_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        self.methods.insert(
            key.clone(),
            FuncInfo {
                name: key.clone(),
                params: signature
                    .params
                    .iter()
                    .map(|param| (param.name.clone(), param.ty.clone()))
                    .collect(),
                return_type: signature.return_type.clone().unwrap_or(AstType::Void),
                type_params: collected_type_params.clone(),
                type_param_bounds: type_param_bounds(signature.type_params),
            },
        );
        if !collected_type_params.is_empty() {
            self.generic_methods.insert(
                key,
                GenericFunctionTemplate::new(
                    collected_type_params,
                    signature.params.to_vec(),
                    signature.return_type.clone(),
                    signature.body.clone(),
                    signature.span,
                )
                .with_dependencies(
                    dependencies.structs.clone(),
                    dependencies.enums.clone(),
                    dependencies.functions.clone(),
                    dependencies.generic_functions.clone(),
                    dependencies.methods.clone(),
                    dependencies.generic_methods.clone(),
                ),
            );
        }
    }

    fn require_resolver_symbol(
        &mut self,
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        let found = symbols.lookup(namespace, name).is_some()
            || matches!(namespace, Namespace::Type | Namespace::Behavior)
                && symbols.lookup(Namespace::Import, name).is_some();

        if !found {
            self.diagnostics.push(Diagnostic::error(
                "E0210",
                format!(
                    "resolver symbol table missing {} symbol '{}'",
                    namespace.diagnostic_name(),
                    name
                ),
                span,
            ));
        }
    }

    fn require_resolver_import_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected_source: &str,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Import, name) else {
            self.require_resolver_symbol(symbols, Namespace::Import, name, span);
            return;
        };

        if symbol.is_public {
            self.diagnostics.push(Diagnostic::error(
                "E0245",
                format!("resolver import symbol '{name}' has visibility public, expected private"),
                span,
            ));
        }

        if symbol.import_source.as_deref() != Some(expected_source) {
            let actual = symbol.import_source.as_deref().unwrap_or("unknown");
            self.diagnostics.push(Diagnostic::error(
                "E0227",
                format!(
                    "resolver import symbol '{name}' has source '{actual}', expected '{expected_source}'"
                ),
                span,
            ));
        }

        self.validate_resolver_import_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_import_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        if symbol.parameter_count.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0281",
                format!(
                    "resolver import symbol '{name}' has parameter count metadata, expected none"
                ),
                span,
            ));
        }

        if symbol.return_type_name.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0282",
                format!("resolver import symbol '{name}' has return type metadata, expected none"),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.parameter_names.is_some(), "E0283", "parameter names"),
            (
                symbol.parameter_type_names.is_some(),
                "E0284",
                "parameter types",
            ),
            (
                symbol.parameter_types.is_some(),
                "E0362",
                "typed parameter types",
            ),
            (symbol.return_type.is_some(), "E0363", "typed return type"),
            (
                symbol.type_parameter_count.is_some(),
                "E0285",
                "type parameter count",
            ),
            (
                symbol.type_parameter_names.is_some(),
                "E0349",
                "type parameter names",
            ),
            (
                symbol.type_parameter_bounds.is_some(),
                "E0286",
                "type parameter bounds",
            ),
            (
                symbol.type_parameter_bound_refs.is_some(),
                "E0364",
                "typed type parameter bound refs",
            ),
            (symbol.field_count.is_some(), "E0287", "field count"),
            (symbol.field_type_names.is_some(), "E0288", "field types"),
            (symbol.field_types.is_some(), "E0365", "typed field types"),
            (symbol.variant_names.is_some(), "E0289", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0290",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0291",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0292",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0366",
                "typed variant payload type",
            ),
            (
                symbol.behavior_method_signatures.is_some(),
                "E0293",
                "behavior methods",
            ),
            (
                symbol.behavior_method_types.is_some(),
                "E0367",
                "typed behavior methods",
            ),
            (
                symbol.behavior_parent_names.is_some(),
                "E0294",
                "behavior parents",
            ),
            (
                symbol.behavior_parent_refs.is_some(),
                "E0368",
                "typed behavior parents",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0295",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0369",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0296",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0370",
                "typed behavior requires",
            ),
            (symbol.is_mutable.is_some(), "E0344", "mutability"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver import symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }

    fn require_resolver_parameter_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        locals: &mut ResolverLocalScope,
    ) {
        for param in params {
            self.require_resolver_local_symbol(
                symbols,
                &param.name,
                param.mutable,
                locals.current_scope_id,
                param.span,
            );
            locals.insert(param.name.clone(), param.mutable);
        }
    }

    fn require_resolver_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.require_resolver_expr_locals(symbols, left, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, right, scope_cursor, locals);
            }
            Expression::UnaryOp { operand, .. } => {
                self.require_resolver_expr_locals(symbols, operand, scope_cursor, locals);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.require_resolver_expr_locals(symbols, receiver, scope_cursor, locals);
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, index, scope_cursor, locals);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.require_resolver_expr_locals(symbols, payload, scope_cursor, locals);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.require_resolver_expr_locals(symbols, element, scope_cursor, locals);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.require_resolver_expr_locals(symbols, scrutinee, scope_cursor, locals);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        let mut guard_locals = scope_cursor.child_scope(locals);
                        self.require_resolver_pattern_locals(
                            symbols,
                            &arm.pattern,
                            scope_cursor,
                            &mut guard_locals,
                        );
                        self.require_resolver_expr_locals(
                            symbols,
                            guard,
                            scope_cursor,
                            &mut guard_locals,
                        );
                    }
                    let mut arm_locals = scope_cursor.child_scope(locals);
                    self.require_resolver_pattern_locals(
                        symbols,
                        &arm.pattern,
                        scope_cursor,
                        &mut arm_locals,
                    );
                    self.require_resolver_expr_locals(
                        symbols,
                        &arm.body,
                        scope_cursor,
                        &mut arm_locals,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                let mut body_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut body_locals);
            }
            Expression::Loop { body, .. } => {
                let mut body_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut body_locals);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                let mut then_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(
                    symbols,
                    then_body,
                    scope_cursor,
                    &mut then_locals,
                );
                if let Some(else_body) = else_body {
                    let mut else_locals = scope_cursor.child_scope(locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        else_body,
                        scope_cursor,
                        &mut else_locals,
                    );
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                let mut block_locals = scope_cursor.child_scope(locals);
                for statement in statements {
                    self.require_resolver_statement_locals(
                        symbols,
                        statement,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
                if let Some(expr) = expr {
                    self.require_resolver_expr_locals(
                        symbols,
                        expr,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::Closure { params, body, .. } => {
                let mut closure_locals = scope_cursor.child_scope(locals);
                for param in params {
                    self.require_resolver_local_symbol(
                        symbols,
                        &param.name,
                        false,
                        closure_locals.current_scope_id,
                        param.span,
                    );
                    closure_locals.insert(param.name.clone(), false);
                }
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut closure_locals);
            }
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            Expression::Identifier { .. }
            | Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn require_resolver_statement_locals(
        &mut self,
        symbols: &SymbolTable,
        statement: &ast::Statement,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match statement {
            ast::Statement::VarDecl {
                name,
                value,
                mutable,
                span,
                constant,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                if *constant || *mutable || !locals.is_mutable(name) {
                    self.require_resolver_local_symbol(
                        symbols,
                        name,
                        *mutable,
                        locals.current_scope_id,
                        *span,
                    );
                    locals.insert(name.clone(), *mutable);
                }
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.require_resolver_expr_locals(symbols, target, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Statement::Expression { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            ast::Statement::Block { stmts, .. } => {
                let mut block_locals = scope_cursor.child_scope(locals);
                for statement in stmts {
                    self.require_resolver_statement_locals(
                        symbols,
                        statement,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
            }
        }
    }

    fn require_resolver_pattern_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match pattern {
            ast::Pattern::Identifier { name, span } => {
                self.require_resolver_local_symbol(
                    symbols,
                    name,
                    false,
                    locals.current_scope_id,
                    *span,
                );
                locals.insert(name.clone(), false);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested, scope_cursor, locals);
                    } else {
                        self.require_resolver_local_symbol(
                            symbols,
                            name,
                            false,
                            locals.current_scope_id,
                            *span,
                        );
                        locals.insert(name.clone(), false);
                    }
                }
            }
            ast::Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.require_resolver_pattern_locals(symbols, payload, scope_cursor, locals);
            }
            ast::Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, locals);
                }
            }
            ast::Pattern::Literal { value, .. } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Pattern::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            ast::Pattern::Wildcard { .. }
            | ast::Pattern::Enum { payload: None, .. }
            | ast::Pattern::BoolTrue { .. }
            | ast::Pattern::BoolFalse { .. } => {}
        }
    }

    fn require_resolver_local_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected_mutable: bool,
        scope_id: u32,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup_in_scope(Namespace::Local, name, scope_id) else {
            self.diagnostics.push(Diagnostic::error(
                "E0228",
                format!("resolver symbol table missing local symbol '{name}'"),
                span,
            ));
            return;
        };

        if symbol.is_mutable != Some(expected_mutable) {
            let actual = match symbol.is_mutable {
                Some(true) => "mutable",
                Some(false) => "immutable",
                None => "unknown",
            };
            let expected = if expected_mutable {
                "mutable"
            } else {
                "immutable"
            };
            self.diagnostics.push(Diagnostic::error(
                "E0231",
                format!(
                    "resolver local symbol '{name}' has mutability {actual}, expected {expected}"
                ),
                span,
            ));
        }

        if symbol.is_public {
            self.diagnostics.push(Diagnostic::error(
                "E0247",
                format!("resolver local symbol '{name}' has visibility public, expected private"),
                span,
            ));
        }

        if let Some(actual) = symbol.import_source.as_deref() {
            self.diagnostics.push(Diagnostic::error(
                "E0248",
                format!("resolver local symbol '{name}' has source '{actual}', expected none"),
                span,
            ));
        }

        if symbol.parameter_count.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0249",
                format!(
                    "resolver local symbol '{name}' has parameter count metadata, expected none"
                ),
                span,
            ));
        }

        if symbol.return_type_name.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0250",
                format!("resolver local symbol '{name}' has return type metadata, expected none"),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.parameter_names.is_some(), "E0251", "parameter names"),
            (
                symbol.parameter_type_names.is_some(),
                "E0252",
                "parameter types",
            ),
            (
                symbol.parameter_types.is_some(),
                "E0380",
                "typed parameter types",
            ),
            (symbol.return_type.is_some(), "E0381", "typed return type"),
            (
                symbol.type_parameter_count.is_some(),
                "E0253",
                "type parameter count",
            ),
            (
                symbol.type_parameter_names.is_some(),
                "E0350",
                "type parameter names",
            ),
            (
                symbol.type_parameter_bounds.is_some(),
                "E0254",
                "type parameter bounds",
            ),
            (
                symbol.type_parameter_bound_refs.is_some(),
                "E0382",
                "typed type parameter bound refs",
            ),
            (symbol.field_count.is_some(), "E0255", "field count"),
            (symbol.field_type_names.is_some(), "E0256", "field types"),
            (symbol.field_types.is_some(), "E0383", "typed field types"),
            (symbol.variant_names.is_some(), "E0257", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0258",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0259",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0260",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0384",
                "typed variant payload type",
            ),
            (
                symbol.behavior_method_signatures.is_some(),
                "E0261",
                "behavior methods",
            ),
            (
                symbol.behavior_method_types.is_some(),
                "E0385",
                "typed behavior methods",
            ),
            (
                symbol.behavior_parent_names.is_some(),
                "E0262",
                "behavior parents",
            ),
            (
                symbol.behavior_parent_refs.is_some(),
                "E0386",
                "typed behavior parents",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0263",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0387",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0264",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0388",
                "typed behavior requires",
            ),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver local symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }

    fn require_resolver_type_like_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        expected: ExpectedTypeLikeSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            self.require_resolver_symbol(symbols, namespace, name, span);
            return None;
        };

        if let Some(expected_is_public) = expected.is_public {
            if symbol.is_public != expected_is_public {
                self.diagnostics.push(Diagnostic::error(
                    "E0225",
                    format!(
                        "resolver {} symbol '{name}' has visibility {}, expected {}",
                        namespace.diagnostic_name(),
                        visibility_name(symbol.is_public),
                        visibility_name(expected_is_public)
                    ),
                    span,
                ));
            }
        }

        if symbol.type_parameter_count != Some(expected.type_parameter_count) {
            let actual = symbol
                .type_parameter_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0213",
                format!(
                    "resolver {} symbol '{name}' has type parameter count {actual}, expected {expected_type_parameter_count}",
                    namespace.diagnostic_name(),
                    expected_type_parameter_count = expected.type_parameter_count
                ),
                span,
            ));
        }

        if symbol.type_parameter_names.as_deref() != Some(expected.type_parameter_names.as_slice())
        {
            let actual = format_type_parameter_names(symbol.type_parameter_names.as_deref());
            let expected = format_type_parameter_names(Some(&expected.type_parameter_names));
            self.diagnostics.push(Diagnostic::error(
                "E0346",
                format!(
                    "resolver {} symbol '{name}' has type parameter names '{actual}', expected '{expected}'",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }

        if symbol.type_parameter_bounds.as_deref()
            != Some(expected.type_parameter_bounds.as_slice())
        {
            let actual = format_type_parameter_bounds(symbol.type_parameter_bounds.as_deref());
            let expected = format_type_parameter_bounds(Some(&expected.type_parameter_bounds));
            self.diagnostics.push(Diagnostic::error(
                "E0222",
                format!(
                    "resolver {} symbol '{name}' has type parameter bounds '{actual}', expected '{expected}'",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }
        if symbol.type_parameter_bound_refs.as_deref()
            != Some(expected.type_parameter_bound_refs.as_slice())
        {
            let actual =
                format_type_parameter_bound_refs(symbol.type_parameter_bound_refs.as_deref());
            let expected =
                format_type_parameter_bound_refs(Some(&expected.type_parameter_bound_refs));
            self.diagnostics.push(Diagnostic::error(
                "E0350",
                format!(
                    "resolver {} symbol '{name}' has type parameter bound refs '{actual}', expected '{expected}'",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }

        self.validate_resolver_type_like_absent_value_metadata(symbol, namespace, name, span);

        Some(symbol)
    }

    fn validate_resolver_type_like_absent_value_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        if let Some(actual) = symbol.import_source.as_deref() {
            self.diagnostics.push(Diagnostic::error(
                "E0309",
                format!(
                    "resolver {} symbol '{name}' has source '{actual}', expected none",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }

        if symbol.parameter_count.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0310",
                format!(
                    "resolver {} symbol '{name}' has parameter count metadata, expected none",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }

        if symbol.return_type_name.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0311",
                format!(
                    "resolver {} symbol '{name}' has return type metadata, expected none",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.parameter_names.is_some(), "E0312", "parameter names"),
            (
                symbol.parameter_type_names.is_some(),
                "E0313",
                "parameter types",
            ),
            (
                symbol.parameter_types.is_some(),
                "E0360",
                "typed parameter types",
            ),
            (symbol.return_type.is_some(), "E0361", "typed return type"),
            (symbol.is_mutable.is_some(), "E0314", "mutability"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!(
                        "resolver {} symbol '{name}' has {label} metadata, expected none",
                        namespace.diagnostic_name()
                    ),
                    span,
                ));
            }
        }
    }

    fn validate_resolver_field_count(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        expected_field_count: usize,
        span: Span,
    ) {
        if symbol.field_count != Some(expected_field_count) {
            let actual = symbol
                .field_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0214",
                format!(
                    "resolver {} symbol '{name}' has field count {actual}, expected {expected_field_count}",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }
    }

    fn validate_resolver_field_types(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        expected_field_types: Vec<(String, AstType)>,
        expected_field_type_names: Vec<(String, String)>,
        span: Span,
    ) {
        if symbol.field_types.as_deref() != Some(expected_field_types.as_slice()) {
            let actual = format_field_types(symbol.field_types.as_deref());
            let expected = format_field_types(Some(&expected_field_types));
            self.diagnostics.push(Diagnostic::error(
                "E0358",
                format!(
                    "resolver {} symbol '{name}' has typed fields '{actual}', expected '{expected}'",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }
        if symbol.field_type_names.as_deref() != Some(expected_field_type_names.as_slice()) {
            let actual = format_field_type_names(symbol.field_type_names.as_deref());
            let expected = format_field_type_names(Some(&expected_field_type_names));
            self.diagnostics.push(Diagnostic::error(
                "E0217",
                format!(
                    "resolver {} symbol '{name}' has fields '{actual}', expected '{expected}'",
                    namespace.diagnostic_name()
                ),
                span,
            ));
        }
    }

    fn validate_resolver_variant_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_variant_names: Vec<String>,
        span: Span,
    ) {
        if symbol.variant_names.as_deref() != Some(expected_variant_names.as_slice()) {
            let actual = format_variant_names(symbol.variant_names.as_deref());
            let expected = format_variant_names(Some(&expected_variant_names));
            self.diagnostics.push(Diagnostic::error(
                "E0241",
                format!(
                    "resolver type symbol '{name}' has variants '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_struct_absent_enum_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        for (present, code, label) in [
            (symbol.variant_names.is_some(), "E0315", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0316",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0317",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0318",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0397",
                "typed variant payload type",
            ),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver type symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }

    fn validate_resolver_enum_absent_struct_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        for (present, code, label) in [
            (symbol.field_count.is_some(), "E0319", "field count"),
            (symbol.field_type_names.is_some(), "E0320", "field types"),
            (symbol.field_types.is_some(), "E0398", "typed field types"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver type symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }

    fn validate_resolver_variant_payload_count(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_payload_count: usize,
        span: Span,
    ) {
        if symbol.variant_payload_count != Some(expected_payload_count) {
            let actual = symbol
                .variant_payload_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0215",
                format!(
                    "resolver variant symbol '{name}' has payload count {actual}, expected {expected_payload_count}"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_variant_owner_name(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_owner_name: &str,
        span: Span,
    ) {
        if symbol.variant_owner_name.as_deref() != Some(expected_owner_name) {
            let actual = symbol.variant_owner_name.as_deref().unwrap_or("unknown");
            self.diagnostics.push(Diagnostic::error(
                "E0242",
                format!(
                    "resolver variant symbol '{name}' has owner '{actual}', expected '{expected_owner_name}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_variant_visibility(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_is_public: bool,
        span: Span,
    ) {
        if symbol.is_public != expected_is_public {
            self.diagnostics.push(Diagnostic::error(
                "E0226",
                format!(
                    "resolver variant symbol '{name}' has visibility {}, expected {}",
                    visibility_name(symbol.is_public),
                    visibility_name(expected_is_public)
                ),
                span,
            ));
        }
    }

    fn validate_resolver_variant_payload_type(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_payload_type: Option<AstType>,
        expected_payload_type_name: Option<String>,
        span: Span,
    ) {
        if symbol.variant_payload_type != expected_payload_type {
            let actual = symbol
                .variant_payload_type
                .as_ref()
                .map(AstType::display_name)
                .unwrap_or_else(|| "none".to_string());
            let expected = expected_payload_type
                .as_ref()
                .map(AstType::display_name)
                .unwrap_or_else(|| "none".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0359",
                format!(
                    "resolver variant symbol '{name}' has typed payload type '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if symbol.variant_payload_type_name != expected_payload_type_name {
            let actual = symbol
                .variant_payload_type_name
                .as_deref()
                .unwrap_or("unknown");
            let expected = expected_payload_type_name.as_deref().unwrap_or("none");
            self.diagnostics.push(Diagnostic::error(
                "E0218",
                format!(
                    "resolver variant symbol '{name}' has payload type '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_variant_absent_other_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        if let Some(actual) = symbol.import_source.as_deref() {
            self.diagnostics.push(Diagnostic::error(
                "E0329",
                format!("resolver variant symbol '{name}' has source '{actual}', expected none"),
                span,
            ));
        }

        if symbol.parameter_count.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0330",
                format!(
                    "resolver variant symbol '{name}' has parameter count metadata, expected none"
                ),
                span,
            ));
        }

        if symbol.return_type_name.is_some() {
            self.diagnostics.push(Diagnostic::error(
                "E0331",
                format!("resolver variant symbol '{name}' has return type metadata, expected none"),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.parameter_names.is_some(), "E0332", "parameter names"),
            (
                symbol.parameter_type_names.is_some(),
                "E0333",
                "parameter types",
            ),
            (
                symbol.parameter_types.is_some(),
                "E0389",
                "typed parameter types",
            ),
            (symbol.return_type.is_some(), "E0390", "typed return type"),
            (
                symbol.type_parameter_count.is_some(),
                "E0334",
                "type parameter count",
            ),
            (
                symbol.type_parameter_names.is_some(),
                "E0351",
                "type parameter names",
            ),
            (
                symbol.type_parameter_bounds.is_some(),
                "E0335",
                "type parameter bounds",
            ),
            (
                symbol.type_parameter_bound_refs.is_some(),
                "E0391",
                "typed type parameter bound refs",
            ),
            (symbol.field_count.is_some(), "E0336", "field count"),
            (symbol.field_type_names.is_some(), "E0337", "field types"),
            (symbol.field_types.is_some(), "E0392", "typed field types"),
            (symbol.variant_names.is_some(), "E0338", "variant names"),
            (
                symbol.behavior_method_signatures.is_some(),
                "E0339",
                "behavior methods",
            ),
            (
                symbol.behavior_method_types.is_some(),
                "E0393",
                "typed behavior methods",
            ),
            (
                symbol.behavior_parent_names.is_some(),
                "E0340",
                "behavior parents",
            ),
            (
                symbol.behavior_parent_refs.is_some(),
                "E0394",
                "typed behavior parents",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0341",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0395",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0342",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0396",
                "typed behavior requires",
            ),
            (symbol.is_mutable.is_some(), "E0343", "mutability"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver variant symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }

    fn validate_resolver_behavior_method_signatures(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_method_signatures: Vec<MethodSignatureMetadata>,
        expected_method_types: Vec<BehaviorMethodTypeMetadata>,
        span: Span,
    ) {
        if symbol.behavior_method_signatures.as_deref()
            != Some(expected_method_signatures.as_slice())
        {
            let actual =
                format_behavior_method_signatures(symbol.behavior_method_signatures.as_deref());
            let expected = format_behavior_method_signatures(Some(&expected_method_signatures));
            self.diagnostics.push(Diagnostic::error(
                "E0219",
                format!(
                    "resolver behavior symbol '{name}' has methods '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if symbol.behavior_method_types.as_deref() != Some(expected_method_types.as_slice()) {
            let actual = format_behavior_method_types(symbol.behavior_method_types.as_deref());
            let expected = format_behavior_method_types(Some(&expected_method_types));
            self.diagnostics.push(Diagnostic::error(
                "E0355",
                format!(
                    "resolver behavior symbol '{name}' has typed methods '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_absent_type_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        for (present, code, label) in [
            (symbol.field_count.is_some(), "E0321", "field count"),
            (symbol.field_type_names.is_some(), "E0322", "field types"),
            (symbol.field_types.is_some(), "E0399", "typed field types"),
            (symbol.variant_names.is_some(), "E0323", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0324",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0325",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0326",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0400",
                "typed variant payload type",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0327",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0401",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0328",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0402",
                "typed behavior requires",
            ),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!(
                        "resolver behavior symbol '{name}' has {label} metadata, expected none"
                    ),
                    span,
                ));
            }
        }
    }

    fn validate_resolver_behavior_parent_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_parent: &str,
        expected_parent_ref: &BehaviorRefMetadata,
        span: Span,
    ) {
        if !symbol
            .behavior_parent_names
            .as_deref()
            .is_some_and(|parents| parents.iter().any(|parent| parent == expected_parent))
        {
            let actual = format_behavior_parent_names(symbol.behavior_parent_names.as_deref());
            self.diagnostics.push(Diagnostic::error(
                "E0235",
                format!(
                    "resolver behavior symbol '{name}' has parents '{actual}', expected to include '{expected_parent}'"
                ),
                span,
            ));
        }
        if !symbol
            .behavior_parent_refs
            .as_deref()
            .is_some_and(|parents| parents.iter().any(|parent| parent == expected_parent_ref))
        {
            let actual = format_behavior_refs(symbol.behavior_parent_refs.as_deref());
            let expected =
                behavior_ref_display(&expected_parent_ref.name, &expected_parent_ref.type_args);
            self.diagnostics.push(Diagnostic::error(
                "E0245",
                format!(
                    "resolver behavior symbol '{name}' has parent refs '{actual}', expected to include '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_parent_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_parents: &[String],
        expected_parent_refs: &[BehaviorRefMetadata],
        span: Span,
    ) {
        if !behavior_ref_names_match(symbol.behavior_parent_names.as_deref(), expected_parents) {
            let actual = format_behavior_parent_names(symbol.behavior_parent_names.as_deref());
            let expected = format_behavior_parent_names(Some(expected_parents));
            self.diagnostics.push(Diagnostic::error(
                "E0240",
                format!(
                    "resolver behavior symbol '{name}' has parents '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if !behavior_refs_match(symbol.behavior_parent_refs.as_deref(), expected_parent_refs) {
            let actual = format_behavior_refs(symbol.behavior_parent_refs.as_deref());
            let expected = format_behavior_refs(Some(expected_parent_refs));
            self.diagnostics.push(Diagnostic::error(
                "E0246",
                format!(
                    "resolver behavior symbol '{name}' has parent refs '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_impl_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_impl: &str,
        expected_impl_ref: &BehaviorRefMetadata,
        span: Span,
    ) {
        if !symbol
            .behavior_impl_names
            .as_deref()
            .is_some_and(|impls| impls.iter().any(|behavior| behavior == expected_impl))
        {
            let actual = format_behavior_ref_names(symbol.behavior_impl_names.as_deref());
            self.diagnostics.push(Diagnostic::error(
                "E0236",
                format!(
                    "resolver type symbol '{name}' has behavior impls '{actual}', expected to include '{expected_impl}'"
                ),
                span,
            ));
        }
        if !symbol
            .behavior_impl_refs
            .as_deref()
            .is_some_and(|impls| impls.iter().any(|behavior| behavior == expected_impl_ref))
        {
            let actual = format_behavior_refs(symbol.behavior_impl_refs.as_deref());
            let expected =
                behavior_ref_display(&expected_impl_ref.name, &expected_impl_ref.type_args);
            self.diagnostics.push(Diagnostic::error(
                "E0247",
                format!(
                    "resolver type symbol '{name}' has behavior impl refs '{actual}', expected to include '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_impl_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_impls: &[String],
        expected_impl_refs: &[BehaviorRefMetadata],
        span: Span,
    ) {
        if !behavior_ref_names_match(symbol.behavior_impl_names.as_deref(), expected_impls) {
            let actual = format_behavior_ref_names(symbol.behavior_impl_names.as_deref());
            let expected = format_behavior_ref_names(Some(expected_impls));
            self.diagnostics.push(Diagnostic::error(
                "E0238",
                format!(
                    "resolver type symbol '{name}' has behavior impls '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if !behavior_refs_match(symbol.behavior_impl_refs.as_deref(), expected_impl_refs) {
            let actual = format_behavior_refs(symbol.behavior_impl_refs.as_deref());
            let expected = format_behavior_refs(Some(expected_impl_refs));
            self.diagnostics.push(Diagnostic::error(
                "E0248",
                format!(
                    "resolver type symbol '{name}' has behavior impl refs '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_required_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_required: &str,
        expected_required_ref: &BehaviorRefMetadata,
        span: Span,
    ) {
        if !symbol
            .behavior_required_names
            .as_deref()
            .is_some_and(|required| {
                required
                    .iter()
                    .any(|behavior| behavior == expected_required)
            })
        {
            let actual = format_behavior_ref_names(symbol.behavior_required_names.as_deref());
            self.diagnostics.push(Diagnostic::error(
                "E0237",
                format!(
                    "resolver type symbol '{name}' has behavior requires '{actual}', expected to include '{expected_required}'"
                ),
                span,
            ));
        }
        if !symbol
            .behavior_required_refs
            .as_deref()
            .is_some_and(|required| {
                required
                    .iter()
                    .any(|behavior| behavior == expected_required_ref)
            })
        {
            let actual = format_behavior_refs(symbol.behavior_required_refs.as_deref());
            let expected = behavior_ref_display(
                &expected_required_ref.name,
                &expected_required_ref.type_args,
            );
            self.diagnostics.push(Diagnostic::error(
                "E0249",
                format!(
                    "resolver type symbol '{name}' has behavior requires refs '{actual}', expected to include '{expected}'"
                ),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_required_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_required: &[String],
        expected_required_refs: &[BehaviorRefMetadata],
        span: Span,
    ) {
        if !behavior_ref_names_match(symbol.behavior_required_names.as_deref(), expected_required) {
            let actual = format_behavior_ref_names(symbol.behavior_required_names.as_deref());
            let expected = format_behavior_ref_names(Some(expected_required));
            self.diagnostics.push(Diagnostic::error(
                "E0239",
                format!(
                    "resolver type symbol '{name}' has behavior requires '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if !behavior_refs_match(
            symbol.behavior_required_refs.as_deref(),
            expected_required_refs,
        ) {
            let actual = format_behavior_refs(symbol.behavior_required_refs.as_deref());
            let expected = format_behavior_refs(Some(expected_required_refs));
            self.diagnostics.push(Diagnostic::error(
                "E0250",
                format!(
                    "resolver type symbol '{name}' has behavior requires refs '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
    }

    fn require_resolver_value_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected_signature: ExpectedValueSignature,
        expected_is_public: bool,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.require_resolver_symbol(symbols, Namespace::Value, name, span);
            return;
        };

        if symbol.is_public != expected_is_public {
            self.diagnostics.push(Diagnostic::error(
                "E0224",
                format!(
                    "resolver value symbol '{name}' has visibility {}, expected {}",
                    visibility_name(symbol.is_public),
                    visibility_name(expected_is_public)
                ),
                span,
            ));
        }

        let expected_parameter_count = expected_signature.parameter_type_names.len();
        if symbol.parameter_count != Some(expected_parameter_count) {
            let actual = symbol
                .parameter_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0211",
                format!(
                    "resolver value symbol '{name}' has parameter count {actual}, expected {expected_parameter_count}"
                ),
                span,
            ));
        }

        if symbol.parameter_names.as_deref() != Some(expected_signature.parameter_names.as_slice())
        {
            let actual = format_parameter_names(symbol.parameter_names.as_deref());
            let expected = format_parameter_names(Some(&expected_signature.parameter_names));
            self.diagnostics.push(Diagnostic::error(
                "E0223",
                format!(
                    "resolver value symbol '{name}' has parameter names '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }

        if symbol.parameter_type_names.as_deref()
            != Some(expected_signature.parameter_type_names.as_slice())
        {
            let actual = format_parameter_type_names(symbol.parameter_type_names.as_deref());
            let expected =
                format_parameter_type_names(Some(&expected_signature.parameter_type_names));
            self.diagnostics.push(Diagnostic::error(
                "E0216",
                format!(
                    "resolver value symbol '{name}' has parameter types '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if symbol.parameter_types.as_deref() != Some(expected_signature.parameter_types.as_slice())
        {
            let actual = format_ast_type_list(symbol.parameter_types.as_deref());
            let expected = format_ast_type_list(Some(&expected_signature.parameter_types));
            self.diagnostics.push(Diagnostic::error(
                "E0356",
                format!(
                    "resolver value symbol '{name}' has typed parameter types '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }

        if symbol.return_type_name.as_deref() != Some(expected_signature.return_type_name.as_str())
        {
            let actual = symbol.return_type_name.as_deref().unwrap_or("unknown");
            self.diagnostics.push(Diagnostic::error(
                "E0212",
                format!(
                    "resolver value symbol '{name}' has return type '{actual}', expected '{}'",
                    expected_signature.return_type_name
                ),
                span,
            ));
        }
        if symbol.return_type.as_ref() != Some(&expected_signature.return_type) {
            let actual = symbol
                .return_type
                .as_ref()
                .map(AstType::display_name)
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0357",
                format!(
                    "resolver value symbol '{name}' has typed return type '{actual}', expected '{}'",
                    expected_signature.return_type.display_name()
                ),
                span,
            ));
        }

        if symbol.type_parameter_count != Some(expected_signature.type_parameter_count) {
            let actual = symbol
                .type_parameter_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            self.diagnostics.push(Diagnostic::error(
                "E0220",
                format!(
                    "resolver value symbol '{name}' has type parameter count {actual}, expected {}",
                    expected_signature.type_parameter_count
                ),
                span,
            ));
        }

        if symbol.type_parameter_names.as_deref()
            != Some(expected_signature.type_parameter_names.as_slice())
        {
            let actual = format_type_parameter_names(symbol.type_parameter_names.as_deref());
            let expected =
                format_type_parameter_names(Some(&expected_signature.type_parameter_names));
            self.diagnostics.push(Diagnostic::error(
                "E0347",
                format!(
                    "resolver value symbol '{name}' has type parameter names '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }

        if symbol.type_parameter_bounds.as_deref()
            != Some(expected_signature.type_parameter_bounds.as_slice())
        {
            let actual = format_type_parameter_bounds(symbol.type_parameter_bounds.as_deref());
            let expected =
                format_type_parameter_bounds(Some(&expected_signature.type_parameter_bounds));
            self.diagnostics.push(Diagnostic::error(
                "E0221",
                format!(
                    "resolver value symbol '{name}' has type parameter bounds '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }
        if symbol.type_parameter_bound_refs.as_deref()
            != Some(expected_signature.type_parameter_bound_refs.as_slice())
        {
            let actual =
                format_type_parameter_bound_refs(symbol.type_parameter_bound_refs.as_deref());
            let expected = format_type_parameter_bound_refs(Some(
                &expected_signature.type_parameter_bound_refs,
            ));
            self.diagnostics.push(Diagnostic::error(
                "E0351",
                format!(
                    "resolver value symbol '{name}' has type parameter bound refs '{actual}', expected '{expected}'"
                ),
                span,
            ));
        }

        self.validate_resolver_value_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_value_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        if let Some(actual) = symbol.import_source.as_deref() {
            self.diagnostics.push(Diagnostic::error(
                "E0297",
                format!("resolver value symbol '{name}' has source '{actual}', expected none"),
                span,
            ));
        }

        for (present, code, label) in [
            (symbol.field_count.is_some(), "E0298", "field count"),
            (symbol.field_type_names.is_some(), "E0299", "field types"),
            (symbol.field_types.is_some(), "E0403", "typed field types"),
            (symbol.variant_names.is_some(), "E0300", "variant names"),
            (
                symbol.variant_owner_name.is_some(),
                "E0301",
                "variant owner",
            ),
            (
                symbol.variant_payload_count.is_some(),
                "E0302",
                "variant payload count",
            ),
            (
                symbol.variant_payload_type_name.is_some(),
                "E0303",
                "variant payload type",
            ),
            (
                symbol.variant_payload_type.is_some(),
                "E0404",
                "typed variant payload type",
            ),
            (
                symbol.behavior_method_signatures.is_some(),
                "E0304",
                "behavior methods",
            ),
            (
                symbol.behavior_method_types.is_some(),
                "E0405",
                "typed behavior methods",
            ),
            (
                symbol.behavior_parent_names.is_some(),
                "E0305",
                "behavior parents",
            ),
            (
                symbol.behavior_parent_refs.is_some(),
                "E0406",
                "typed behavior parents",
            ),
            (
                symbol.behavior_impl_names.is_some(),
                "E0306",
                "behavior impls",
            ),
            (
                symbol.behavior_impl_refs.is_some(),
                "E0407",
                "typed behavior impls",
            ),
            (
                symbol.behavior_required_names.is_some(),
                "E0307",
                "behavior requires",
            ),
            (
                symbol.behavior_required_refs.is_some(),
                "E0408",
                "typed behavior requires",
            ),
            (symbol.is_mutable.is_some(), "E0308", "mutability"),
        ] {
            if present {
                self.diagnostics.push(Diagnostic::error(
                    code,
                    format!("resolver value symbol '{name}' has {label} metadata, expected none"),
                    span,
                ));
            }
        }
    }
}

fn expected_return_type_name(return_type: &Option<AstType>) -> String {
    return_type
        .as_ref()
        .unwrap_or(&AstType::Void)
        .display_name()
}

fn visibility_name(is_public: bool) -> &'static str {
    if is_public {
        "public"
    } else {
        "private"
    }
}

fn expected_parameter_type_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.ty.display_name()).collect()
}

fn expected_parameter_types(params: &[Param]) -> Vec<AstType> {
    params.iter().map(|param| param.ty.clone()).collect()
}

fn expected_parameter_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

fn expected_value_signature(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
) -> ExpectedValueSignature {
    ExpectedValueSignature {
        parameter_names: expected_parameter_names(params),
        parameter_types: expected_parameter_types(params),
        parameter_type_names: expected_parameter_type_names(params),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        return_type_name: expected_return_type_name(return_type),
        type_parameter_count: type_params.len(),
        type_parameter_names: expected_type_parameter_names(type_params),
        type_parameter_bounds: expected_type_parameter_bounds(type_params),
        type_parameter_bound_refs: expected_type_parameter_bound_refs(type_params),
    }
}

fn expected_type_parameter_names(type_params: &[ast::TypeParam]) -> Vec<String> {
    type_params
        .iter()
        .map(|type_param| type_param.name.clone())
        .collect()
}

fn expected_type_parameter_bounds(
    type_params: &[ast::TypeParam],
) -> Vec<TypeParameterBoundMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param_bound_display(type_param)
                .map(|constraint| (type_param.name.clone(), constraint))
        })
        .collect()
}

fn expected_type_parameter_bound_refs(
    type_params: &[ast::TypeParam],
) -> Vec<TypeParameterBoundRefMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param
                .constraint
                .as_ref()
                .map(|behavior| TypeParameterBoundRefMetadata {
                    type_parameter: type_param.name.clone(),
                    behavior: behavior.clone(),
                    type_args: type_param.constraint_type_args.clone(),
                })
        })
        .collect()
}

fn expected_type_like_symbol(
    type_params: &[ast::TypeParam],
    is_public: Option<bool>,
) -> ExpectedTypeLikeSymbol {
    ExpectedTypeLikeSymbol {
        type_parameter_count: type_params.len(),
        type_parameter_names: expected_type_parameter_names(type_params),
        type_parameter_bounds: expected_type_parameter_bounds(type_params),
        type_parameter_bound_refs: expected_type_parameter_bound_refs(type_params),
        is_public,
    }
}

fn format_type_parameter_names(names: Option<&[String]>) -> String {
    match names {
        Some(names) => format!("({})", names.join(", ")),
        None => "unknown".to_string(),
    }
}

fn format_type_parameter_bounds(bounds: Option<&[TypeParameterBoundMetadata]>) -> String {
    match bounds {
        Some(bounds) => format!(
            "({})",
            bounds
                .iter()
                .map(|(name, behavior)| format!("{name}: {behavior}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_type_parameter_bound_refs(bounds: Option<&[TypeParameterBoundRefMetadata]>) -> String {
    match bounds {
        Some(bounds) => format!(
            "({})",
            bounds
                .iter()
                .map(|bound| format!(
                    "{}: {}",
                    bound.type_parameter,
                    behavior_ref_display(&bound.behavior, &bound.type_args)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_parameter_type_names(names: Option<&[String]>) -> String {
    match names {
        Some(names) => format!("({})", names.join(", ")),
        None => "unknown".to_string(),
    }
}

fn format_ast_type_list(types: Option<&[AstType]>) -> String {
    match types {
        Some(types) => format!(
            "({})",
            types
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_parameter_names(names: Option<&[String]>) -> String {
    match names {
        Some(names) => format!("({})", names.join(", ")),
        None => "unknown".to_string(),
    }
}

fn expected_field_type_names(fields: &[StructField]) -> Vec<(String, String)> {
    fields
        .iter()
        .map(|field| (field.name.clone(), field.ty.display_name()))
        .collect()
}

fn expected_field_types(fields: &[StructField]) -> Vec<(String, AstType)> {
    fields
        .iter()
        .map(|field| (field.name.clone(), field.ty.clone()))
        .collect()
}

fn format_field_types(fields: Option<&[(String, AstType)]>) -> String {
    match fields {
        Some(fields) => format!(
            "({})",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", ty.display_name()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_field_type_names(fields: Option<&[(String, String)]>) -> String {
    match fields {
        Some(fields) => format!(
            "({})",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {ty}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn expected_variant_names(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

fn format_variant_names(variants: Option<&[String]>) -> String {
    match variants {
        Some(variants) => format!("({})", variants.join(", ")),
        None => "unknown".to_string(),
    }
}

fn expected_variant_payload_type_name(payload: &Option<AstType>) -> Option<String> {
    payload.as_ref().map(AstType::display_name)
}

fn expected_behavior_method_signatures(
    methods: &[ast::BehaviorMethod],
) -> Vec<MethodSignatureMetadata> {
    methods
        .iter()
        .map(|method| {
            (
                method.name.clone(),
                expected_parameter_type_names(&method.params),
                expected_return_type_name(&method.return_type),
            )
        })
        .collect()
}

fn expected_behavior_method_types(
    methods: &[ast::BehaviorMethod],
) -> Vec<BehaviorMethodTypeMetadata> {
    methods
        .iter()
        .map(|method| BehaviorMethodTypeMetadata {
            name: method.name.clone(),
            parameter_types: method.params.iter().map(|param| param.ty.clone()).collect(),
            return_type: method.return_type.clone().unwrap_or(AstType::Void),
        })
        .collect()
}

fn expected_behavior_associations(
    program: &ast::Program,
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
    let mut impls: HashMap<String, Vec<String>> = HashMap::new();
    let mut requires: HashMap<String, Vec<String>> = HashMap::new();
    for decl in &program.declarations {
        match decl {
            Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                ..
            } => {
                impls
                    .entry(type_name.clone())
                    .or_default()
                    .push(behavior_ref_display(behavior, behavior_type_args));
            }
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                ..
            } => {
                requires
                    .entry(type_name.clone())
                    .or_default()
                    .push(behavior_ref_display(behavior, behavior_type_args));
            }
            _ => {}
        }
    }
    (impls, requires)
}

fn expected_behavior_association_refs(
    program: &ast::Program,
) -> (
    HashMap<String, Vec<BehaviorRefMetadata>>,
    HashMap<String, Vec<BehaviorRefMetadata>>,
) {
    let mut impls: HashMap<String, Vec<BehaviorRefMetadata>> = HashMap::new();
    let mut requires: HashMap<String, Vec<BehaviorRefMetadata>> = HashMap::new();
    for decl in &program.declarations {
        match decl {
            Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                ..
            } => {
                impls
                    .entry(type_name.clone())
                    .or_default()
                    .push(BehaviorRefMetadata {
                        name: behavior.clone(),
                        type_args: behavior_type_args.clone(),
                    });
            }
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                ..
            } => {
                requires
                    .entry(type_name.clone())
                    .or_default()
                    .push(BehaviorRefMetadata {
                        name: behavior.clone(),
                        type_args: behavior_type_args.clone(),
                    });
            }
            _ => {}
        }
    }
    (impls, requires)
}

fn expected_behavior_parent_associations(program: &ast::Program) -> HashMap<String, Vec<String>> {
    let mut parents: HashMap<String, Vec<String>> = HashMap::new();
    for decl in &program.declarations {
        if let Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            ..
        } = decl
        {
            parents
                .entry(behavior.clone())
                .or_default()
                .push(behavior_ref_display(parent, parent_type_args));
        }
    }
    parents
}

fn expected_behavior_parent_ref_associations(
    program: &ast::Program,
) -> HashMap<String, Vec<BehaviorRefMetadata>> {
    let mut parents: HashMap<String, Vec<BehaviorRefMetadata>> = HashMap::new();
    for decl in &program.declarations {
        if let Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            ..
        } = decl
        {
            parents
                .entry(behavior.clone())
                .or_default()
                .push(BehaviorRefMetadata {
                    name: parent.clone(),
                    type_args: parent_type_args.clone(),
                });
        }
    }
    parents
}

fn expected_resolver_declaration_symbols(program: &ast::Program) -> HashSet<(Namespace, String)> {
    let mut expected = HashSet::new();
    let validate_imports = program
        .declarations
        .iter()
        .any(|decl| matches!(decl, Declaration::Import { .. }));
    for decl in &program.declarations {
        match decl {
            Declaration::Function { name, .. } => {
                expected.insert((Namespace::Value, name.clone()));
            }
            Declaration::Method {
                type_name,
                method_name,
                ..
            } => {
                expected.insert((Namespace::Value, format!("{type_name}.{method_name}")));
            }
            Declaration::Struct { name, .. } => {
                expected.insert((Namespace::Type, name.clone()));
            }
            Declaration::Enum { name, variants, .. } => {
                expected.insert((Namespace::Type, name.clone()));
                for variant in variants {
                    expected.insert((Namespace::Variant, variant.name.clone()));
                }
            }
            Declaration::Behavior { name, .. } => {
                expected.insert((Namespace::Behavior, name.clone()));
            }
            Declaration::Import {
                names, module_path, ..
            } if validate_imports => {
                expected.insert((Namespace::Module, module_path.join(".")));
                for name in names {
                    expected.insert((Namespace::Import, name.clone()));
                }
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                for method in methods {
                    if let Declaration::Function { name, .. } = method {
                        expected.insert((Namespace::Value, format!("{type_name}.{name}")));
                    }
                }
            }
            Declaration::Import { .. }
            | Declaration::Requires { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
    }
    expected
}

fn expected_resolver_local_symbols(program: &ast::Program) -> HashSet<(String, u32)> {
    let mut expected = HashSet::new();
    let mut scope_cursor = ResolverScopeCursor::default();
    for decl in &program.declarations {
        match decl {
            Declaration::Function { params, body, .. }
            | Declaration::Method { params, body, .. } => {
                let mut locals = scope_cursor.new_scope();
                expected_resolver_parameter_locals(params, &mut locals, &mut expected);
                expected_resolver_expr_locals(body, &mut scope_cursor, &mut locals, &mut expected);
            }
            Declaration::Struct { fields, .. } => {
                for field in fields {
                    if let Some(default) = &field.default {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_expr_locals(
                            default,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::Behavior { methods, .. } => {
                for method in methods {
                    if let Some(default_body) = &method.default_body {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_parameter_locals(
                            &method.params,
                            &mut locals,
                            &mut expected,
                        );
                        expected_resolver_expr_locals(
                            default_body,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::ImplBlock { methods, .. } => {
                for method in methods {
                    if let Declaration::Function { params, body, .. } = method {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_parameter_locals(params, &mut locals, &mut expected);
                        expected_resolver_expr_locals(
                            body,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::TopLevelExpr { expr, .. } => {
                let mut locals = scope_cursor.new_scope();
                expected_resolver_expr_locals(expr, &mut scope_cursor, &mut locals, &mut expected);
            }
            Declaration::Enum { .. }
            | Declaration::Import { .. }
            | Declaration::Requires { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::Error { .. } => {}
        }
    }
    expected
}

fn expected_resolver_parameter_locals(
    params: &[Param],
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    for param in params {
        expected_resolver_local(&param.name, param.mutable, locals, expected);
    }
}

fn expected_resolver_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match expr {
        Expression::BinaryOp { left, right, .. } => {
            expected_resolver_expr_locals(left, scope_cursor, locals, expected);
            expected_resolver_expr_locals(right, scope_cursor, locals, expected);
        }
        Expression::UnaryOp { operand, .. } => {
            expected_resolver_expr_locals(operand, scope_cursor, locals, expected);
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MethodCall { receiver, args, .. } => {
            expected_resolver_expr_locals(receiver, scope_cursor, locals, expected);
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MemberAccess { object, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
        }
        Expression::IndexAccess { object, index, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
            expected_resolver_expr_locals(index, scope_cursor, locals, expected);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                expected_resolver_expr_locals(payload, scope_cursor, locals, expected);
            }
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                expected_resolver_expr_locals(element, scope_cursor, locals, expected);
            }
        }
        Expression::Match {
            scrutinee, arms, ..
        } => {
            expected_resolver_expr_locals(scrutinee, scope_cursor, locals, expected);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    let mut guard_locals = scope_cursor.child_scope(locals);
                    expected_resolver_pattern_locals(
                        &arm.pattern,
                        scope_cursor,
                        &mut guard_locals,
                        expected,
                    );
                    expected_resolver_expr_locals(guard, scope_cursor, &mut guard_locals, expected);
                }
                let mut arm_locals = scope_cursor.child_scope(locals);
                expected_resolver_pattern_locals(
                    &arm.pattern,
                    scope_cursor,
                    &mut arm_locals,
                    expected,
                );
                expected_resolver_expr_locals(&arm.body, scope_cursor, &mut arm_locals, expected);
            }
        }
        Expression::WhileLoop {
            condition, body, ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            let mut body_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(body, scope_cursor, &mut body_locals, expected);
        }
        Expression::Loop { body, .. } => {
            let mut body_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(body, scope_cursor, &mut body_locals, expected);
        }
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            let mut then_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(then_body, scope_cursor, &mut then_locals, expected);
            if let Some(else_body) = else_body {
                let mut else_locals = scope_cursor.child_scope(locals);
                expected_resolver_expr_locals(else_body, scope_cursor, &mut else_locals, expected);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            let mut block_locals = scope_cursor.child_scope(locals);
            for statement in statements {
                expected_resolver_statement_locals(
                    statement,
                    scope_cursor,
                    &mut block_locals,
                    expected,
                );
            }
            if let Some(expr) = expr {
                expected_resolver_expr_locals(expr, scope_cursor, &mut block_locals, expected);
            }
        }
        Expression::Return { value, .. } => {
            if let Some(value) = value {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::Closure { params, body, .. } => {
            let mut closure_locals = scope_cursor.child_scope(locals);
            for param in params {
                expected_resolver_local(&param.name, false, &mut closure_locals, expected);
            }
            expected_resolver_expr_locals(body, scope_cursor, &mut closure_locals, expected);
        }
        Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let ast::StringPart::Expr(expr) = part {
                    expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
                }
            }
        }
        Expression::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        Expression::Identifier { .. }
        | Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::CharLiteral { .. }
        | Expression::Break { .. }
        | Expression::Continue { .. }
        | Expression::Error { .. } => {}
    }
}

fn expected_resolver_statement_locals(
    statement: &ast::Statement,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match statement {
        ast::Statement::VarDecl {
            name,
            value,
            mutable,
            constant,
            ..
        } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            if *constant || *mutable || !locals.is_mutable(name) {
                expected_resolver_local(name, *mutable, locals, expected);
            }
        }
        ast::Statement::Assignment { target, value, .. } => {
            expected_resolver_expr_locals(target, scope_cursor, locals, expected);
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Statement::Expression { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        ast::Statement::Block { stmts, .. } => {
            let mut block_locals = scope_cursor.child_scope(locals);
            for statement in stmts {
                expected_resolver_statement_locals(
                    statement,
                    scope_cursor,
                    &mut block_locals,
                    expected,
                );
            }
        }
    }
}

fn expected_resolver_pattern_locals(
    pattern: &ast::Pattern,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match pattern {
        ast::Pattern::Identifier { name, .. } => {
            expected_resolver_local(name, false, locals, expected);
        }
        ast::Pattern::Struct { fields, .. } => {
            for (name, nested) in fields {
                if let Some(nested) = nested {
                    expected_resolver_pattern_locals(nested, scope_cursor, locals, expected);
                } else {
                    expected_resolver_local(name, false, locals, expected);
                }
            }
        }
        ast::Pattern::Enum {
            payload: Some(payload),
            ..
        } => {
            expected_resolver_pattern_locals(payload, scope_cursor, locals, expected);
        }
        ast::Pattern::Or { patterns, .. } => {
            for pattern in patterns {
                expected_resolver_pattern_locals(pattern, scope_cursor, locals, expected);
            }
        }
        ast::Pattern::Literal { value, .. } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Pattern::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        ast::Pattern::Wildcard { .. }
        | ast::Pattern::Enum { payload: None, .. }
        | ast::Pattern::BoolTrue { .. }
        | ast::Pattern::BoolFalse { .. } => {}
    }
}

fn expected_resolver_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected.insert((name.to_string(), locals.current_scope_id));
    locals.insert(name.to_string(), mutable);
}

fn format_behavior_method_signatures(methods: Option<&[MethodSignatureMetadata]>) -> String {
    match methods {
        Some(methods) => format!(
            "({})",
            methods
                .iter()
                .map(|(name, params, return_type)| {
                    format!("{name}({}) {return_type}", params.join(", "))
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_behavior_method_types(methods: Option<&[BehaviorMethodTypeMetadata]>) -> String {
    match methods {
        Some(methods) => format!(
            "({})",
            methods
                .iter()
                .map(|method| {
                    let params = method
                        .parameter_types
                        .iter()
                        .map(AstType::display_name)
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{}({}) {}",
                        method.name,
                        params,
                        method.return_type.display_name()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unknown".to_string(),
    }
}

fn format_behavior_parent_names(parents: Option<&[String]>) -> String {
    format_behavior_ref_names(parents)
}

fn format_behavior_ref_names(parents: Option<&[String]>) -> String {
    match parents {
        Some(parents) if !parents.is_empty() => parents.join(", "),
        _ => "none".to_string(),
    }
}

fn format_behavior_refs(refs: Option<&[BehaviorRefMetadata]>) -> String {
    match refs {
        Some(refs) if !refs.is_empty() => refs
            .iter()
            .map(|behavior| behavior_ref_display(&behavior.name, &behavior.type_args))
            .collect::<Vec<_>>()
            .join(", "),
        _ => "none".to_string(),
    }
}

fn behavior_ref_names_match(actual: Option<&[String]>, expected: &[String]) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

fn behavior_refs_match(
    actual: Option<&[BehaviorRefMetadata]>,
    expected: &[BehaviorRefMetadata],
) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::declarations::StructField;
    use crate::ast::expressions::BinaryOp;
    use crate::error::Span;

    fn parse_program(src: &str) -> ast::Program {
        let mut files = crate::error::FileTable::new();
        let file_id = files.add_file("test.zen".to_string(), src.to_string());
        let tokens = crate::lexer::tokenize(src, file_id).expect("tokenize");
        crate::parser::parse(tokens, file_id).expect("parse")
    }

    #[test]
    fn resolve_primitive_types() {
        let tc = TypeChecker::new();
        assert_eq!(tc.resolve_type(&AstType::I32), Type::I32);
        assert_eq!(tc.resolve_type(&AstType::F64), Type::F64);
        assert_eq!(tc.resolve_type(&AstType::Bool), Type::Bool);
        assert_eq!(tc.resolve_type(&AstType::Void), Type::Void);
        assert_eq!(tc.resolve_type(&AstType::Str), Type::Str);
    }

    #[test]
    fn resolve_pointer_types() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.resolve_type(&AstType::Ptr(Box::new(AstType::I32))),
            Type::Ptr(Box::new(Type::I32))
        );
    }

    #[test]
    fn check_program_rejects_self_type_outside_method_or_behavior() {
        let program = parse_program(
            r#"
main = (value: Self) i32 { return 0 }
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("Self should require a method or behavior context");

        assert!(
            err.iter()
                .any(|d| d.message.contains("Self type is only valid")),
            "expected invalid Self type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_rejects_unknown_type_references() {
        let program = parse_program(
            r#"
main = (value: Missing, items: Bag<i32>) i32 { return 0 }
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("unknown type reference should fail");

        assert!(
            err.iter()
                .any(|d| d.message.contains("unknown type symbol 'Missing'")),
            "expected unknown type diagnostic, got {err:?}"
        );
        assert!(
            err.iter()
                .any(|d| d.message.contains("unknown type symbol 'Bag'")),
            "expected unknown generic type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn scope_variable_lookup() {
        let mut tc = TypeChecker::new();
        tc.define_var("x", Type::I32);
        assert_eq!(tc.lookup_var("x"), Some(Type::I32));

        tc.push_scope();
        tc.define_var("y", Type::Bool);
        assert_eq!(tc.lookup_var("y"), Some(Type::Bool));
        assert_eq!(tc.lookup_var("x"), Some(Type::I32)); // parent scope

        tc.pop_scope();
        assert_eq!(tc.lookup_var("y"), None); // out of scope
    }

    #[test]
    fn collect_struct_info() {
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Struct {
            name: "Point".into(),
            type_params: Vec::new(),
            fields: vec![
                StructField {
                    name: "x".into(),
                    ty: AstType::F64,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                },
                StructField {
                    name: "y".into(),
                    ty: AstType::F64,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                },
            ],
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        assert!(tc.structs.contains_key("Point"));
        assert_eq!(tc.structs["Point"].fields.len(), 2);
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_type_metadata() {
        let mut program = parse_program(
            r#"
apply = (callback: (i32) i32) (i32) i32 {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            params[0].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.functions.get("apply").expect("function info");
        assert_eq!(
            info.params[0].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.return_type,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
apply<T: Json<T>> = (callback: (T) T) (T) T {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function {
            type_params,
            params,
            return_type,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[0].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc.generic_functions.get("apply").expect("generic template");
        assert_eq!(template.type_params, vec!["T".to_string()]);
        assert_eq!(
            tc.functions
                .get("apply")
                .expect("function info")
                .type_param_bounds
                .get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            template.params[0].ty,
            AstType::Function {
                params: vec![AstType::Named("T".to_string())],
                ret: Box::new(AstType::Named("T".to_string())),
            }
        );
        assert_eq!(
            template.return_type,
            Some(AstType::Function {
                params: vec![AstType::Named("T".to_string())],
                ret: Box::new(AstType::Named("T".to_string())),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }
Box.apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method {
            type_params,
            params,
            return_type,
            ..
        } = &mut program.declarations[3]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.apply")
            .expect("generic method template");
        assert_eq!(template.type_params, vec!["U".to_string()]);
        assert_eq!(
            tc.methods
                .get("Box.apply")
                .expect("method info")
                .type_param_bounds
                .get("U"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("U".to_string())],
            })
        );
        assert_eq!(
            template.params[1].ty,
            AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            }
        );
        assert_eq!(
            template.return_type,
            Some(AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_struct_field_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Pipeline<T: Json<T>>: { callback: (i32) i32 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct {
            type_params,
            fields,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            fields[0].ty = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.structs.get("Pipeline").expect("struct info");
        assert_eq!(
            info.type_param_bounds.get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            info.fields[0].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_enum_payload_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Callback<T: Json<T>>: Wrap((i32) i32), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Enum {
            type_params,
            variants,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            variants[0].payload = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.enums.get("Callback").expect("enum info");
        assert_eq!(
            info.type_param_bounds.get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            info.variants[0].1,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_metadata() {
        let mut program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Mapper").expect("behavior info");
        assert_eq!(
            info.methods[0].params[1].ty,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.methods[0].return_type,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32 { return callback }
}

Point.implements(Mapper) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.methods.get("Point.map").expect("default method info");
        assert_eq!(
            info.params[1].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.return_type,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_defers_impl_checks_until_resolver_metadata_is_collected() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { return callback }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { return callback }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function {
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                params[1].ty = AstType::I32;
                *return_type = Some(AstType::I32);
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::BehaviorExtends {
            parent_type_args, ..
        } = &mut program.declarations[2]
        {
            parent_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let parents = tc
            .behavior_extends
            .get("PrettyJson")
            .expect("behavior parents");
        assert_eq!(parents[0].behavior, "Json");
        assert_eq!(parents[0].type_args, vec![AstType::Str]);
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            behavior_type_args, ..
        } = &mut program.declarations[2]
        {
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.behavior_impls
                .contains(&("Point".to_string(), "Json_str".to_string())),
            "resolver metadata should restore the validated Json<str> impl"
        );
        assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "AST-only Json<i32> impl drift should not remain after resolver collection"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_declarations() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let empty_symbols = SymbolTable::default();
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &empty_symbols)
            .expect_err("missing resolver symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing value symbol 'main'")),
            "expected missing resolver symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_declarations() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
main = () i32 { return 0 }
extra = () i32 { return 1 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver declarations should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra value symbol 'extra'")),
            "expected extra resolver symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
{ io, math } = std
main = () i32 { return 0 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver imports should fail when AST imports are present");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra import symbol 'math'")),
            "expected extra resolver import diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
{ io } = std
{ helper } = other
main = () i32 { return 0 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver modules should fail when AST imports are present");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra module symbol 'other'")),
            "expected extra resolver module diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_method_receiver_type() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Point.label = () str { return "point" }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Type, "Point");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing receiver type resolver symbol should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing type symbol 'Point'")),
            "expected missing method receiver type symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_method_signature() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    return self.value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "Box.get",
            Some(vec!["Box<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.get' has parameter types '(Box<i32>)', expected '(Box<T>)'"
            )),
            "expected resolver method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_method_function_type_signature() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    return callback
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "Box.map",
            Some(vec!["Box<T>".to_string(), "T".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver method function type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.map' has parameter types '(Box<T>, T)', expected '(Box<T>, (T) T)'"
            )),
            "expected resolver method function type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_uses_resolver_import_bindings() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 {
    io.println("ok")
    return 0
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));

        let mut tc = TypeChecker::new();
        tc.check_program_with_symbols(&program, &symbols)
            .expect("resolver import symbols should seed typechecker imports");

        assert!(tc.is_root_std_import("io"));
    }

    #[test]
    fn check_program_with_symbols_validates_stripped_resolver_import_sources() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Import, "io", None);
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver imports without sources should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver import symbol 'io' has source 'unknown', expected a module source"
            )),
            "expected stripped resolver import source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_stripped_resolver_import_visibility() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Import, "io", true);
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver import visibility should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has visibility public, expected private")),
            "expected stripped resolver import visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_stripped_resolver_import_modules() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Module, "std");
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver imports should require source module symbols");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing module symbol 'std'")),
            "expected stripped resolver import module diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_module_graph_entry_uses_graph_import_bindings() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let math_path = tmp.path().join("math.zen");
        std::fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            "{ add } = math\n\nmain = () i32 { return add(1, 2) }\n",
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");
        let entry = graph.module(graph.entry).expect("entry module");
        assert!(
            !entry
                .program
                .declarations
                .iter()
                .any(|decl| decl.name() == Some("add")),
            "graph entry should not merge imported declarations"
        );

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed imported signatures");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "main"));
        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "add"));
    }

    #[test]
    fn check_module_graph_entry_seeds_imported_function_type_signatures() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let callbacks_path = tmp.path().join("callbacks.zen");
        std::fs::write(
            &callbacks_path,
            "pub apply = (callback: (i32) i32, value: i32) i32 { return value }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { return value }
    return apply(callback, 1)
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        tc.check_module_graph_entry(&graph)
            .expect("graph import bindings should seed function-typed signatures");
    }

    #[test]
    fn check_module_graph_entry_specializes_imported_generic_functions() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let identity_path = tmp.path().join("identity.zen");
        std::fs::write(
            &identity_path,
            "pub id<T> = (value: T) T { return value }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            "{ id } = identity\n\nmain = () i32 { return id<i32>(1) }\n",
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed generic templates");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "id_i32"));
    }

    #[test]
    fn check_module_graph_entry_specializes_imported_generic_enums() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let option_path = tmp.path().join("option.zen");
        std::fs::write(
            &option_path,
            r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, str>.Ok(9)
    return 0
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed generic enum templates");

        assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
        assert!(typed.types.iter().any(|ty| ty.name == "Result_i32_str"));
    }

    #[test]
    fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    return self.x
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.value()
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        tc.check_module_graph_entry(&graph)
            .expect("imported public type should seed its public methods");
    }

    #[test]
    fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    return self.x
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.value()
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let err = TypeChecker::new()
            .check_module_graph_entry(&graph)
            .expect_err("private imported methods should not be seeded");

        assert!(
            err.iter()
                .any(|d| d.message.contains("type `Point` has no method `value`")),
            "expected private imported method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    return value
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.keep<i32>(1)
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("imported public type should seed public generic method templates");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "Point.keep_i32"));
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_sources() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Import, "io", Some("other".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import source mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has source 'other', expected 'std'")),
            "expected resolver import source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_visibility() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Import, "io", true);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has visibility public, expected private")),
            "expected resolver import visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_absent_declaration_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Import, "io", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import declaration metadata should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver import symbol 'io' has parameter count metadata, expected none"
            )),
            "expected resolver import parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has return type metadata, expected none")),
            "expected resolver import return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_absent_type_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Import, "io", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Import, "io", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Import, "io", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Import,
            "io",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Import,
            "io",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Import,
            "io",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(Namespace::Import, "io", Some(vec!["Some".to_string()]));
        symbols.set_variant_owner_name_for_test(
            Namespace::Import,
            "io",
            Some("Option".to_string()),
        );
        symbols.set_variant_payload_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Import,
            "io",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Import, "io", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Import,
            "io",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import type metadata should fail");

        for expected in [
            "resolver import symbol 'io' has parameter names metadata, expected none",
            "resolver import symbol 'io' has parameter types metadata, expected none",
            "resolver import symbol 'io' has typed parameter types metadata, expected none",
            "resolver import symbol 'io' has typed return type metadata, expected none",
            "resolver import symbol 'io' has type parameter count metadata, expected none",
            "resolver import symbol 'io' has type parameter names metadata, expected none",
            "resolver import symbol 'io' has type parameter bounds metadata, expected none",
            "resolver import symbol 'io' has typed type parameter bound refs metadata, expected none",
            "resolver import symbol 'io' has field count metadata, expected none",
            "resolver import symbol 'io' has field types metadata, expected none",
            "resolver import symbol 'io' has typed field types metadata, expected none",
            "resolver import symbol 'io' has variant names metadata, expected none",
            "resolver import symbol 'io' has variant owner metadata, expected none",
            "resolver import symbol 'io' has variant payload count metadata, expected none",
            "resolver import symbol 'io' has variant payload type metadata, expected none",
            "resolver import symbol 'io' has typed variant payload type metadata, expected none",
            "resolver import symbol 'io' has behavior methods metadata, expected none",
            "resolver import symbol 'io' has typed behavior methods metadata, expected none",
            "resolver import symbol 'io' has behavior parents metadata, expected none",
            "resolver import symbol 'io' has typed behavior parents metadata, expected none",
            "resolver import symbol 'io' has behavior impls metadata, expected none",
            "resolver import symbol 'io' has typed behavior impls metadata, expected none",
            "resolver import symbol 'io' has behavior requires metadata, expected none",
            "resolver import symbol 'io' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver import metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_and_module_absent_mutability() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_mutability_for_test(Namespace::Import, "io", Some(true));
        symbols.set_mutability_for_test(Namespace::Module, "std", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import/module mutability metadata should fail");

        for expected in [
            "resolver import symbol 'io' has mutability metadata, expected none",
            "resolver module symbol 'std' has mutability metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver import/module mutability diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_symbols() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Module, "std", true);
        symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has visibility public, expected private")),
            "expected resolver module visibility diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has source 'other', expected none")),
            "expected resolver module source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module declaration metadata should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver module symbol 'std' has parameter count metadata, expected none"
            )),
            "expected resolver module parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has return type metadata, expected none")),
            "expected resolver module return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_absent_type_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Module, "std", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Module, "std", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Module, "std", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Module,
            "std",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Module,
            "std",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Module,
            "std",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_owner_name_for_test(
            Namespace::Module,
            "std",
            Some("Option".to_string()),
        );
        symbols.set_variant_payload_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Module,
            "std",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Module, "std", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Module,
            "std",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module type metadata should fail");

        for expected in [
            "resolver module symbol 'std' has parameter names metadata, expected none",
            "resolver module symbol 'std' has parameter types metadata, expected none",
            "resolver module symbol 'std' has typed parameter types metadata, expected none",
            "resolver module symbol 'std' has typed return type metadata, expected none",
            "resolver module symbol 'std' has type parameter count metadata, expected none",
            "resolver module symbol 'std' has type parameter names metadata, expected none",
            "resolver module symbol 'std' has type parameter bounds metadata, expected none",
            "resolver module symbol 'std' has typed type parameter bound refs metadata, expected none",
            "resolver module symbol 'std' has field count metadata, expected none",
            "resolver module symbol 'std' has field types metadata, expected none",
            "resolver module symbol 'std' has typed field types metadata, expected none",
            "resolver module symbol 'std' has variant names metadata, expected none",
            "resolver module symbol 'std' has variant owner metadata, expected none",
            "resolver module symbol 'std' has variant payload count metadata, expected none",
            "resolver module symbol 'std' has variant payload type metadata, expected none",
            "resolver module symbol 'std' has typed variant payload type metadata, expected none",
            "resolver module symbol 'std' has behavior methods metadata, expected none",
            "resolver module symbol 'std' has typed behavior methods metadata, expected none",
            "resolver module symbol 'std' has behavior parents metadata, expected none",
            "resolver module symbol 'std' has typed behavior parents metadata, expected none",
            "resolver module symbol 'std' has behavior impls metadata, expected none",
            "resolver module symbol 'std' has typed behavior impls metadata, expected none",
            "resolver module symbol 'std' has behavior requires metadata, expected none",
            "resolver module symbol 'std' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver module metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_impl_methods() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = SymbolTable::default();
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing impl method resolver symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing value symbol 'Point.stringify'")),
            "expected missing impl method symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_impl_method_signature() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(
            Namespace::Value,
            "Point.stringify",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver impl method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Point.stringify' has return type 'i32', expected 'str'"
            )),
            "expected resolver impl method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_impl_function_type_signature() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        return callback
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(
            Namespace::Value,
            "Point.map",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver impl method function type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Point.map' has return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver impl method function type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_impl_method_body_locals() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str {
        label = "point"
        return label
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "label");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver impl method body local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'label'")),
            "expected missing resolver impl method body local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_enum_variants() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Variant, "Some");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver enum variant symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing variant symbol 'Some'")),
            "expected missing enum variant symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_arity() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Value, "add", Some(1));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function arity mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver value symbol 'add' has parameter count 1, expected 2")),
            "expected resolver function arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_parameter_types() {
        let program = parse_program(
            r#"
add = (a: i32, b: f64) f64 { return b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "add",
            Some(vec!["i32".to_string(), "i32".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function parameter type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
            )),
            "expected resolver function parameter type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_metadata() {
        let program = parse_program(
            r#"
apply = (callback: (i32) i32, value: i32) i32 { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "apply",
            Some(vec!["i32".to_string(), "i32".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function type parameter metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has parameter types '(i32, i32)', expected '((i32) i32, i32)'"
            )),
            "expected resolver function type parameter metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_parameter_names() {
        let program = parse_program(
            r#"
add = (a: i32, b: f64) f64 { return b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(
            Namespace::Value,
            "add",
            Some(vec!["a".to_string(), "other".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function parameter name mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
            )),
            "expected resolver function parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_parameter_locals() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "a");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver parameter local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'a'")),
            "expected missing resolver parameter local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
        let program = parse_program(
            r#"
add = (mut a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_local_mutability_for_test("a", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver parameter local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
            "expected resolver parameter local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Local, "a", true);
        symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local visibility/source mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has visibility public, expected private")),
            "expected resolver local visibility diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has source 'std', expected none")),
            "expected resolver local source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_absent_declaration_metadata() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local declaration metadata should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has parameter count metadata, expected none")),
            "expected resolver local parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has return type metadata, expected none")),
            "expected resolver local return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_absent_type_metadata() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Local, "a", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Local, "a", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Local, "a", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Local,
            "a",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Local,
            "a",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Local,
            "a",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(Namespace::Local, "a", Some(vec!["Some".to_string()]));
        symbols.set_variant_owner_name_for_test(Namespace::Local, "a", Some("Option".to_string()));
        symbols.set_variant_payload_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Local,
            "a",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Local, "a", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Local,
            "a",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local type metadata should fail");

        for expected in [
            "resolver local symbol 'a' has parameter names metadata, expected none",
            "resolver local symbol 'a' has parameter types metadata, expected none",
            "resolver local symbol 'a' has typed parameter types metadata, expected none",
            "resolver local symbol 'a' has typed return type metadata, expected none",
            "resolver local symbol 'a' has type parameter count metadata, expected none",
            "resolver local symbol 'a' has type parameter names metadata, expected none",
            "resolver local symbol 'a' has type parameter bounds metadata, expected none",
            "resolver local symbol 'a' has typed type parameter bound refs metadata, expected none",
            "resolver local symbol 'a' has field count metadata, expected none",
            "resolver local symbol 'a' has field types metadata, expected none",
            "resolver local symbol 'a' has typed field types metadata, expected none",
            "resolver local symbol 'a' has variant names metadata, expected none",
            "resolver local symbol 'a' has variant owner metadata, expected none",
            "resolver local symbol 'a' has variant payload count metadata, expected none",
            "resolver local symbol 'a' has variant payload type metadata, expected none",
            "resolver local symbol 'a' has typed variant payload type metadata, expected none",
            "resolver local symbol 'a' has behavior methods metadata, expected none",
            "resolver local symbol 'a' has typed behavior methods metadata, expected none",
            "resolver local symbol 'a' has behavior parents metadata, expected none",
            "resolver local symbol 'a' has typed behavior parents metadata, expected none",
            "resolver local symbol 'a' has behavior impls metadata, expected none",
            "resolver local symbol 'a' has typed behavior impls metadata, expected none",
            "resolver local symbol 'a' has behavior requires metadata, expected none",
            "resolver local symbol 'a' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver local metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_var_decl_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    value = 1
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver var local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver var local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
        let program = parse_program(
            r#"
main = () i32 {
    value ::= 1
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_local_mutability_for_test("value", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver var local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver local symbol 'value' has mutability immutable, expected mutable"
            )),
            "expected resolver var local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    return 0
}
"#,
        );
        let symbols_program = parse_program(
            r#"
main = () i32 {
    value = 1
    return 0
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra local symbol 'value'")),
            "expected extra resolver local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
        let program = parse_program(
            r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let inner_scope = symbols
            .symbols()
            .iter()
            .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
            .map(|symbol| symbol.scope_id)
            .max()
            .expect("inner value local");
        symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver scoped local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver local symbol 'value' has mutability mutable, expected immutable"
            )),
            "expected scoped resolver local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_pattern_locals() {
        let program = parse_program(
            r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    return value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "inner");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver pattern local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'inner'")),
            "expected missing resolver pattern local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_top_level_expr_locals() {
        let program = parse_program(
            r#"
value := 1
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver top-level expr local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver top-level expr local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_closure_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "inner");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver closure local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'inner'")),
            "expected missing resolver closure local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_struct_field_default_locals() {
        let program = parse_program(
            r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver struct field default local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver struct field default local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_behavior_default_locals() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str {
        value = "{}"
        value
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver behavior default local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver behavior default local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_visibility() {
        let program = parse_program(
            r#"
pub exported = () i32 { return 1 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Value, "exported", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'exported' has visibility private, expected public"
            )),
            "expected resolver function visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_return_type() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(Namespace::Value, "main", Some("bool".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function return mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver value symbol 'main' has return type 'bool', expected 'i32'")),
            "expected resolver function return diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_return_metadata() {
        let program = parse_program(
            r#"
factory = () (i32) i32 {
    return (value: i32) i32 { value }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(Namespace::Value, "factory", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function type return metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'factory' has return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver function type return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata() {
        let program = parse_program(
            r#"
apply = (callback: (i32) i32) (i32) i32 {
    return callback
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_types_for_test(Namespace::Value, "apply", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Value, "apply", Some(AstType::I32));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed function signature metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
            )),
            "expected resolver typed parameter diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver typed return diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_counts() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_count_for_test(Namespace::Value, "identity", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic arity mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'identity' has type parameter count 0, expected 1"
            )),
            "expected resolver function generic arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_names() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_names_for_test(
            Namespace::Value,
            "identity",
            Some(vec!["U".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic parameter name mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver function generic parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { return "encoded" }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Value,
            "encode",
            Some(vec![("T".to_string(), "Other".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'encode' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver function generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Value,
            "identity",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: vec![AstType::Str],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic bound ref mismatch should fail");

        let expected = "resolver value symbol 'identity' has type parameter bound refs '(T: Json<str>)', expected '(T: Json<T>)'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver function generic bound ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_absent_declaration_metadata() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Value, "main", Some("std".to_string()));
        symbols.set_field_count_for_test(Namespace::Value, "main", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Value,
            "main",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Value,
            "main",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Value,
            "main",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Value, "main", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Value,
            "main",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_types: vec![AstType::Named("Self".to_string())],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function declaration metadata should fail");

        for expected in [
            "resolver value symbol 'main' has source 'std', expected none",
            "resolver value symbol 'main' has field count metadata, expected none",
            "resolver value symbol 'main' has field types metadata, expected none",
            "resolver value symbol 'main' has typed field types metadata, expected none",
            "resolver value symbol 'main' has variant names metadata, expected none",
            "resolver value symbol 'main' has variant payload type metadata, expected none",
            "resolver value symbol 'main' has typed variant payload type metadata, expected none",
            "resolver value symbol 'main' has behavior methods metadata, expected none",
            "resolver value symbol 'main' has typed behavior methods metadata, expected none",
            "resolver value symbol 'main' has behavior parents metadata, expected none",
            "resolver value symbol 'main' has typed behavior parents metadata, expected none",
            "resolver value symbol 'main' has behavior impls metadata, expected none",
            "resolver value symbol 'main' has typed behavior impls metadata, expected none",
            "resolver value symbol 'main' has behavior requires metadata, expected none",
            "resolver value symbol 'main' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver function declaration metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_counts() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_count_for_test(Namespace::Type, "Box", Some(0));
        symbols.set_type_parameter_count_for_test(Namespace::Behavior, "Serializable", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic arity mismatches should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Box' has type parameter count 0, expected 1")),
            "expected resolver type generic arity diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter count 0, expected 1"
            )),
            "expected resolver behavior generic arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_names() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_names_for_test(
            Namespace::Type,
            "Box",
            Some(vec!["U".to_string()]),
        );
        symbols.set_type_parameter_names_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec!["U".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic parameter name mismatches should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver type generic parameter name diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver behavior generic parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_visibility() {
        let program = parse_program(
            r#"
pub Box<T>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Type, "Box", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Box' has visibility private, expected public")),
            "expected resolver type visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_visibility() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Behavior, "Json", true);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has visibility public, expected private"
            )),
            "expected resolver behavior visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Type,
            "Box",
            Some(vec![("T".to_string(), "Other".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver type generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec![("T".to_string(), "Json<i32>".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter bounds '(T: Json<i32>)', expected '(T: Json<T>)'"
            )),
            "expected resolver behavior generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Type, "Box", Some("std".to_string()));
        symbols.set_parameter_count_for_test(Namespace::Type, "Box", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Type, "Box", Some("i32".to_string()));
        symbols.set_return_type_for_test(Namespace::Type, "Box", Some(AstType::I32));
        symbols.set_import_source_for_test(Namespace::Behavior, "Json", Some("std".to_string()));
        symbols.set_parameter_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["value".to_string()]),
        );
        symbols.set_parameter_type_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Self".to_string()]),
        );
        symbols.set_parameter_types_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![AstType::SelfType]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type-like value metadata should fail");

        for expected in [
            "resolver type symbol 'Box' has source 'std', expected none",
            "resolver type symbol 'Box' has parameter count metadata, expected none",
            "resolver type symbol 'Box' has return type metadata, expected none",
            "resolver type symbol 'Box' has typed return type metadata, expected none",
            "resolver behavior symbol 'Json' has source 'std', expected none",
            "resolver behavior symbol 'Json' has parameter names metadata, expected none",
            "resolver behavior symbol 'Json' has parameter types metadata, expected none",
            "resolver behavior symbol 'Json' has typed parameter types metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver type-like value metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_method_signatures() {
        let program = parse_program(
            r#"
Serializable: behavior {
    encode: (Self, i32) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string(), "bool".to_string()],
                "str".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has methods '(encode(Self, bool) str)', expected '(encode(Self, i32) str)'"
            )),
            "expected resolver behavior method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![(
                "map".to_string(),
                vec!["Self".to_string(), "i32".to_string()],
                "(i32) i32".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior function type method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'"
            )),
            "expected resolver behavior function type method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_method_types() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_types_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "map".to_string(),
                parameter_types: vec![AstType::SelfType, AstType::I32],
                return_type: AstType::Function {
                    params: vec![AstType::I32],
                    ret: Box::new(AstType::I32),
                },
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed behavior method metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has typed methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'"
            )),
            "expected resolver typed behavior method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has methods '(encode(Self) str)', expected '(encode(Self) T)'"
            )),
            "expected resolver generic behavior method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures(
    ) {
        let program = parse_program(
            r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![(
                "map".to_string(),
                vec!["Self".to_string(), "T".to_string()],
                "(T) T".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior function type method mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, T) (T) T)', expected '(map(Self, (T) T) (T) T)'"
            )),
            "expected resolver generic behavior function type method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_absent_type_metadata() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_count_for_test(Namespace::Behavior, "Json", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Behavior,
            "Json",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Behavior, "Json", Some(AstType::I32));
        symbols.set_behavior_impl_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Debug".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Debug".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior type metadata should fail");

        for expected in [
            "resolver behavior symbol 'Json' has field count metadata, expected none",
            "resolver behavior symbol 'Json' has field types metadata, expected none",
            "resolver behavior symbol 'Json' has typed field types metadata, expected none",
            "resolver behavior symbol 'Json' has variant names metadata, expected none",
            "resolver behavior symbol 'Json' has variant payload type metadata, expected none",
            "resolver behavior symbol 'Json' has typed variant payload type metadata, expected none",
            "resolver behavior symbol 'Json' has behavior impls metadata, expected none",
            "resolver behavior symbol 'Json' has typed behavior impls metadata, expected none",
            "resolver behavior symbol 'Json' has behavior requires metadata, expected none",
            "resolver behavior symbol 'Json' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver behavior type metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_parent_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(Namespace::Behavior, "PrettyJson", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior parent metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'none', expected to include 'Json'"
            )),
            "expected resolver behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_parent_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior parent metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'Json<i32>', expected to include 'Json<str>'"
            )),
            "expected resolver generic behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior parent ref mismatch should fail");

        let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior parent ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior parent metadata should fail");

        let expected =
            "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_impl_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(Namespace::Type, "Point", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior impl metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'none', expected to include 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_impl_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior impl metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_impl_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior impl ref mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impl refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior impl ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_required_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(Namespace::Type, "Point", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior requires metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'none', expected to include 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_required_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior requires metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior requires ref mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior requires ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior impl metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_required_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior requires metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_field_counts() {
        let program = parse_program(
            r#"
Point: { x: i32, y: i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct field count mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Point' has field count 1, expected 2")),
            "expected resolver struct field count diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_field_types() {
        let program = parse_program(
            r#"
Point: { x: i32, y: f64 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec![
                ("x".to_string(), "i32".to_string()),
                ("y".to_string(), "i32".to_string()),
            ]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct field type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'"
            )),
            "expected resolver struct field type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
        let program = parse_program(
            r#"
Pipeline: { callback: (i32) i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Pipeline",
            Some(vec![("callback".to_string(), "i32".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct function type field mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver struct function type field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
        let program = parse_program(
            r#"
Pipeline: { callback: (i32) i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_types_for_test(
            Namespace::Type,
            "Pipeline",
            Some(vec![("callback".to_string(), AstType::I32)]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed struct field metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver typed struct field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Box",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic struct field mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
            )),
            "expected resolver generic struct field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Type,
            "Point",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
        symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Option",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Type,
            "Option",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct/enum kind metadata should fail");

        for expected in [
            "resolver type symbol 'Point' has variant names metadata, expected none",
            "resolver type symbol 'Point' has variant payload type metadata, expected none",
            "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
            "resolver type symbol 'Option' has field count metadata, expected none",
            "resolver type symbol 'Option' has field types metadata, expected none",
            "resolver type symbol 'Option' has typed field types metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant payload count mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
            "expected resolver enum variant payload count diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
        let program = parse_program(
            r#"
pub Option<T>: Some(T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Variant, "Some", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Some' has visibility private, expected public"
            )),
            "expected resolver enum variant visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Some",
            Some("bool".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant payload type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Some' has payload type 'bool', expected 'i32'"
            )),
            "expected resolver enum variant payload type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
        let program = parse_program(
            r#"
Callback: Wrap((i32) i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Wrap",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum function type payload mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
            )),
            "expected resolver enum function type payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
        let program = parse_program(
            r#"
Callback: Wrap((i32) i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed enum payload metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
            )),
            "expected resolver typed enum payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
        let program = parse_program(
            r#"
Callback<T>: Wrap((T) T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Wrap",
            Some("T".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic enum function type payload mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
            "expected resolver generic enum function type payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
        let program = parse_program(
            r#"
Result<T, E>: Ok(T), Err(E)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Err",
            Some("T".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic enum payload mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
            "expected resolver generic enum payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_variant_absent_other_metadata() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Variant, "Some", Some("std".to_string()));
        symbols.set_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_parameter_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["value".to_string()]),
        );
        symbols.set_parameter_type_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Variant, "Some", Some(vec![AstType::I32]));
        symbols.set_return_type_name_for_test(Namespace::Variant, "Some", Some("i32".to_string()));
        symbols.set_return_type_for_test(Namespace::Variant, "Some", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Other".to_string()]),
        );
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver variant non-variant metadata should fail");

        for expected in [
            "resolver variant symbol 'Some' has source 'std', expected none",
            "resolver variant symbol 'Some' has parameter count metadata, expected none",
            "resolver variant symbol 'Some' has parameter names metadata, expected none",
            "resolver variant symbol 'Some' has parameter types metadata, expected none",
            "resolver variant symbol 'Some' has typed parameter types metadata, expected none",
            "resolver variant symbol 'Some' has return type metadata, expected none",
            "resolver variant symbol 'Some' has typed return type metadata, expected none",
            "resolver variant symbol 'Some' has type parameter count metadata, expected none",
            "resolver variant symbol 'Some' has type parameter names metadata, expected none",
            "resolver variant symbol 'Some' has type parameter bounds metadata, expected none",
            "resolver variant symbol 'Some' has typed type parameter bound refs metadata, expected none",
            "resolver variant symbol 'Some' has field count metadata, expected none",
            "resolver variant symbol 'Some' has field types metadata, expected none",
            "resolver variant symbol 'Some' has typed field types metadata, expected none",
            "resolver variant symbol 'Some' has variant names metadata, expected none",
            "resolver variant symbol 'Some' has behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has behavior requires metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver variant metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_names() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(
            Namespace::Type,
            "Option",
            Some(vec!["Some".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant names mismatch should fail");

        let expected =
            "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver enum variant names diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_owner_name_for_test(
            Namespace::Variant,
            "Some",
            Some("Result".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant owner mismatch should fail");

        let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver enum variant owner diagnostic, got {err:?}"
        );
    }

    #[test]
    fn binary_op_types() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.check_binary_op(BinaryOp::Add, &Type::I32, &Type::I32, &Span::dummy())
                .unwrap(),
            Type::I32
        );
        assert_eq!(
            tc.check_binary_op(BinaryOp::Eq, &Type::I32, &Type::I32, &Span::dummy())
                .unwrap(),
            Type::Bool
        );
        assert_eq!(
            tc.check_binary_op(BinaryOp::And, &Type::Bool, &Type::Bool, &Span::dummy())
                .unwrap(),
            Type::Bool
        );
    }

    #[test]
    fn binary_op_type_mismatch() {
        let tc = TypeChecker::new();
        // Arithmetic on non-numeric type
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::I32, &Type::Str, &Span::dummy())
            .is_err());
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::Bool, &Type::I32, &Span::dummy())
            .is_err());
        // Logical op on non-bool
        assert!(tc
            .check_binary_op(BinaryOp::And, &Type::I32, &Type::Bool, &Span::dummy())
            .is_err());
        // Unknown is permissive (error recovery)
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::Unknown, &Type::Str, &Span::dummy())
            .is_ok());
    }

    #[test]
    fn binary_op_mixed_numeric_width_requires_cast() {
        let tc = TypeChecker::new();
        let err = tc
            .check_binary_op(BinaryOp::Add, &Type::I32, &Type::I64, &Span::dummy())
            .expect_err("mixed integer arithmetic should fail");
        assert!(
            err.message
                .contains("arithmetic operands must have the same type"),
            "expected mixed numeric diagnostic, got {err:?}"
        );

        let err = tc
            .check_binary_op(BinaryOp::Mul, &Type::F32, &Type::F64, &Span::dummy())
            .expect_err("mixed float arithmetic should fail");
        assert!(
            err.message
                .contains("arithmetic operands must have the same type"),
            "expected mixed numeric diagnostic, got {err:?}"
        );
    }

    #[test]
    fn unknown_function_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "nonexistent".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: Vec::new(),
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|d| d.message.contains("undefined function")));
    }

    #[test]
    fn return_type_mismatch_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "foo".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::I32),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: Some(Box::new(Expression::Return {
                        value: Some(Box::new(Expression::StringLiteral {
                            value: "hello".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|d| d.message.contains("return type mismatch")));
    }

    #[test]
    fn function_call_wrong_arity_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    name: "add".into(),
                    type_params: Vec::new(),
                    params: vec![
                        ast::Param {
                            name: "a".into(),
                            ty: AstType::I32,
                            mutable: false,
                            span: Span::dummy(),
                        },
                        ast::Param {
                            name: "b".into(),
                            ty: AstType::I32,
                            mutable: false,
                            span: Span::dummy(),
                        },
                    ],
                    return_type: Some(AstType::I32),
                    body: Expression::Block {
                        statements: Vec::new(),
                        expr: Some(Box::new(Expression::Return {
                            value: Some(Box::new(Expression::Identifier {
                                name: "a".into(),
                                span: Span::dummy(),
                            })),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![ast::Statement::Expression {
                            expr: Expression::FunctionCall {
                                name: "add".into(),
                                module: None,
                                type_args: Vec::new(),
                                args: vec![Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                }],
                                span: Span::dummy(),
                            },
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("wrong arity should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("function `add` expects 2 arguments, found 1")),
            "expected arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn function_call_argument_type_mismatch_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    name: "takes_i32".into(),
                    type_params: Vec::new(),
                    params: vec![ast::Param {
                        name: "value".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    }],
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: Vec::new(),
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![ast::Statement::Expression {
                            expr: Expression::FunctionCall {
                                name: "takes_i32".into(),
                                module: None,
                                type_args: Vec::new(),
                                args: vec![Expression::StringLiteral {
                                    value: "bad".into(),
                                    span: Span::dummy(),
                                }],
                                span: Span::dummy(),
                            },
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("argument type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("argument 1 for `takes_i32` expects `i32`, found `str`")),
            "expected argument type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn struct_literal_missing_field_is_error() {
        use crate::ast::declarations::StructField;
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Struct {
                    name: "Point".into(),
                    type_params: Vec::new(),
                    fields: vec![
                        StructField {
                            name: "x".into(),
                            ty: AstType::I32,
                            default: None,
                            mutable: false,
                            span: Span::dummy(),
                        },
                        StructField {
                            name: "y".into(),
                            ty: AstType::I32,
                            default: None,
                            mutable: false,
                            span: Span::dummy(),
                        },
                    ],
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "p".into(),
                            ty: None,
                            value: Expression::StructLiteral {
                                name: "Point".into(),
                                type_args: Vec::new(),
                                fields: vec![(
                                    "x".into(),
                                    Expression::IntLiteral {
                                        value: 1,
                                        span: Span::dummy(),
                                    },
                                )],
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("missing struct field should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("missing field `y` for struct `Point`")),
            "expected missing field diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn struct_literal_field_type_mismatch_is_error() {
        use crate::ast::declarations::StructField;
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Struct {
                    name: "Point".into(),
                    type_params: Vec::new(),
                    fields: vec![StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    }],
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "p".into(),
                            ty: None,
                            value: Expression::StructLiteral {
                                name: "Point".into(),
                                type_args: Vec::new(),
                                fields: vec![(
                                    "x".into(),
                                    Expression::StringLiteral {
                                        value: "bad".into(),
                                        span: Span::dummy(),
                                    },
                                )],
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("struct field type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("field `x` for struct `Point` expects `i32`, found `str`")),
            "expected field type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_variant_unknown_variant_is_error() {
        let program = parse_program(
            r#"
Status: Ok, Err

main = () void {
    value = Status.Pending
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown enum variant should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
            "expected unknown variant diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_variant_payload_type_mismatch_is_error() {
        let program = parse_program(
            r#"
Maybe: Some(i32), None

main = () void {
    value = Maybe.Some("bad")
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum payload type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("payload for enum variant `Maybe.Some` expects `i32`, found `str`")),
            "expected payload type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn assignment_to_immutable_binding_is_error() {
        let program = parse_program(
            r#"
main = () void {
    x = 1
    x = 2
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("immutable assignment should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("cannot assign to immutable variable `x`")),
            "expected immutable assignment diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn assignment_type_mismatch_is_error() {
        let program = parse_program(
            r#"
main = () void {
    x ::= 1
    x = "bad"
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("assignment type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("assignment to `x` expects `i32`, found `str`")),
            "expected assignment type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn invalid_field_access_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

main = () void {
    p = Point { x: 1 }
    y = p.y
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("invalid field access should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("type `Point` has no field `y`")),
            "expected invalid field diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn implicit_integer_width_conversion_is_error() {
        let program = parse_program(
            r#"
take_i64 = (value: i64) void {}

main = () void {
    x: i32 = 1
    take_i64(x)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("implicit integer conversion should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("argument 1 for `take_i64` expects `i64`, found `i32`")),
            "expected integer conversion diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn implicit_float_width_conversion_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "take_f32".into(),
                type_params: Vec::new(),
                params: vec![ast::Param {
                    name: "value".into(),
                    ty: AstType::F32,
                    mutable: false,
                    span: Span::dummy(),
                }],
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        tc.collect_declarations(&program.declarations);

        let expected = tc.functions["take_f32"].params[0].1.clone();
        assert!(!tc.types_compatible(&tc.resolve_type(&expected), &Type::F64));
    }

    #[test]
    fn unknown_root_std_module_call_is_error() {
        let program = parse_program(
            r#"
{ io } = std

main = () void {
    io.nope("bad")
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown std module function should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("undefined module function `io.nope`")),
            "expected undefined module function diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn known_root_std_runtime_standins_remain_allowed() {
        let program = parse_program(
            r#"
{ io } = std

main = () void {
    io.print("hello")
    io.println("world")
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("temporary root std io stand-ins should typecheck");
    }

    #[test]
    fn non_void_function_without_return_is_error() {
        let program = parse_program(
            r#"
missing = () i32 {
    x = 1
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-void fallthrough should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("function `missing` must return `i32` on all non-error paths")),
            "expected missing return diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_missing_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-exhaustive enum match should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-exhaustive match on `Color`: missing `Blue`")),
            "expected non-exhaustive enum diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_duplicate_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Red { "again" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate enum match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("duplicate match arm for `Color.Red`")),
            "expected duplicate enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_unknown_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Blue { "blue" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown enum match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("enum `Color` has no variant `Blue`")),
            "expected unknown enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_payload_shape_is_checked() {
        let program = parse_program(
            r#"
Maybe: Some(i32), None

describe = (m: Maybe) StaticString {
    m ?
        | Some { "some" }
        | None(value) { "none" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum match payload shape should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("match arm `Maybe.Some` requires a payload")),
            "expected missing payload diagnostic, got {errors:?}"
        );
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("match arm `Maybe.None` does not accept a payload")),
            "expected forbidden payload diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_wildcard_after_all_variants_is_redundant() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
        | _ { "fallback" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("redundant enum wildcard arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("redundant wildcard match arm")),
            "expected redundant wildcard diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_variant_after_wildcard_is_redundant() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | _ { "fallback" }
        | Red { "red" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum variant after wildcard should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("redundant match arm for `Color.Red`")),
            "expected redundant enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn bool_match_missing_arm_is_error_for_value_match() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-exhaustive boolean value match should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-exhaustive bool match: missing `false`")),
            "expected non-exhaustive bool diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn bool_match_duplicate_arm_is_error() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
        | true { "again" }
        | false { "no" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate boolean match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("duplicate match arm for `true`")),
            "expected duplicate bool arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn match_arm_return_does_not_force_never_result_type() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { return "early" }
        | false { "late" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_program(&program)
            .expect("returning arm should not force match type to never");
        let body = &typed.functions[0].body;
        assert_eq!(body.ty, Type::Str);
    }

    #[test]
    fn types_compatible_basics() {
        let tc = TypeChecker::new();
        // Same types
        assert!(tc.types_compatible(&Type::I32, &Type::I32));
        // Numeric conversions require explicit casts except literal coercion.
        assert!(!tc.types_compatible(&Type::I64, &Type::I32));
        assert!(!tc.types_compatible(&Type::F32, &Type::F64));
        // Unknown is permissive
        assert!(tc.types_compatible(&Type::I32, &Type::Unknown));
        // Named types are nominal and do not match unrelated concrete types.
        assert!(tc.types_compatible(&Type::Named("UserId".into()), &Type::Named("UserId".into())));
        assert!(!tc.types_compatible(
            &Type::Named("UserId".into()),
            &Type::Named("OrderId".into())
        ));
        assert!(!tc.types_compatible(&Type::Str, &Type::Named("StaticString".into())));
        // Clear mismatch
        assert!(!tc.types_compatible(&Type::I32, &Type::Str));
        assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
    }

    #[test]
    fn literal_coercion_in_var_decl() {
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "x".into(),
                        ty: Some(AstType::I64),
                        value: Expression::IntLiteral {
                            value: 42,
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program).unwrap();
        // The variable should have type I64 (coerced from I32 literal)
        let body = &result.functions[0].body;
        match &body.statements[0].kind {
            TypedStatementKind::VarDecl { ty, .. } => assert_eq!(*ty, Type::I64),
            _ => panic!("expected VarDecl"),
        }
    }

    #[test]
    fn resolve_string_type() {
        let tc = TypeChecker::new();
        // "String" as a named type should resolve to Type::String
        assert_eq!(
            tc.resolve_type(&AstType::Named("String".into())),
            Type::String
        );
    }

    #[test]
    fn resolve_slice_type() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.resolve_type(&AstType::Slice(Box::new(AstType::I32))),
            Type::Slice(Box::new(Type::I32))
        );
    }

    #[test]
    fn infer_type_args_basic() {
        let tc = TypeChecker::new();
        // Generic function: identity<T>(x: T) -> T
        let type_params = vec!["T".to_string()];
        let params = vec![("x".to_string(), AstType::Named("T".into()))];
        let arg_types = vec![Type::I32];
        let subs = tc.infer_type_args(&type_params, &params, &arg_types);
        assert_eq!(subs.get("T"), Some(&Type::I32));
    }

    #[test]
    fn substitute_type_basic() {
        let tc = TypeChecker::new();
        let mut subs = HashMap::new();
        subs.insert("T".to_string(), Type::I32);
        // T → I32
        assert_eq!(
            tc.substitute_type(&AstType::Named("T".into()), &subs),
            Type::I32
        );
        // Ptr<T> → Ptr<I32>
        assert_eq!(
            tc.substitute_type(&AstType::Ptr(Box::new(AstType::Named("T".into()))), &subs),
            Type::Ptr(Box::new(Type::I32))
        );
        // Non-generic type unchanged
        assert_eq!(tc.substitute_type(&AstType::Bool, &subs), Type::Bool);
    }

    #[test]
    fn generic_function_collection() {
        use crate::ast::Expression;
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Function {
            name: "identity".into(),
            type_params: vec![crate::ast::declarations::TypeParam {
                name: "T".into(),
                constraint: None,
                constraint_type_args: Vec::new(),
                span: Span::dummy(),
            }],
            params: vec![crate::ast::Param {
                name: "x".into(),
                ty: AstType::Named("T".into()),
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Named("T".into())),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        let info = tc.functions.get("identity").unwrap();
        assert_eq!(info.type_params, vec!["T".to_string()]);
    }

    #[test]
    fn behavior_declaration_collection() {
        let program = parse_program(
            r#"
Serializable: behavior {
    to_json: (Self) String
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.collect_declarations(&program.declarations);
        let info = tc.behaviors.get("Serializable").unwrap();
        assert_eq!(info.name, "Serializable");
        assert_eq!(info.methods.len(), 1);
        assert_eq!(info.methods[0].name, "to_json");
    }

    #[test]
    fn behavior_impl_with_required_method_passes() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("valid behavior impl should typecheck");
    }

    #[test]
    fn behavior_impl_missing_required_method_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("missing behavior method should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `Json` is missing required method `to_json`"
            )),
            "expected missing behavior method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_can_omit_default_method() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str { return "{}" }
}

Point.implements(Json) {
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("behavior impl may omit a method with a default body");
    }

    #[test]
    fn behavior_impl_duplicate_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate behavior impl should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("duplicate implementation of behavior `Json` for type `Point`")),
            "expected duplicate behavior impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_without_type_args_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior impl without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior impl arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior impl should satisfy matching generic requires");
    }

    #[test]
    fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { return value }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior type argument bound should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` does not implement behavior `Json<Point>` required by `T`"
            )),
            "expected generic behavior type argument bound diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { return value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { return value }
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior type argument bound should pass when satisfied");
    }

    #[test]
    fn behavior_requires_generic_behavior_type_arg_arity_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, str>)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior requires arity mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 2")),
            "expected generic behavior requires arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_substitutes_method_signature() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) i32 { return 1 }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior impl return mismatch should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "method `encode` for behavior `Json_str` expects return `str`, found `i32`"
            )),
            "expected substituted behavior method return diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_overlapping_inherited_behavior_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("overlapping inherited behavior impl should fail");
        assert!(
            errors.iter().any(|d| {
                d.message.contains(
                    "overlapping implementations of behaviors `Json` and `PrettyJson` for type `Point`",
                )
            }),
            "expected overlapping behavior impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_requires_passes_when_impl_exists() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.requires(Json)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("requires should pass when behavior impl exists");
    }

    #[test]
    fn behavior_requires_rejects_missing_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.requires(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("requires should fail without behavior impl");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("type `Point` does not implement required behavior `Json`")),
            "expected requires missing impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_requires_generic_behavior_without_type_args_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.requires(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior requires without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior requires arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_requires_parent_methods() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("extended behavior should require parent methods");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `PrettyJson` is missing required method `to_json`"
            )),
            "expected inherited missing method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_impl_satisfies_parent_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("implementation of child behavior should satisfy parent requires");
    }

    #[test]
    fn behavior_extends_generic_parent_requires_substituted_methods() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic parent method should be required with substituted signature");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            )),
            "expected inherited generic parent missing method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_generic_parent_satisfies_specialized_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json<str>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("child behavior impl should satisfy specialized generic parent requires");
    }

    #[test]
    fn behavior_extends_generic_parent_accepts_child_type_parameter_arg() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior parent should accept child type parameter args");
    }

    #[test]
    fn behavior_impl_generic_parent_overlap_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("specialized parent and child behavior impls should overlap");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            )),
            "expected specialized behavior impl overlap diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { return value.x }
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("distinct behavior specializations should not overlap");
    }

    #[test]
    fn behavior_extends_cycle_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("cyclic behavior inheritance should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("behavior inheritance cycle")),
            "expected behavior inheritance cycle diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_duplicate_parent_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("duplicate behavior inheritance edge should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("duplicate behavior inheritance `PrettyJson.extends(Json)`")
            }),
            "expected duplicate behavior inheritance diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_duplicate_generic_parent_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Json<str>)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("duplicate specialized behavior inheritance edge should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("duplicate behavior inheritance `PrettyJson.extends(Json<str>)`")
            }),
            "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_generic_parent_without_type_args_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior extends parent without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior extends parent arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_conflicting_method_signature_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    to_json: (Self) i32
}

PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("conflicting inherited behavior method should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("conflicting behavior method `to_json` inherited by `PrettyJson`")
            }),
            "expected conflicting inherited behavior method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_signature_mismatch_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: i32) i32 { return value }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("behavior impl signature mismatch should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("parameter 1 for method `to_json`")),
            "expected behavior parameter mismatch diagnostic, got {errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("expects return `str`, found `i32`")),
            "expected behavior return mismatch diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_function_explicit_type_arg_arity_is_error() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T {
    return value
}

main = () i32 {
    return identity<i32, str>(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("wrong generic type-argument arity should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic function `identity` expects 1 type arguments, found 2")),
            "expected generic arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn nongeneric_function_explicit_type_args_are_error() {
        let program = parse_program(
            r#"
id = (value: i32) i32 {
    return value
}

main = () i32 {
    return id<i32>(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-generic function type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-generic function `id` does not accept type arguments")),
            "expected non-generic type-argument diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_function_inference_failure_is_error() {
        let program = parse_program(
            r#"
make_default<T> = () T {
    return 0
}

main = () i32 {
    return make_default()
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("uninferred generic type argument should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("cannot infer type argument `T` for generic function `make_default`")),
            "expected generic inference diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_bound_references_unknown_behavior_is_error() {
        let program = parse_program(
            r#"
show<T: Display> = (value: T) T {
    return value
}

main = () i32 {
    return show(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown generic behavior bounds should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "generic bound `Display` on type parameter `T` references undefined behavior"
            )),
            "expected generic bound diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_bound_rejects_unspecialized_generic_behavior() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("generic behavior bound without type arguments should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("generic behavior `Json` expects 1 type arguments, found 0")
            }),
            "expected generic behavior bound arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_behavior_bound_with_type_args_accepts_matching_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point { return value }
}

identity<T: Json<T>> = (value: T) T {
    return value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    return same.x
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior bound type argument should substitute at call site");
    }

    #[test]
    fn generic_behavior_bound_with_type_args_rejects_mismatched_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

identity<T: Json<T>> = (value: T) T {
    return value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    return same.x
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior bound should require matching behavior type args");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` does not implement behavior `Json<Point>` required by `T`"
            )),
            "expected generic behavior bound type argument diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_generic_bound_accepts_later_behavior_declaration() {
        let program = parse_program(
            r#"
Serializable<T: Json>: behavior {
    encode: (Self) str
}

Json: behavior {
    to_json: (Self) str
}

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("behavior generic bounds should be independent of declaration order");
    }

    #[test]
    fn generic_behavior_bound_accepts_type_with_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("type with behavior impl should satisfy generic bound");
    }

    #[test]
    fn generic_behavior_bound_accepts_inherited_behavior_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("child behavior impl should satisfy inherited generic bound");
    }

    #[test]
    fn generic_behavior_bound_rejects_type_without_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("type without behavior impl should not satisfy generic bound");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("type `Point` does not implement behavior `Json`")),
            "expected missing generic bound impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn func_info_non_generic_has_empty_type_params() {
        use crate::ast::Expression;
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Function {
            name: "add".into(),
            type_params: Vec::new(),
            params: vec![
                crate::ast::Param {
                    name: "a".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                },
                crate::ast::Param {
                    name: "b".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                },
            ],
            return_type: Some(AstType::I32),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        let info = tc.functions.get("add").unwrap();
        assert!(info.type_params.is_empty());
    }
}
