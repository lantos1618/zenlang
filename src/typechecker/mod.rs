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
mod behavior_impl_support;
mod behavior_impl_validation;
mod closures;
mod environment;
mod expressions;
mod generic_type_validation;
mod monomorphize;
mod patterns;
mod program_checking;
mod resolve;
mod resolver_validation;
mod self_type_validation;
mod statements;

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

    // ── Phase 1: Collect ──────────────────────────────────────────

    fn collect_declarations(&mut self, decls: &[Declaration]) {
        let tasks = Self::collect_ast_declaration_collection_tasks(decls);
        self.collect_behavior_declarations_from_tasks(&tasks.behaviors);
        self.validate_ast_precollection_tasks(&tasks.precollection_validations);
        if !self.resolver_backed_collection {
            self.collect_ast_type_declarations_from_tasks(&tasks.types);
        }
        self.collect_callable_declarations_from_tasks(&tasks.callable);
        self.collect_impl_block_declarations_from_tasks(&tasks.impl_blocks);
        self.collect_ast_import_declarations_from_tasks(&tasks.imports);
    }

    fn collect_ast_declaration_collection_tasks(
        decls: &[Declaration],
    ) -> AstDeclarationCollectionTasks<'_> {
        let mut tasks = AstDeclarationCollectionTasks::default();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks.behaviors);
            Self::push_ast_type_declaration_task(decl, &mut tasks.types);
            Self::push_callable_declaration_task(decl, &mut tasks.callable);
            Self::push_impl_block_declaration_task(decl, &mut tasks.impl_blocks);
            Self::push_ast_import_declaration_task(decl, &mut tasks.imports);
            Self::push_self_type_context_validation_task(
                decl,
                &mut tasks.precollection_validations.self_type_contexts,
            );
            Self::push_behavior_extends_replay_task(
                decl,
                &mut tasks
                    .precollection_validations
                    .behavior_associations
                    .extends,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_ast_import_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstImportDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_import_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_import_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstImportDeclarationTask<'a>>,
    ) {
        if let Declaration::Import {
            names, module_path, ..
        } = decl
        {
            tasks.push(AstImportDeclarationTask { names, module_path });
        }
    }

    fn collect_ast_import_declarations_from_tasks(
        &mut self,
        tasks: &[AstImportDeclarationTask<'_>],
    ) {
        for task in tasks {
            for name in task.names {
                self.imports.insert(name.clone(), task.module_path.to_vec());
            }
        }
    }

    #[cfg(test)]
    fn collect_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ImplBlockDeclarationTask<'a>>,
    ) {
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            behavior_type_args,
            methods,
            ..
        } = decl
        {
            tasks.push(ImplBlockDeclarationTask {
                type_name,
                behavior: behavior.as_deref(),
                behavior_type_args,
                methods,
            });
        }
    }

    fn collect_impl_block_declarations_from_tasks(
        &mut self,
        tasks: &[ImplBlockDeclarationTask<'_>],
    ) {
        for task in tasks {
            if self.resolver_backed_collection {
                self.collect_resolver_backed_impl_block_templates(task.type_name, task.methods);
            } else {
                self.collect_ast_impl_block_declaration(
                    task.type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        }
    }

    fn collect_ast_impl_block_declaration(
        &mut self,
        type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
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

    fn collect_resolver_backed_impl_block_templates(
        &mut self,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            self.collect_resolver_backed_impl_method_template(type_name, method);
        }
    }

    #[cfg(test)]
    fn collect_callable_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<CallableDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_callable_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_callable_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<CallableDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            _ => {}
        }
    }

    fn collect_callable_declarations_from_tasks(&mut self, tasks: &[CallableDeclarationTask<'_>]) {
        for task in tasks {
            match task {
                CallableDeclarationTask::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_function_template(
                            name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        self.collect_ast_function_declaration(
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
                CallableDeclarationTask::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_method_template(
                            type_name,
                            method_name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        let key = Self::method_key(type_name, method_name);
                        self.collect_ast_method_declaration(
                            &key,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
            }
        }
    }

    fn collect_ast_function_declaration(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.functions.insert(
            name.to_string(),
            func_info_from_ast_signature(name.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    fn collect_ast_method_declaration(
        &mut self,
        key: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.methods.insert(
            key.to_string(),
            func_info_from_ast_signature(key.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_methods.insert(key.to_string(), template);
        }
    }

    fn collect_resolver_backed_function_template(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    fn collect_resolver_backed_method_template(
        &mut self,
        type_name: &str,
        method_name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, method_name), template);
        }
    }

    #[cfg(test)]
    fn collect_ast_type_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_type_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeDeclarationTask::Struct {
                name,
                type_params,
                fields,
            }),
            Declaration::Enum {
                name,
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeDeclarationTask::Enum {
                name,
                type_params,
                variants,
            }),
            _ => {}
        }
    }

    fn collect_ast_type_declarations_from_tasks(&mut self, tasks: &[AstTypeDeclarationTask<'_>]) {
        for task in tasks {
            match task {
                AstTypeDeclarationTask::Struct {
                    name,
                    type_params,
                    fields,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.structs.insert(
                        (*name).to_string(),
                        struct_info_from_ast_fields((*name).to_string(), type_params, fields),
                    );
                }
                AstTypeDeclarationTask::Enum {
                    name,
                    type_params,
                    variants,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.enums.insert(
                        (*name).to_string(),
                        enum_info_from_ast_variants((*name).to_string(), type_params, variants),
                    );
                }
            }
        }
    }

    #[cfg(test)]
    fn collect_behavior_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<BehaviorDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_behavior_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorDeclarationTask<'a>>,
    ) {
        if let Declaration::Behavior {
            name,
            type_params,
            methods,
            ..
        } = decl
        {
            tasks.push(BehaviorDeclarationTask {
                name,
                type_params,
                methods,
            });
        }
    }

    fn collect_behavior_declarations_from_tasks(&mut self, tasks: &[BehaviorDeclarationTask<'_>]) {
        let mut type_params_to_validate = Vec::new();

        for task in tasks {
            let BehaviorDeclarationTask {
                name,
                type_params,
                methods,
            } = task;

            if self.resolver_backed_collection {
                self.collect_resolver_backed_behavior_declaration_stub(name, methods);
            } else {
                self.collect_ast_behavior_declaration_signature(name, type_params, methods);
                type_params_to_validate.push(type_params);
            }
        }

        for type_params in type_params_to_validate {
            self.validate_generic_bounds(type_params);
        }
    }

    fn collect_ast_behavior_declaration_signature(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        methods: &[BehaviorMethod],
    ) {
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_ast_methods(name.to_string(), type_params, methods),
        );
    }

    fn collect_resolver_backed_behavior_declaration_stub(
        &mut self,
        name: &str,
        methods: &[BehaviorMethod],
    ) {
        self.behaviors.insert(
            name.to_string(),
            behavior_info_for_resolver_backed_stub(name.to_string(), methods),
        );
    }

    fn validate_ast_precollection_tasks(&mut self, tasks: &AstPrecollectionValidationTasks<'_>) {
        self.validate_self_type_context_tasks(&tasks.self_type_contexts);

        if self.resolver_backed_collection {
            return;
        }

        self.validate_ast_behavior_extends_tasks(&tasks.behavior_associations);
    }

    #[cfg(test)]
    fn collect_ast_precollection_validation_tasks(
        decls: &[Declaration],
    ) -> AstPrecollectionValidationTasks<'_> {
        let mut tasks = AstPrecollectionValidationTasks::default();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks.self_type_contexts);
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
        }
        tasks
    }

    fn validate_ast_behavior_extends_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
    ) {
        self.validate_behavior_extends_tasks(tasks);
        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();
    }

    fn push_behavior_extends_replay_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorExtendsValidationTask<'a>>,
    ) -> bool {
        if let Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            span,
        } = decl
        {
            tasks.push(BehaviorExtendsValidationTask {
                behavior,
                parent,
                parent_type_args,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn validate_behavior_extends_tasks(&mut self, tasks: &BehaviorAssociationValidationTasks<'_>) {
        for task in &tasks.extends {
            self.check_behavior_extends(
                task.behavior,
                task.parent,
                task.parent_type_args,
                task.span,
            );
        }
    }

    fn collect_declarations_with_symbols(&mut self, decls: &[Declaration], symbols: &SymbolTable) {
        self.with_resolver_backed_collection(|checker| checker.collect_declarations(decls));

        let tasks = Self::collect_resolver_declaration_metadata_tasks(decls);
        self.collect_resolver_declaration_metadata(symbols, &tasks);
        self.collect_resolver_behavior_impl_metadata(&tasks, symbols);
        self.validate_resolver_collected_declaration_semantics(symbols, &tasks);
        self.clear_resolver_behavior_ref_state();
        self.refresh_resolver_type_behavior_impls(&tasks, symbols);
    }

    fn collect_resolver_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        let mut tasks = ResolverDeclarationMetadataTasks::default();
        for decl in decls {
            let callable_handled = Self::push_resolver_callable_replay_tasks(
                decl,
                &mut tasks.callable,
                &mut tasks.type_references,
            );
            let type_handled = if callable_handled {
                false
            } else {
                Self::push_resolver_type_replay_tasks(
                    decl,
                    &mut tasks.types,
                    &mut tasks.type_references,
                )
            };
            let behavior_handled = if callable_handled || type_handled {
                false
            } else {
                Self::push_resolver_behavior_replay_tasks(
                    decl,
                    &mut tasks.behaviors,
                    &mut tasks.type_references,
                )
            };
            let behavior_impl_handled = if callable_handled || type_handled || behavior_handled {
                false
            } else {
                Self::push_resolver_behavior_impl_replay_tasks(
                    decl,
                    &mut tasks.behavior_associations.impls,
                    &mut tasks.type_references,
                )
            };
            if !callable_handled && !type_handled && !behavior_handled && !behavior_impl_handled {
                Self::push_resolver_type_reference_validation_task(
                    decl,
                    &mut tasks.type_references,
                );
            }
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
            Self::push_behavior_requires_replay_task(
                decl,
                &mut tasks.behavior_associations.requires,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_resolver_type_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_type_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
            }
            _ => {}
        }
    }

    fn push_resolver_type_replay_tasks<'a>(
        decl: &'a Declaration,
        type_tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                true
            }
            Declaration::Enum { name, span, .. } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn collect_resolver_behavior_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_behavior_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_behavior_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    ) {
        if let Declaration::Behavior { name, span, .. } = decl {
            tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
        }
    }

    fn push_resolver_behavior_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior {
            name,
            methods,
            span,
            ..
        } = decl
        {
            behavior_tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
            type_reference_tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                name,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn push_resolver_behavior_impl_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_impl_tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        let handled = Self::push_behavior_impl_block_declaration_task(decl, behavior_impl_tasks);
        if handled {
            let Declaration::ImplBlock {
                type_name, methods, ..
            } = decl
            else {
                return false;
            };
            type_reference_tasks
                .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
        }
        handled
    }

    #[cfg(test)]
    fn collect_resolver_behavior_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_behavior_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        {
            tasks.push(ResolverBehaviorImplBlockDeclarationTask {
                ast_type_name: type_name,
                behavior,
                behavior_type_args,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    fn collect_resolver_callable_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverCallableDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_callable_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_callable_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Function { name, span, .. } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
            }
            Declaration::Method {
                type_name,
                method_name,
                span,
                ..
            } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
            }
            _ => {}
        }
    }

    fn push_resolver_callable_replay_tasks<'a>(
        decl: &'a Declaration,
        callable_tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                callable_tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn collect_resolver_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_resolver_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
            }
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
            }
            Declaration::Behavior {
                name,
                methods,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span: *span,
                });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    fn collect_resolver_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.collect_resolver_callable_declaration_metadata(symbols, tasks);
        self.collect_resolver_type_declaration_metadata(symbols, tasks);
        self.collect_resolver_behavior_declaration_metadata_pass(symbols, tasks);
    }

    fn collect_resolver_behavior_declaration_metadata_pass(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.behaviors {
            self.collect_resolver_behavior_declaration(symbols, task.name, task.span);
        }
    }

    fn collect_resolver_type_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.types {
            match task {
                ResolverTypeDeclarationMetadataTask::Struct { name, fields, span } => {
                    self.collect_resolver_struct_declaration_metadata(symbols, name, fields, *span);
                }
                ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    self.collect_resolver_enum_declaration_metadata(symbols, name, *span);
                }
            }
        }
    }

    fn collect_resolver_callable_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.callable {
            match task {
                ResolverCallableDeclarationMetadataTask::Function { name, span } => {
                    self.collect_resolver_function_signature(symbols, name, *span);
                }
                ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span,
                } => {
                    self.collect_resolver_method_signature(symbols, type_name, method_name, *span);
                }
                ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods } => {
                    self.collect_resolver_type_impl_declaration_metadata(
                        symbols, type_name, methods,
                    );
                }
            }
        }
    }

    fn collect_resolver_type_impl_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, span, .. } = method {
                self.collect_resolver_method_signature(symbols, type_name, name, *span);
            }
        }
    }

    fn collect_resolver_struct_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_struct_fields(symbols, name, fields);
            },
        );
    }

    fn collect_resolver_enum_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_enum_variants(symbols, name);
            },
        );
    }

    fn collect_resolver_type_declaration_metadata_for(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        collect: impl FnOnce(&mut Self, &str),
    ) {
        let restored_name =
            self.collect_resolver_type_behavior_refs_for_declaration(symbols, name, span);
        collect(self, &restored_name);
    }

    fn collect_resolver_behavior_impl_metadata(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let impl_tasks = self.resolver_behavior_impl_block_tasks(tasks, symbols);

        self.with_resolver_backed_collection(|checker| {
            for task in &impl_tasks {
                checker.collect_resolver_behavior_impl_method_signatures(
                    symbols,
                    task.ast_type_name,
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }

            checker.validate_collected_behavior_extends_semantics();

            for task in &impl_tasks {
                checker.collect_behavior_default_method_signatures(
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        });
    }

    fn validate_resolver_collected_declaration_semantics(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.with_resolver_backed_collection(|checker| {
            checker.validate_behavior_association_tasks(tasks, Some(symbols));
            checker.validate_resolver_type_reference_tasks(tasks, Some(symbols));
            checker.validate_resolver_struct_field_default_tasks(tasks, Some(symbols));
        });
    }

    fn clear_resolver_behavior_ref_state(&mut self) {
        self.resolver_behavior_impl_refs.clear();
        self.resolver_behavior_required_refs.clear();
        self.resolver_missing_behavior_impl_refs.clear();
        self.resolver_missing_behavior_required_refs.clear();
    }

    fn refresh_resolver_type_behavior_impls(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        for task in self.resolver_type_behavior_refresh_tasks(tasks, symbols) {
            self.collect_resolver_type_behavior_impls(symbols, &task.restored_name);
        }
    }

    fn with_resolver_backed_collection(&mut self, collect: impl FnOnce(&mut Self)) {
        let previous = self.resolver_backed_collection;
        self.resolver_backed_collection = true;
        collect(self);
        self.resolver_backed_collection = previous;
    }

    fn resolver_behavior_impl_block_tasks<'a>(
        &self,
        tasks: &'a ResolverDeclarationMetadataTasks<'a>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverBehaviorImplBlockTask<'a>> {
        let mut impl_tasks = Vec::new();
        for raw_task in &tasks.behavior_associations.impls {
            let restored_type_name = self.resolver_impl_type_name_for(
                symbols,
                raw_task.ast_type_name,
                raw_task.methods,
                Some((raw_task.behavior, raw_task.behavior_type_args)),
            );
            impl_tasks.push(ResolverBehaviorImplBlockTask {
                ast_type_name: raw_task.ast_type_name,
                restored_type_name,
                behavior: raw_task.behavior,
                behavior_type_args: raw_task.behavior_type_args,
                methods: raw_task.methods,
            });
        }
        impl_tasks
    }

    fn resolver_type_behavior_refresh_tasks(
        &self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverTypeBehaviorRefreshTask> {
        let mut refresh_tasks = Vec::new();
        for type_task in &tasks.types {
            match type_task {
                ResolverTypeDeclarationMetadataTask::Struct { name, span, .. }
                | ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    let restored_name =
                        Self::resolver_symbol_name_for(symbols, Namespace::Type, name, *span);
                    refresh_tasks.push(ResolverTypeBehaviorRefreshTask { restored_name });
                }
            }
        }
        refresh_tasks
    }

    fn collect_resolver_type_behavior_refs_for_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) -> String {
        let restored_name = Self::resolver_symbol_name_for(symbols, Namespace::Type, name, span);
        self.collect_resolver_type_behavior_impl_refs(symbols, &restored_name);
        self.collect_resolver_type_behavior_requires(symbols, &restored_name);
        restored_name
    }

    fn collect_resolver_behavior_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name =
            Self::resolver_symbol_name_for(symbols, Namespace::Behavior, name, span);
        self.rekey_behavior_declaration(name, &restored_name);
        self.collect_resolver_behavior_methods(symbols, &restored_name);
        self.collect_resolver_behavior_parents(symbols, &restored_name);
    }

    fn rekey_behavior_declaration(&mut self, old_name: &str, new_name: &str) {
        if old_name == new_name {
            return;
        }
        if let Some(info) = self.behaviors.remove(old_name) {
            self.behaviors.insert(
                new_name.to_string(),
                BehaviorInfo {
                    name: new_name.to_string(),
                    ..info
                },
            );
        }
    }

    fn validate_collected_declaration_semantics(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_semantic_validation_tasks(decls);
            self.validate_behavior_association_tasks(&tasks, symbols);
            self.validate_resolver_type_reference_tasks(&tasks, symbols);
            self.validate_resolver_struct_field_default_tasks(&tasks, symbols);
            return;
        }

        let tasks = Self::collect_ast_declaration_validation_tasks(decls);
        self.validate_behavior_association_tasks(&tasks.behavior_associations, symbols);
        self.validate_ast_type_reference_tasks(&tasks.type_references);
        self.validate_ast_struct_field_default_tasks(&tasks.struct_field_defaults);
    }

    fn collect_ast_declaration_validation_tasks(
        decls: &[Declaration],
    ) -> AstDeclarationValidationTasks<'_> {
        let mut tasks = AstDeclarationValidationTasks::default();
        for decl in decls {
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
            Self::push_behavior_impl_block_declaration_task(
                decl,
                &mut tasks.behavior_associations.impls,
            );
            Self::push_behavior_requires_replay_task(
                decl,
                &mut tasks.behavior_associations.requires,
            );
            Self::push_ast_type_reference_validation_task(decl, &mut tasks.type_references);
            Self::push_ast_struct_field_default_validation_task(
                decl,
                &mut tasks.struct_field_defaults,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_behavior_association_validation_tasks(
        decls: &[Declaration],
    ) -> BehaviorAssociationValidationTasks<'_> {
        let mut tasks = BehaviorAssociationValidationTasks::default();
        for decl in decls {
            Self::push_behavior_extends_replay_task(decl, &mut tasks.extends);
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks.impls);
            Self::push_behavior_requires_replay_task(decl, &mut tasks.requires);
        }
        tasks
    }

    fn collect_resolver_semantic_validation_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        Self::collect_resolver_declaration_metadata_tasks(decls)
    }

    fn validate_behavior_association_tasks<'a>(
        &mut self,
        tasks: &impl BehaviorAssociationValidationTaskSource<'a>,
        symbols: Option<&SymbolTable>,
    ) {
        let tasks = tasks.behavior_association_tasks();
        self.validate_behavior_impl_tasks(tasks, symbols);
        self.validate_behavior_requires_tasks(tasks, symbols);
    }

    fn validate_behavior_impl_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.impls {
            self.validate_collected_behavior_impl_declaration(
                symbols,
                task.ast_type_name,
                task.behavior,
                task.behavior_type_args,
                task.methods,
                task.span,
            );
        }
    }

    fn push_behavior_requires_replay_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorRequiresValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            span,
        } = decl
        {
            tasks.push(BehaviorRequiresValidationTask {
                type_name,
                behavior,
                behavior_type_args,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn validate_behavior_requires_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.requires {
            self.validate_collected_behavior_requires_declaration(
                symbols,
                task.type_name,
                task.behavior,
                task.behavior_type_args,
                task.span,
            );
        }
    }

    fn validate_collected_behavior_impl_declaration(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        let restored_type_name = symbols
            .map(|symbols| {
                self.resolver_impl_type_name_for(
                    symbols,
                    type_name,
                    methods,
                    Some((behavior, behavior_type_args)),
                )
            })
            .unwrap_or_else(|| type_name.to_string());
        self.check_behavior_impl(
            &restored_type_name,
            behavior,
            behavior_type_args,
            methods,
            span,
            symbols,
        );
    }

    fn validate_collected_behavior_requires_declaration(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let type_name = symbols
            .map(|symbols| {
                self.resolver_required_type_name_for(
                    symbols,
                    type_name,
                    behavior,
                    behavior_type_args,
                )
            })
            .unwrap_or_else(|| type_name.to_string());
        self.check_behavior_requires(&type_name, behavior, behavior_type_args, span);
    }

    #[cfg(test)]
    fn validate_struct_field_defaults(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_declaration_metadata_tasks(decls);
            self.validate_resolver_struct_field_default_tasks(&tasks, symbols);
            return;
        }

        let tasks = Self::collect_ast_struct_field_default_validation_tasks(decls);
        self.validate_ast_struct_field_default_tasks(&tasks);
    }

    #[cfg(test)]
    fn collect_ast_struct_field_default_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstStructFieldDefaultValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_struct_field_default_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_struct_field_default_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstStructFieldDefaultValidationTask<'a>>,
    ) {
        if let Declaration::Struct {
            type_params,
            fields,
            ..
        } = decl
        {
            tasks.push(AstStructFieldDefaultValidationTask {
                type_params,
                fields,
            });
        }
    }

    fn validate_ast_struct_field_default_tasks(
        &mut self,
        tasks: &[AstStructFieldDefaultValidationTask<'_>],
    ) {
        for task in tasks {
            self.validate_ast_struct_field_defaults(!task.type_params.is_empty(), task.fields);
        }
    }

    fn validate_resolver_struct_field_defaults(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        let Some(info) = self.structs.get(&restored_name).cloned() else {
            return;
        };
        if !info.type_params.is_empty() {
            return;
        }
        for (field_name, expected) in &info.fields {
            if let Some(default) = info.field_defaults.get(field_name) {
                self.validate_struct_field_default(field_name, expected, default);
            }
        }
    }

    fn validate_resolver_struct_field_default_tasks(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.types {
            if let ResolverTypeDeclarationMetadataTask::Struct { name, span, .. } = task {
                self.validate_resolver_struct_field_defaults(symbols, name, *span);
            }
        }
    }

    fn validate_ast_struct_field_defaults(
        &mut self,
        has_type_params: bool,
        fields: &[StructField],
    ) {
        if has_type_params {
            return;
        }
        for field in fields {
            if let Some(default) = &field.default {
                self.validate_struct_field_default(&field.name, &field.ty, default);
            }
        }
    }

    fn validate_struct_field_default(
        &mut self,
        field_name: &str,
        expected: &AstType,
        default: &Expression,
    ) {
        let expected = self.resolve_type(expected);
        self.push_scope();
        let actual = self.check_expr(default);
        self.pop_scope();

        let Ok(actual) = actual else {
            self.diagnostics.push(actual.expect_err("checked error"));
            return;
        };
        let actual_ty = if (expected.is_integer()
            && matches!(actual.kind, TypedExprKind::IntLiteral(_)))
            || (expected.is_float() && matches!(actual.kind, TypedExprKind::FloatLiteral(_)))
        {
            expected.clone()
        } else {
            actual.ty.clone()
        };

        if !self.types_compatible(&expected, &actual_ty) {
            self.diagnostics.push(Diagnostic::error(
                "E3073",
                format!(
                    "field `{}` default expects `{}`, found `{}`",
                    field_name,
                    expected.display_name(),
                    actual.ty.display_name()
                ),
                actual.span,
            ));
        }
    }

    fn validate_collected_behavior_extends_semantics(&mut self) {
        let behavior_extends: Vec<(String, Vec<BehaviorParentRef>, Span)> = self
            .behavior_extends
            .iter()
            .map(|(behavior, parents)| {
                (
                    behavior.clone(),
                    parents.clone(),
                    self.behavior_extends_spans
                        .get(behavior)
                        .copied()
                        .unwrap_or_else(Span::dummy),
                )
            })
            .collect();

        for (behavior, parents, span) in behavior_extends {
            let scoped_type_params: HashSet<String> = self
                .behaviors
                .get(&behavior)
                .map(|info| info.type_params.iter().cloned().collect())
                .unwrap_or_default();
            for parent in parents {
                self.behavior_type_arg_substitutions(
                    &parent.behavior,
                    &parent.type_args,
                    &scoped_type_params,
                    span,
                );
            }
        }

        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();
    }

    fn collect_resolver_value_signature(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.remove_callable_signature(name);
            return;
        };
        let Some(signature) = Self::resolver_callable_signature_metadata(symbol) else {
            self.remove_callable_signature(name);
            return;
        };
        let info = func_info_from_resolver_signature(
            name.to_string(),
            symbol,
            signature.parameter_names,
            signature.parameter_types,
            signature.return_type,
        );
        self.insert_callable_signature(name, info);
        let type_parameter_names = resolver_type_param_names(symbol);
        self.collect_resolver_generic_template_signature(
            name,
            &type_parameter_names,
            signature.parameter_names,
            signature.parameter_types,
            signature.return_type,
        );
    }

    fn resolver_callable_signature_metadata(
        symbol: &Symbol,
    ) -> Option<ResolverCallableSignature<'_>> {
        Some(ResolverCallableSignature {
            parameter_names: symbol.parameter_names.as_deref()?,
            parameter_types: symbol.parameter_types.as_deref()?,
            return_type: symbol.return_type.as_ref()?,
        })
    }

    fn remove_callable_signature(&mut self, name: &str) {
        self.functions.remove(name);
        self.methods.remove(name);
        self.generic_functions.remove(name);
        self.generic_methods.remove(name);
    }

    fn insert_callable_signature(&mut self, name: &str, info: FuncInfo) {
        self.functions.remove(name);
        self.methods.remove(name);
        if is_method_signature_key(name) {
            self.methods.insert(name.to_string(), info);
        } else {
            self.functions.insert(name.to_string(), info);
        }
    }

    fn generic_callable_template_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut GenericFunctionTemplate> {
        if is_method_signature_key(name) {
            self.generic_methods.get_mut(name)
        } else {
            self.generic_functions.get_mut(name)
        }
    }

    fn collect_resolver_generic_template_signature(
        &mut self,
        name: &str,
        type_parameter_names: &[String],
        parameter_names: &[String],
        parameter_types: &[AstType],
        return_type: &AstType,
    ) {
        let Some(template) = self.generic_callable_template_mut(name) else {
            return;
        };
        template.type_params = type_parameter_names.to_vec();
        let existing_params = template.params.clone();
        template.params = Self::resolver_params_from_metadata(
            &existing_params,
            parameter_names,
            parameter_types,
            template.span,
        );
        template.return_type = Self::resolver_optional_return_type(return_type);
    }

    fn collect_resolver_method_signature(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        method_name: &str,
        span: Span,
    ) {
        let ast_key = Self::method_key(type_name, method_name);
        let restored_key =
            Self::resolver_method_signature_name_for(symbols, &ast_key, type_name, span);

        self.collect_resolver_callable_signature_for_key(symbols, &ast_key, &restored_key);
    }

    fn collect_resolver_function_signature(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::resolver_symbol_name_for(symbols, Namespace::Value, name, span);

        self.collect_resolver_callable_signature_for_key(symbols, name, &restored_name);
    }

    fn collect_resolver_callable_signature_for_key(
        &mut self,
        symbols: &SymbolTable,
        ast_key: &str,
        restored_key: &str,
    ) {
        if restored_key != ast_key {
            self.rekey_callable_template(ast_key, restored_key);
            self.remove_callable_signature(ast_key);
        }
        self.collect_resolver_value_signature(symbols, restored_key);
    }

    fn rekey_callable_template(&mut self, old_key: &str, new_key: &str) {
        let template = self
            .generic_functions
            .remove(old_key)
            .or_else(|| self.generic_methods.remove(old_key));

        if let Some(template) = template {
            if is_method_signature_key(new_key) {
                self.generic_methods.insert(new_key.to_string(), template);
            } else {
                self.generic_functions.insert(new_key.to_string(), template);
            }
        }
    }

    fn resolver_symbol_name_for(
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(namespace, name)
            .or_else(|| Self::resolver_symbol_by_span(symbols, namespace, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| name.to_string())
    }

    fn resolver_method_signature_name_for(
        symbols: &SymbolTable,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(Namespace::Value, ast_key)
            .or_else(|| {
                let prefix = format!("{type_name}.");
                Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
                    symbol.name.starts_with(&prefix)
                })
            })
            .or_else(|| Self::resolver_method_signature_symbol_by_span(symbols, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| ast_key.to_string())
    }

    fn resolver_symbol_by_span(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
    ) -> Option<&crate::resolver::Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, namespace, span, |_| true)
    }

    fn resolver_method_signature_symbol_by_span(
        symbols: &SymbolTable,
        span: Span,
    ) -> Option<&crate::resolver::Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
            is_method_signature_key(&symbol.name)
        })
    }

    fn resolver_symbol_by_span_matching(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
        matches: impl Fn(&crate::resolver::Symbol) -> bool,
    ) -> Option<&crate::resolver::Symbol> {
        symbols.symbols().iter().find(|symbol| {
            symbol.namespace == namespace && symbol.definition_span == span && matches(symbol)
        })
    }

    fn resolver_impl_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
        behavior_ref: Option<(&str, &[AstType])>,
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(type_name) = methods.iter().find_map(|method| {
            let Declaration::Function { span, .. } = method else {
                return None;
            };
            Self::resolver_method_signature_symbol_by_span(symbols, *span)
                .and_then(|symbol| method_signature_receiver_name(&symbol.name).map(str::to_string))
        }) {
            return type_name;
        }

        if let Some((behavior, behavior_type_args)) = behavior_ref {
            if let Some(candidate) = self.resolver_behavior_ref_owner_for(
                &self.resolver_behavior_impl_refs,
                &self.resolver_missing_behavior_impl_refs,
                behavior,
                behavior_type_args,
            ) {
                return candidate;
            }
        }

        type_name.to_string()
    }

    fn resolver_required_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(candidate) = self.resolver_behavior_ref_owner_for(
            &self.resolver_behavior_required_refs,
            &self.resolver_missing_behavior_required_refs,
            behavior,
            behavior_type_args,
        ) {
            return candidate;
        }

        type_name.to_string()
    }

    fn resolver_behavior_ref_owner_for(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &HashSet<String>,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<String> {
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);
        self.unique_behavior_ref_owner_for_key(refs_by_type, &behavior_key)
            .or_else(|| self.unique_behavior_ref_owner(refs_by_type, |_| true))
            .or_else(|| Self::unique_owned_candidate(missing_refs.iter().cloned()))
    }

    fn unique_behavior_ref_owner_for_key(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        behavior_key: &str,
    ) -> Option<String> {
        self.unique_behavior_ref_owner(refs_by_type, |reference| {
            self.behavior_reference_matches_key(reference, behavior_key)
        })
    }

    fn behavior_reference_matches_key(
        &self,
        reference: &BehaviorRefMetadata,
        behavior_key: &str,
    ) -> bool {
        self.behavior_reference_key(&reference.name, &reference.type_args) == behavior_key
    }

    fn unique_behavior_ref_owner(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        matches: impl Fn(&BehaviorRefMetadata) -> bool,
    ) -> Option<String> {
        Self::unique_owned_candidate(refs_by_type.iter().filter_map(|(candidate_type, refs)| {
            refs.iter().any(&matches).then_some(candidate_type.clone())
        }))
    }

    fn unique_owned_candidate(mut candidates: impl Iterator<Item = String>) -> Option<String> {
        let candidate = candidates.next()?;
        candidates.next().is_none().then_some(candidate)
    }

    fn collect_resolver_struct_fields(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        ast_fields: &[StructField],
    ) {
        let Some((symbol, field_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                Self::resolver_struct_field_metadata(symbol)
            })
        else {
            self.structs.remove(name);
            return;
        };

        let (fields, field_defaults) =
            Self::resolver_struct_fields_from_metadata(field_types, ast_fields);
        self.structs.insert(
            name.to_string(),
            struct_info_from_resolver_fields(name.to_string(), symbol, fields, field_defaults),
        );
    }

    fn resolver_struct_field_metadata(symbol: &Symbol) -> Option<&[(String, AstType)]> {
        symbol.field_types.as_deref()
    }

    fn resolver_struct_fields_from_metadata(
        fields: &[(String, AstType)],
        ast_fields: &[StructField],
    ) -> (Vec<(String, AstType)>, HashMap<String, Expression>) {
        let field_defaults = ast_fields
            .iter()
            .zip(fields.iter())
            .filter_map(|(field, (restored_name, _))| {
                field
                    .default
                    .as_ref()
                    .map(|default| (restored_name.clone(), default.clone()))
            })
            .collect();
        (fields.to_vec(), field_defaults)
    }

    fn collect_resolver_enum_variants(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, variant_names)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                Self::resolver_enum_variant_name_metadata(symbol)
            })
        else {
            self.enums.remove(name);
            return;
        };

        let variants = Self::resolver_enum_variants_from_metadata(symbols, name, variant_names);
        self.enums.insert(
            name.to_string(),
            enum_info_from_resolver_variants(name.to_string(), symbol, variants),
        );
    }

    fn resolver_enum_variant_name_metadata(symbol: &Symbol) -> Option<&[String]> {
        symbol.variant_names.as_deref()
    }

    fn resolver_enum_variants_from_metadata(
        symbols: &SymbolTable,
        enum_name: &str,
        variant_names: &[String],
    ) -> Vec<(String, Option<AstType>)> {
        variant_names
            .iter()
            .map(|variant_name| {
                (
                    variant_name.clone(),
                    symbols
                        .lookup_variant(enum_name, variant_name)
                        .and_then(|variant| variant.variant_payload_type.clone()),
                )
            })
            .collect()
    }

    fn collect_resolver_behavior_methods(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, method_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Behavior, name, |symbol| {
                Self::resolver_behavior_method_metadata(symbol)
            })
        else {
            self.behaviors.remove(name);
            return;
        };

        let Some(existing) = self.behaviors.get(name).cloned() else {
            return;
        };
        let methods = Self::resolver_behavior_methods_from_metadata(
            existing.methods,
            method_types,
            symbol.definition_span,
        );
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_resolver_methods(name.to_string(), symbol, methods),
        );
    }

    fn resolver_behavior_method_metadata(symbol: &Symbol) -> Option<&[BehaviorMethodTypeMetadata]> {
        symbol.behavior_method_types.as_deref()
    }

    fn resolver_behavior_methods_from_metadata(
        existing_methods: Vec<ast::BehaviorMethod>,
        method_types: &[BehaviorMethodTypeMetadata],
        span: Span,
    ) -> Vec<ast::BehaviorMethod> {
        let mut existing_methods: VecDeque<ast::BehaviorMethod> =
            existing_methods.into_iter().collect();
        let mut methods = Vec::new();
        for (metadata_index, metadata) in method_types.iter().cloned().enumerate() {
            let future_method_names = method_types[metadata_index + 1..]
                .iter()
                .map(|metadata| metadata.name.as_str());
            let method = Self::named_queue_index_preserving_future_front(
                &existing_methods,
                &metadata.name,
                future_method_names,
                |method| method.name.as_str(),
            )
            .and_then(|index| existing_methods.remove(index));
            methods.push(Self::resolver_behavior_method_from_metadata(
                method.as_ref(),
                metadata,
                span,
            ));
        }
        methods
    }

    fn resolver_behavior_method_from_metadata(
        existing_method: Option<&ast::BehaviorMethod>,
        metadata: BehaviorMethodTypeMetadata,
        span: Span,
    ) -> ast::BehaviorMethod {
        let params = Self::resolver_params_from_metadata(
            existing_method
                .map(|method| method.params.as_slice())
                .unwrap_or(&[]),
            &metadata.parameter_names,
            &metadata.parameter_types,
            Span::dummy(),
        );
        let return_type = Self::resolver_optional_return_type(&metadata.return_type);
        ast::BehaviorMethod {
            name: metadata.name,
            params,
            return_type,
            default_body: existing_method.and_then(|method| method.default_body.clone()),
            span: existing_method.map(|method| method.span).unwrap_or(span),
        }
    }

    fn resolver_params_from_metadata(
        existing_params: &[Param],
        parameter_names: &[String],
        parameter_types: &[AstType],
        default_span: Span,
    ) -> Vec<Param> {
        parameter_types
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, ty)| match existing_params.get(index).cloned() {
                Some(mut param) => {
                    if let Some(name) = parameter_names.get(index) {
                        param.name = name.clone();
                    }
                    param.ty = ty;
                    param
                }
                None => Param {
                    name: parameter_names.get(index).cloned().unwrap_or_default(),
                    ty,
                    mutable: false,
                    span: default_span,
                },
            })
            .collect()
    }

    fn resolver_optional_return_type(return_type: &AstType) -> Option<AstType> {
        match return_type {
            AstType::Void => None,
            ty => Some(ty.clone()),
        }
    }

    fn collect_resolver_behavior_parents(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((parent_refs, definition_span)) =
            Self::resolver_behavior_refs(symbols, Namespace::Behavior, name, |symbol| {
                &symbol.behavior_parent_refs
            })
            .map(|(refs, symbol)| (refs, symbol.definition_span))
        else {
            return;
        };

        let parents = self.behavior_parent_refs_from_metadata(parent_refs);
        self.behavior_extends.insert(name.to_string(), parents);
        self.behavior_extends_spans
            .entry(name.to_string())
            .or_insert(definition_span);
    }

    fn collect_resolver_type_behavior_impls(&mut self, symbols: &SymbolTable, name: &str) {
        self.behavior_impls
            .retain(|(type_name, _)| type_name != name);
        let Some((impl_refs, _)) =
            Self::resolver_behavior_refs(symbols, Namespace::Type, name, |symbol| {
                &symbol.behavior_impl_refs
            })
        else {
            return;
        };

        for behavior in impl_refs {
            let behavior_ref = self.behavior_parent_ref(&behavior.name, &behavior.type_args);
            let implementation = (name.to_string(), behavior_ref.key.clone());
            self.behavior_impls.insert(implementation);
            self.behavior_refs_by_key
                .insert(behavior_ref.key.clone(), behavior_ref);
        }
    }

    fn collect_resolver_type_behavior_impl_refs(&mut self, symbols: &SymbolTable, name: &str) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_impl_refs,
            &mut self.resolver_behavior_impl_refs,
            &mut self.resolver_missing_behavior_impl_refs,
        );
    }

    fn collect_resolver_type_behavior_requires(&mut self, symbols: &SymbolTable, name: &str) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_required_refs,
            &mut self.resolver_behavior_required_refs,
            &mut self.resolver_missing_behavior_required_refs,
        );
    }

    fn collect_resolver_type_behavior_refs(
        symbols: &SymbolTable,
        name: &str,
        select_refs: impl Fn(&crate::resolver::Symbol) -> &Option<Vec<BehaviorRefMetadata>>,
        collected_refs: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &mut HashSet<String>,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };

        if let Some(refs) = select_refs(symbol).as_deref() {
            collected_refs.insert(name.to_string(), refs.iter().cloned().collect());
        } else {
            missing_refs.insert(name.to_string());
        }
    }

    fn resolver_behavior_refs<'a>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_refs: impl Fn(&'a crate::resolver::Symbol) -> &'a Option<Vec<BehaviorRefMetadata>>,
    ) -> Option<(&'a [BehaviorRefMetadata], &'a crate::resolver::Symbol)> {
        let (symbol, refs) = Self::resolver_symbol_metadata(symbols, namespace, name, |symbol| {
            select_refs(symbol).as_deref()
        })?;

        Some((refs, symbol))
    }

    fn resolver_symbol_metadata<'a, T: ?Sized>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_metadata: impl Fn(&'a crate::resolver::Symbol) -> Option<&'a T>,
    ) -> Option<(&'a crate::resolver::Symbol, &'a T)> {
        let symbol = symbols.lookup(namespace, name)?;
        let metadata = select_metadata(symbol)?;
        Some((symbol, metadata))
    }

    fn behavior_parent_ref_from_metadata(
        &self,
        metadata: &BehaviorRefMetadata,
    ) -> BehaviorParentRef {
        self.behavior_parent_ref(&metadata.name, &metadata.type_args)
    }

    fn behavior_parent_refs_from_metadata(
        &self,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<BehaviorParentRef> {
        metadata
            .iter()
            .map(|parent| self.behavior_parent_ref_from_metadata(parent))
            .collect()
    }

    fn behavior_parent_ref(&self, behavior: &str, type_args: &[AstType]) -> BehaviorParentRef {
        BehaviorParentRef {
            behavior: behavior.to_string(),
            type_args: type_args.to_vec(),
            key: self.behavior_reference_key(behavior, type_args),
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
            let restored_key = Self::method_key(type_name, &restored_name);
            self.collect_resolver_callable_signature_for_key(symbols, &ast_key, &restored_key);
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

    fn behavior_reference_key(&self, behavior: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            behavior.to_string()
        } else {
            self.mangle_generic_type_name(behavior, type_args)
        }
    }

    fn insert_behavior_impl_ref(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) {
        let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
        let behavior_key = behavior_ref.key.clone();
        self.behavior_impls
            .insert((type_name.to_string(), behavior_key));
        self.behavior_refs_by_key
            .insert(behavior_ref.key.clone(), behavior_ref);
    }

    #[cfg(test)]
    fn behavior_impl_refs_from_metadata(
        &self,
        type_name: &str,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<(String, String)> {
        metadata
            .iter()
            .map(|behavior| {
                let behavior_ref = self.behavior_parent_ref(&behavior.name, &behavior.type_args);
                (type_name.to_string(), behavior_ref.key)
            })
            .collect()
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
}

#[cfg(test)]
mod tests;
