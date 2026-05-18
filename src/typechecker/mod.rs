//! Typechecker — transforms untyped AST → TypedProgram.
//!
//! Pipeline:
//! 1. **Collect**: Register all struct/enum/function/behavior signatures
//! 2. **Resolve**: Resolve type references (Named("Foo") → Struct fields)
//! 3. **Check**: Type-check function bodies, produce TypedExpression
//!
//! The typechecker NEVER defaults unknown types to I32. If a type can't be
//! resolved, it's an error.

mod behavior_associations;
mod behavior_impl_methods;
mod behavior_impl_support;
mod behavior_impl_validation;
mod behavior_ref_metadata;
mod closures;
mod declaration_collection;
mod declaration_collection_ast;
mod declaration_collection_ast_callables;
mod declaration_collection_resolver_semantic_tasks;
mod declaration_collection_resolver_tasks;
mod environment;
mod expressions;
mod gated_intrinsics;
mod generic_bound_validation;
mod generic_type_reference_walker;
mod generic_type_validation;
mod import_roots;
mod monomorphize;
mod monomorphize_dependencies;
mod monomorphize_inference;
mod monomorphize_specialized_types;
mod monomorphize_types;
mod patterns;
mod program_checking;
mod resolve;
mod resolver_backed_collection;
mod resolver_lookup;
mod resolver_metadata_collection;
mod resolver_validation;
mod scope_management;
mod self_type_validation;
mod semantic_validation;
mod statements;
mod std_runtime_calls;

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::typed::*;
use crate::ast::{
    self, AstType, BehaviorMethod, Declaration, EnumVariant, Expression, Param, StructField,
};
use crate::error::{Diagnostic, Span};
use crate::module_system::{ResolvedModule, ResolvedModuleGraph};
use crate::resolver::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

pub use environment::{BehaviorBound, BehaviorInfo, EnumInfo, FuncInfo, StructInfo};
pub(crate) use environment::{
    GenericFunctionTemplate, SourceModuleDependencies, TemplateDependencyEntry,
    TemplateDependencyState,
};

include!("declaration_tasks.rs");
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
    behavior_refs_by_key: HashMap<String, BehaviorParentRef>,
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
    resolver_behavior_impl_refs: HashMap<String, VecDeque<BehaviorRefMetadata>>,
    resolver_behavior_required_refs: HashMap<String, VecDeque<BehaviorRefMetadata>>,
    resolver_missing_behavior_impl_refs: HashSet<String>,
    resolver_missing_behavior_required_refs: HashSet<String>,
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
            behavior_refs_by_key: HashMap::new(),
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
            resolver_behavior_impl_refs: HashMap::new(),
            resolver_behavior_required_refs: HashMap::new(),
            resolver_missing_behavior_impl_refs: HashSet::new(),
            resolver_missing_behavior_required_refs: HashSet::new(),
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
        let key = Self::method_key(type_name, name);
        self.methods.insert(
            key.clone(),
            func_info_from_ast_signature(key.clone(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, *span)
        {
            self.generic_methods.insert(key, template);
        }
    }

    fn collect_resolver_backed_impl_method_template(
        &mut self,
        type_name: &str,
        method: &Declaration,
    ) {
        let Declaration::Function {
            name,
            type_params,
            params,
            body,
            span,
            ..
        } = method
        else {
            return;
        };
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, *span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, name), template);
        }
    }

    fn collect_resolver_behavior_impl_method_signatures(
        &mut self,
        symbols: &SymbolTable,
        ast_type_name: &str,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        let mut required_methods: VecDeque<ast::BehaviorMethod> = self
            .behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .into_iter()
            .collect();

        for method in methods {
            let Declaration::Function { name, span, .. } = method else {
                continue;
            };
            let ast_key = Self::method_key(ast_type_name, name);
            let resolver_owned_key =
                self.resolver_backed_impl_method_key(Some(symbols), &ast_key, type_name, *span);
            let restored_name = self.resolver_backed_behavior_impl_method_signature_name(
                &mut required_methods,
                name,
                resolver_owned_key.as_deref(),
                type_name,
            );
            let Some(restored_name) = restored_name else {
                continue;
            };
            let restored_key =
                resolver_owned_key.unwrap_or_else(|| Self::method_key(type_name, &restored_name));
            self.collect_resolver_callable_signature_for_key(
                symbols,
                &ast_key,
                &restored_key,
                *span,
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
        if self.should_skip_behavior_default_synthesis(type_name) {
            return;
        }
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        for default in
            self.behavior_default_methods_for_impl(type_name, behavior, behavior_type_args, methods)
        {
            self.seed_behavior_default_method_signature(type_name, &default);
        }
    }

    fn should_skip_behavior_default_synthesis(&self, type_name: &str) -> bool {
        self.resolver_backed_collection
            && self.resolver_missing_behavior_impl_refs.contains(type_name)
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

        if info.type_params.is_empty() && !type_args.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic behavior `{}` does not accept type arguments",
                    behavior
                ),
                span,
            ));
            return None;
        }

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
}

include!("resolver_callable_signatures.rs");

#[cfg(test)]
mod tests;
