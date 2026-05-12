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
use crate::ast::{self, AstType, Declaration, Expression, Param, StructField};
use crate::error::{Diagnostic, Span};
use crate::resolver::{
    MethodSignatureMetadata, Namespace, SymbolTable, TypeParameterBoundMetadata,
};

// ── Type Environment ──────────────────────────────────────────────

/// Information about a struct type.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, AstType)>,
    pub type_params: Vec<String>,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<(String, Option<AstType>)>,
    pub type_params: Vec<String>,
}

/// Information about a function signature.
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BehaviorInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<ast::BehaviorMethod>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericFunctionTemplate {
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub body: Expression,
    pub span: Span,
}

struct ExpectedValueSignature {
    parameter_names: Vec<String>,
    parameter_type_names: Vec<String>,
    return_type_name: String,
    type_parameter_count: usize,
    type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
}

struct ExpectedTypeLikeSymbol {
    type_parameter_count: usize,
    type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
    is_public: Option<bool>,
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

fn type_param_bounds(type_params: &[ast::TypeParam]) -> HashMap<String, String> {
    type_params
        .iter()
        .filter_map(|param| {
            param
                .constraint
                .as_ref()
                .map(|bound| (param.name.clone(), bound.clone()))
        })
        .collect()
}

// ── TypeChecker ───────────────────────────────────────────────────

pub struct TypeChecker {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    methods: HashMap<String, FuncInfo>, // key: "TypeName.method_name"
    behaviors: HashMap<String, BehaviorInfo>,
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
        }
    }

    /// Type-check a program and produce a TypedProgram.
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);

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
                    behavior: Some(_),
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
                            match self.check_function(&full_name, params, return_type, body, span) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
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
        self.check_program(program)
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
                self.validate_generic_bounds(type_params);
                self.behaviors.insert(
                    name.clone(),
                    BehaviorInfo {
                        name: name.clone(),
                        type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        methods: methods.clone(),
                    },
                );
            }
        }

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
                            GenericFunctionTemplate {
                                type_params: collected_type_params,
                                params: params.clone(),
                                return_type: return_type.clone(),
                                body: body.clone(),
                                span: *span,
                            },
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
                            GenericFunctionTemplate {
                                type_params: collected_type_params,
                                params: params.clone(),
                                return_type: return_type.clone(),
                                body: body.clone(),
                                span: *span,
                            },
                        );
                    }
                }
                Declaration::Behavior { type_params, .. } => {
                    self.validate_generic_bounds(type_params);
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(_),
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            type_params,
                            params,
                            return_type,
                            ..
                        } = method
                        {
                            self.validate_generic_bounds(type_params);
                            let key = format!("{}.{}", type_name, name);
                            self.methods.insert(
                                key.clone(),
                                FuncInfo {
                                    name: key,
                                    params: params
                                        .iter()
                                        .map(|p| (p.name.clone(), p.ty.clone()))
                                        .collect(),
                                    return_type: return_type.clone().unwrap_or(AstType::Void),
                                    type_params: type_params
                                        .iter()
                                        .map(|tp| tp.name.clone())
                                        .collect(),
                                    type_param_bounds: type_param_bounds(type_params),
                                },
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        for decl in decls {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                methods,
                span,
                ..
            } = decl
            {
                self.check_behavior_impl(type_name, behavior, methods, *span);
            }
        }
    }

    fn check_behavior_impl(
        &mut self,
        type_name: &str,
        behavior: &str,
        methods: &[Declaration],
        span: Span,
    ) {
        if !self
            .behavior_impls
            .insert((type_name.to_string(), behavior.to_string()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for type `{}`",
                    behavior, type_name
                ),
                span,
            ));
            return;
        }

        let Some(info) = self.behaviors.get(behavior).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "E6000",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return;
        };

        for required in &info.methods {
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
                        type_name, behavior, required.name
                    ),
                    span,
                ));
                continue;
            };

            let (actual_params, actual_return_type, actual_span) = actual;
            if actual_params.len() != required.params.len() {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects {} parameters, found {}",
                        required.name,
                        behavior,
                        required.params.len(),
                        actual_params.len()
                    ),
                    actual_span,
                ));
                continue;
            }

            for (idx, (expected, actual)) in
                required.params.iter().zip(actual_params.iter()).enumerate()
            {
                if !self.impl_ast_types_compatible(&expected.ty, &actual.ty, type_name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E6002",
                        format!(
                            "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
                            idx + 1,
                            required.name,
                            behavior,
                            self.impl_type_display(&expected.ty, type_name),
                            actual.ty.display_name()
                        ),
                        actual.span,
                    ));
                }
            }

            let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
            let actual_return = actual_return_type.as_ref().unwrap_or(&AstType::Void);
            if !self.impl_ast_types_compatible(expected_return, actual_return, type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                        required.name,
                        behavior,
                        self.impl_type_display(expected_return, type_name),
                        actual_return.display_name()
                    ),
                    actual_span,
                ));
            }
        }
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
                }
            }
        }
    }

    pub(crate) fn check_generic_bounds(
        &mut self,
        bounds: &HashMap<String, String>,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        for (param, behavior) in bounds {
            let Some(concrete) = substitutions.get(param) else {
                continue;
            };
            let Some(type_name) = self.behavior_bound_type_name(concrete) else {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        concrete.display_name(),
                        behavior,
                        param
                    ),
                    span,
                ));
                continue;
            };
            if !self
                .behavior_impls
                .contains(&(type_name.clone(), behavior.clone()))
            {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        type_name, behavior, param
                    ),
                    span,
                ));
            }
        }
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
                    self.require_resolver_parameter_locals(symbols, params);
                    self.require_resolver_expr_locals(symbols, body);
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
                    self.require_resolver_value_symbol(
                        symbols,
                        &format!("{type_name}.{method_name}"),
                        expected_value_signature(params, return_type, type_params),
                        *public,
                        *span,
                    );
                    self.require_resolver_parameter_locals(symbols, params);
                    self.require_resolver_expr_locals(symbols, body);
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
                        expected_field_type_names(fields),
                        *span,
                    );
                    for field in fields {
                        if let Some(default) = &field.default {
                            self.require_resolver_expr_locals(symbols, default);
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
                    self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Type,
                        name,
                        expected_type_like_symbol(type_params, Some(*public)),
                        *span,
                    );
                    for variant in variants {
                        let Some(symbol) = symbols.lookup(Namespace::Variant, &variant.name) else {
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
                        self.validate_resolver_variant_visibility(
                            symbol,
                            &variant.name,
                            *public,
                            variant.span,
                        );
                        self.validate_resolver_variant_payload_type(
                            symbol,
                            &variant.name,
                            expected_variant_payload_type_name(&variant.payload),
                            variant.span,
                        );
                    }
                }
                Declaration::Behavior {
                    name,
                    type_params,
                    methods,
                    span,
                    ..
                } => {
                    let Some(symbol) = self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Behavior,
                        name,
                        expected_type_like_symbol(type_params, None),
                        *span,
                    ) else {
                        continue;
                    };
                    self.validate_resolver_behavior_method_signatures(
                        symbol,
                        name,
                        expected_behavior_method_signatures(methods),
                        *span,
                    );
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            self.require_resolver_parameter_locals(symbols, &method.params);
                            self.require_resolver_expr_locals(symbols, default_body);
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
                    methods,
                    span,
                    ..
                } => {
                    self.require_resolver_type_like_symbol(
                        symbols,
                        Namespace::Type,
                        type_name,
                        ExpectedTypeLikeSymbol {
                            type_parameter_count: 0,
                            type_parameter_bounds: Vec::new(),
                            is_public: None,
                        },
                        *span,
                    );
                    if let Some(behavior) = behavior {
                        self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
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
                            self.require_resolver_parameter_locals(symbols, params);
                            self.require_resolver_expr_locals(symbols, body);
                        }
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.require_resolver_expr_locals(symbols, expr);
                }
                Declaration::Error { .. } => {}
            }
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

    fn require_resolver_symbol(
        &mut self,
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        if symbols.lookup(namespace, name).is_none() {
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
    }

    fn require_resolver_parameter_locals(&mut self, symbols: &SymbolTable, params: &[Param]) {
        for param in params {
            self.require_resolver_local_symbol(symbols, &param.name, param.mutable, param.span);
        }
    }

    fn require_resolver_expr_locals(&mut self, symbols: &SymbolTable, expr: &Expression) {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.require_resolver_expr_locals(symbols, left);
                self.require_resolver_expr_locals(symbols, right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.require_resolver_expr_locals(symbols, operand);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.require_resolver_expr_locals(symbols, receiver);
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.require_resolver_expr_locals(symbols, object);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.require_resolver_expr_locals(symbols, object);
                self.require_resolver_expr_locals(symbols, index);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.require_resolver_expr_locals(symbols, value);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.require_resolver_expr_locals(symbols, payload);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.require_resolver_expr_locals(symbols, element);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.require_resolver_expr_locals(symbols, scrutinee);
                for arm in arms {
                    self.require_resolver_pattern_locals(symbols, &arm.pattern);
                    self.require_resolver_expr_locals(symbols, &arm.body);
                    if let Some(guard) = &arm.guard {
                        self.require_resolver_expr_locals(symbols, guard);
                    }
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition);
                self.require_resolver_expr_locals(symbols, body);
            }
            Expression::Loop { body, .. } => {
                self.require_resolver_expr_locals(symbols, body);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition);
                self.require_resolver_expr_locals(symbols, then_body);
                if let Some(else_body) = else_body {
                    self.require_resolver_expr_locals(symbols, else_body);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.require_resolver_statement_locals(symbols, statement);
                }
                if let Some(expr) = expr {
                    self.require_resolver_expr_locals(symbols, expr);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value);
                }
            }
            Expression::Closure { params, body, .. } => {
                self.require_resolver_parameter_locals(symbols, params);
                self.require_resolver_expr_locals(symbols, body);
            }
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.require_resolver_expr_locals(symbols, expr);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start);
                self.require_resolver_expr_locals(symbols, end);
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
    ) {
        match statement {
            ast::Statement::VarDecl {
                name,
                ty,
                value,
                mutable,
                span,
                ..
            } => {
                let reassignment_form = !*mutable
                    && ty.is_none()
                    && symbols.symbols().iter().any(|symbol| {
                        symbol.namespace == Namespace::Local
                            && symbol.name == *name
                            && symbol.is_mutable == Some(true)
                    });
                if !reassignment_form {
                    self.require_resolver_local_symbol(symbols, name, *mutable, *span);
                }
                self.require_resolver_expr_locals(symbols, value);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.require_resolver_expr_locals(symbols, target);
                self.require_resolver_expr_locals(symbols, value);
            }
            ast::Statement::Expression { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.require_resolver_statement_locals(symbols, statement);
                }
            }
        }
    }

    fn require_resolver_pattern_locals(&mut self, symbols: &SymbolTable, pattern: &ast::Pattern) {
        match pattern {
            ast::Pattern::Identifier { name, span } => {
                self.require_resolver_local_symbol(symbols, name, false, *span);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested);
                    } else {
                        self.require_resolver_local_symbol(symbols, name, false, *span);
                    }
                }
            }
            ast::Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.require_resolver_pattern_locals(symbols, payload);
            }
            ast::Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.require_resolver_pattern_locals(symbols, pattern);
                }
            }
            ast::Pattern::Literal { value, .. } => {
                self.require_resolver_expr_locals(symbols, value);
            }
            ast::Pattern::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start);
                self.require_resolver_expr_locals(symbols, end);
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
        span: Span,
    ) {
        let matching_symbols: Vec<_> = symbols
            .symbols()
            .iter()
            .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == name)
            .collect();

        if matching_symbols.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E0228",
                format!("resolver symbol table missing local symbol '{name}'"),
                span,
            ));
            return;
        };

        if !matching_symbols
            .iter()
            .any(|symbol| symbol.is_mutable == Some(expected_mutable))
        {
            let actual = match matching_symbols[0].is_mutable {
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

        Some(symbol)
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
        expected_field_type_names: Vec<(String, String)>,
        span: Span,
    ) {
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
        expected_payload_type_name: Option<String>,
        span: Span,
    ) {
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

    fn validate_resolver_behavior_method_signatures(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_method_signatures: Vec<MethodSignatureMetadata>,
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
        parameter_type_names: expected_parameter_type_names(params),
        return_type_name: expected_return_type_name(return_type),
        type_parameter_count: type_params.len(),
        type_parameter_bounds: expected_type_parameter_bounds(type_params),
    }
}

fn expected_type_parameter_bounds(
    type_params: &[ast::TypeParam],
) -> Vec<TypeParameterBoundMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param
                .constraint
                .as_ref()
                .map(|constraint| (type_param.name.clone(), constraint.clone()))
        })
        .collect()
}

fn expected_type_like_symbol(
    type_params: &[ast::TypeParam],
    is_public: Option<bool>,
) -> ExpectedTypeLikeSymbol {
    ExpectedTypeLikeSymbol {
        type_parameter_count: type_params.len(),
        type_parameter_bounds: expected_type_parameter_bounds(type_params),
        is_public,
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

fn format_parameter_type_names(names: Option<&[String]>) -> String {
    match names {
        Some(names) => format!("({})", names.join(", ")),
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
