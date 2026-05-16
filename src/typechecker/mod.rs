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
mod environment;
mod expressions;
mod monomorphize;
mod patterns;
mod resolve;
mod resolver_validation;
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

    /// Type-check a program and produce a TypedProgram.
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);
        self.validate_collected_declaration_semantics(&program.declarations, None);
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
                    let full_name = Self::method_key(type_name, method_name);
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
                            let full_name = Self::method_key(type_name, name);
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
                            let full_name = Self::method_key(type_name, &default.name);
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

        self.collect_declarations_with_symbols(&module.program.declarations, &module.symbols);
        self.check_program_after_collection(&module.program)
    }

    /// Get all diagnostics (errors + warnings).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
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

    fn check_behavior_requires(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let resolver_required_ref = self.resolver_required_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_required_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_required_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_required_ref.as_ref(), behavior, behavior_type_args);

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

    fn resolver_required_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        self.resolver_behavior_ref_for(BehaviorRefRole::Required, type_name, behavior)
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

        let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
        let parent_display = behavior_ref_display(parent, parent_type_args);
        let parents = self
            .behavior_extends
            .entry(behavior.to_string())
            .or_default();
        if parents
            .iter()
            .any(|existing| existing.key == parent_ref.key)
        {
            self.diagnostics.push(Diagnostic::error(
                "E6011",
                format!("duplicate behavior inheritance `{behavior}.extends({parent_display})`"),
                span,
            ));
            return;
        }

        parents.push(parent_ref);
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
        if !self.mark_behavior_seen(behavior, substitutions, seen_behaviors) {
            return;
        }

        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
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
                let method = substituted_behavior_method_signature(method, substitutions);

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

        if self.behavior_extends_parent_matches(behavior, &HashMap::new(), parent, seen) {
            return true;
        }

        self.behavior_refs_by_key
            .get(behavior)
            .is_some_and(|behavior_ref| {
                let substitutions = self.behavior_type_param_substitutions(
                    &behavior_ref.behavior,
                    &behavior_ref.type_args,
                );
                self.behavior_extends_parent_matches(
                    &behavior_ref.behavior,
                    &substitutions,
                    parent,
                    seen,
                )
            })
    }

    fn behavior_extends_parent_matches(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents.iter().any(|candidate| {
                let candidate_args: Vec<AstType> = candidate
                    .type_args
                    .iter()
                    .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
                    .collect();
                let candidate_ref = self.behavior_parent_ref(&candidate.behavior, &candidate_args);
                candidate_ref.key == parent
                    || self.behavior_ref_inherits_from_inner(&candidate_ref, parent, seen)
            })
        })
    }

    fn behavior_ref_inherits_from_inner(
        &self,
        behavior_ref: &BehaviorParentRef,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(behavior_ref.key.clone()) {
            return false;
        }

        let substitutions =
            self.behavior_type_param_substitutions(&behavior_ref.behavior, &behavior_ref.type_args);
        self.behavior_extends_parent_matches(&behavior_ref.behavior, &substitutions, parent, seen)
    }

    fn check_behavior_impl(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
        symbols: Option<&SymbolTable>,
    ) {
        let resolver_impl_ref = self.resolver_impl_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_impl_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_impl_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_impl_ref.as_ref(), behavior, behavior_type_args);

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
        self.behavior_refs_by_key.insert(
            behavior_key.clone(),
            self.behavior_parent_ref(behavior, behavior_type_args),
        );
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());
        let mut unmatched_required: VecDeque<String> = required_methods
            .iter()
            .map(|required| required.name.clone())
            .collect();
        let effective_methods = self.effective_behavior_impl_methods(
            symbols,
            type_name,
            methods,
            &mut unmatched_required,
        );

        for method in &effective_methods {
            if let Declaration::Function { span, .. } = method.declaration {
                if !required_methods
                    .iter()
                    .any(|required| required.name == method.method_name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            method.method_name, behavior_key
                        ),
                        *span,
                    ));
                }
            }
        }

        for required in &required_methods {
            let Some(actual) =
                effective_methods
                    .iter()
                    .find_map(|method| match method.declaration {
                        Declaration::Function {
                            params,
                            return_type,
                            span,
                            ..
                        } if method.method_name == required.name => {
                            Some((params, return_type, *span))
                        }
                        _ => None,
                    })
            else {
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
            let collected_signature =
                self.resolver_backed_method_signature(type_name, &required.name);
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

    fn resolver_impl_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        self.resolver_behavior_ref_for(BehaviorRefRole::Impl, type_name, behavior)
    }

    fn resolver_behavior_ref_for(
        &mut self,
        role: BehaviorRefRole,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        match role {
            BehaviorRefRole::Impl => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_impl_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Required => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_required_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Parent => None,
        }
    }

    fn behavior_ref_parts<'a>(
        resolver_ref: Option<&'a BehaviorRefMetadata>,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        resolver_ref
            .map(|reference| (reference.name.as_str(), reference.type_args.as_slice()))
            .unwrap_or((behavior, behavior_type_args))
    }

    fn pop_resolver_behavior_ref(
        resolver_backed_collection: bool,
        refs_by_type: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get_mut(type_name)?;
        Self::pop_resolver_behavior_ref_from_queue(refs, behavior)
    }

    fn should_skip_missing_resolver_behavior_ref(
        &self,
        resolver_ref: Option<&BehaviorRefMetadata>,
        type_name: &str,
        missing_refs: &HashSet<String>,
    ) -> bool {
        self.resolver_backed_collection
            && resolver_ref.is_none()
            && missing_refs.contains(type_name)
    }

    fn pop_resolver_behavior_ref_from_queue(
        refs: &mut VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        let index = Self::resolver_behavior_ref_queue_index(refs, behavior)?;
        refs.remove(index)
    }

    fn resolver_behavior_impl_ref_for_peek(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> Option<&BehaviorRefMetadata> {
        Self::peek_resolver_behavior_ref(
            self.resolver_backed_collection,
            &self.resolver_behavior_impl_refs,
            type_name,
            behavior,
        )
    }

    fn peek_resolver_behavior_ref<'a>(
        resolver_backed_collection: bool,
        refs_by_type: &'a HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<&'a BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get(type_name)?;
        Self::resolver_behavior_ref_queue_index(refs, behavior).and_then(|index| refs.get(index))
    }

    fn resolver_behavior_ref_queue_index(
        refs: &VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<usize> {
        Self::named_queue_index(refs, behavior, |reference| reference.name.as_str())
    }

    fn named_queue_index<T>(
        items: &VecDeque<T>,
        name: &str,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        items
            .iter()
            .position(|item| item_name(item) == name)
            .or_else(|| (!items.is_empty()).then_some(0))
    }

    fn named_queue_index_preserving_future_front<'a, T>(
        items: &VecDeque<T>,
        name: &str,
        future_names: impl IntoIterator<Item = &'a str>,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if let Some(index) = items.iter().position(|item| item_name(item) == name) {
            return Some(index);
        }

        let front_name = item_name(items.front()?);
        (!future_names
            .into_iter()
            .any(|future_name| future_name == front_name))
        .then_some(0)
    }

    fn resolver_behavior_impl_ref_parts<'a>(
        &'a self,
        type_name: &str,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        match self.resolver_behavior_impl_ref_for_peek(type_name, behavior) {
            Some(implementation) => (
                implementation.name.as_str(),
                implementation.type_args.as_slice(),
            ),
            None => (behavior, behavior_type_args),
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
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .iter()
            .filter(|required| {
                required.default_body.is_some()
                    && !self.impl_methods_include_behavior_method(
                        type_name,
                        methods,
                        &required.name,
                    )
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

    fn seed_behavior_default_method_signature(
        &mut self,
        type_name: &str,
        default: &DefaultBehaviorMethod,
    ) {
        let key = Self::method_key(type_name, &default.name);
        self.methods.insert(
            key.clone(),
            func_info_from_behavior_method(key, &default.params, &default.return_type),
        );
    }

    fn impl_methods_include_behavior_method(
        &self,
        type_name: &str,
        methods: &[Declaration],
        required_name: &str,
    ) -> bool {
        methods
            .iter()
            .any(|decl| matches!(decl, Declaration::Function { name, .. } if name == required_name))
            || (self.resolver_backed_collection
                && self
                    .resolver_backed_method_signature(type_name, required_name)
                    .is_some())
    }

    fn impl_effective_method_name(
        &self,
        unmatched_required: &mut VecDeque<String>,
        ast_name: &str,
        resolver_owned_key: Option<String>,
        type_name: &str,
    ) -> String {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(&resolver_owned_key, type_name)
                    .unwrap_or(&resolver_owned_key)
                    .to_string();
            return Self::remove_named_queue_entry(unmatched_required, &resolver_owned_name)
                .unwrap_or(resolver_owned_name);
        }

        if let Some(name) = Self::remove_named_queue_entry(unmatched_required, ast_name) {
            return name;
        }

        if self.resolver_backed_collection {
            if let Some(index) = unmatched_required.iter().position(|required| {
                self.resolver_backed_method_signature(type_name, required)
                    .is_some()
            }) {
                return unmatched_required
                    .remove(index)
                    .unwrap_or_else(|| ast_name.to_string());
            }
        }

        ast_name.to_string()
    }

    fn effective_behavior_impl_methods<'a>(
        &self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &'a [Declaration],
        unmatched_required: &mut VecDeque<String>,
    ) -> Vec<EffectiveBehaviorImplMethod<'a>> {
        methods
            .iter()
            .map(|method| {
                let ast_name = match method {
                    Declaration::Function { name, .. } => name.as_str(),
                    _ => "",
                };
                let ast_key = Self::method_key(type_name, ast_name);
                let resolver_owned_name = self.resolver_backed_impl_method_key(
                    symbols,
                    &ast_key,
                    type_name,
                    method.span(),
                );
                let method_name = self.impl_effective_method_name(
                    unmatched_required,
                    ast_name,
                    resolver_owned_name,
                    type_name,
                );
                EffectiveBehaviorImplMethod {
                    declaration: method,
                    method_name,
                }
            })
            .collect()
    }

    fn resolver_backed_behavior_impl_method_signature_name(
        &self,
        required_methods: &mut VecDeque<ast::BehaviorMethod>,
        ast_name: &str,
        resolver_owned_key: Option<&str>,
        type_name: &str,
    ) -> Option<String> {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(resolver_owned_key, type_name)
                    .unwrap_or(resolver_owned_key);
            if let Some(index) =
                Self::named_queue_index(required_methods, resolver_owned_name, |required| {
                    required.name.as_str()
                })
            {
                return required_methods.remove(index).map(|required| required.name);
            }
        }

        Self::named_queue_index(required_methods, ast_name, |required| {
            required.name.as_str()
        })
        .and_then(|index| required_methods.remove(index).map(|required| required.name))
    }

    fn resolver_backed_impl_method_key(
        &self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> Option<String> {
        self.resolver_backed_collection
            .then(|| Self::validation_method_key(symbols, ast_key, type_name, span))
    }

    fn resolver_backed_method_signature(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&FuncInfo> {
        self.resolver_backed_collection
            .then(|| self.methods.get(&Self::method_key(type_name, method_name)))
            .flatten()
    }

    fn method_key(type_name: &str, method_name: &str) -> String {
        method_signature_key(type_name, method_name)
    }

    fn remove_named_queue_entry(items: &mut VecDeque<String>, name: &str) -> Option<String> {
        items
            .iter()
            .position(|item| item == name)
            .and_then(|index| items.remove(index))
    }

    fn behavior_methods_with_inherited_substituted(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        if !self.mark_behavior_seen(behavior, substitutions, seen) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
                methods.extend(self.behavior_methods_with_inherited_substituted(
                    &parent.behavior,
                    &parent_substitutions,
                    seen,
                ));
            }
        }
        if let Some(info) = self.behaviors.get(behavior) {
            methods.extend(
                info.methods
                    .iter()
                    .map(|method| substituted_behavior_method_signature(method, substitutions)),
            );
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

    fn mark_behavior_seen(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> bool {
        let behavior_seen_key = self.behavior_seen_key(behavior, substitutions);
        seen.insert(behavior_seen_key)
    }

    fn behavior_methods_for_impl(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        self.behavior_methods_with_inherited_substituted(behavior, substitutions, seen)
    }

    fn behavior_type_param_substitutions(
        &self,
        behavior: &str,
        type_args: &[AstType],
    ) -> HashMap<String, AstType> {
        self.behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .cloned()
                    .zip(type_args.iter().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn behavior_parent_type_param_substitutions(
        &self,
        parent: &BehaviorParentRef,
        substitutions: &HashMap<String, AstType>,
    ) -> HashMap<String, AstType> {
        let parent_type_args: Vec<AstType> = parent
            .type_args
            .iter()
            .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
            .collect();
        self.behavior_type_param_substitutions(&parent.behavior, &parent_type_args)
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

    #[cfg(test)]
    fn collect_ast_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Struct {
                type_params,
                fields,
            }),
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Enum {
                type_params,
                variants,
            }),
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Function {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Method {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Behavior {
                type_params,
                methods,
            }),
            Declaration::ImplBlock { methods, .. } => {
                tasks.push(AstTypeReferenceValidationTask::ImplBlock { methods });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(AstTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    fn validate_ast_type_reference_tasks(&mut self, tasks: &[AstTypeReferenceValidationTask<'_>]) {
        for task in tasks {
            match task {
                AstTypeReferenceValidationTask::Struct {
                    type_params,
                    fields,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in *fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                        if let Some(default) = &field.default {
                            self.validate_generic_expr_type_references(default, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Enum {
                    type_params,
                    variants,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Behavior {
                    type_params,
                    methods,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in *methods {
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
                AstTypeReferenceValidationTask::ImplBlock { methods } => {
                    for method in *methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            self.validate_ast_callable_type_references(
                                type_params,
                                params,
                                return_type,
                                body,
                                method.span(),
                            );
                        }
                    }
                }
                AstTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_ast_callable_type_references(
        &mut self,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        return_span: Span,
    ) {
        let scoped = type_param_name_set(type_params);
        for param in params {
            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
        }
        if let Some(return_type) = return_type {
            self.validate_generic_type_ref_bounds(return_type, &scoped, return_span);
        }
        self.validate_generic_expr_type_references(body, &scoped);
    }

    fn validate_resolver_type_reference_tasks(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.type_references {
            match task {
                ResolverTypeReferenceValidationTask::Struct { name, fields, span } => {
                    self.validate_resolver_struct_type_references(symbols, name, fields, *span);
                }
                ResolverTypeReferenceValidationTask::Enum { name, span } => {
                    self.validate_resolver_enum_type_references(symbols, name, *span);
                }
                ResolverTypeReferenceValidationTask::Function { name, body, span } => {
                    self.validate_resolver_function_type_references(symbols, name, body, *span);
                }
                ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span,
                } => {
                    let ast_key = Self::method_key(type_name, method_name);
                    self.validate_resolver_method_type_references(
                        symbols, &ast_key, type_name, body, *span,
                    );
                }
                ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span,
                } => {
                    self.validate_resolver_behavior_type_references(symbols, name, methods, *span);
                }
                ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods } => {
                    self.validate_resolver_impl_method_type_references(symbols, type_name, methods);
                }
                ResolverTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_resolver_enum_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_enum_type_references(&restored_name, &scoped, span);
        }
    }

    fn validate_resolver_struct_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_struct_type_references(&restored_name, &scoped, span);
            for field in fields {
                if let Some(default) = &field.default {
                    self.validate_generic_expr_type_references(default, &scoped);
                }
            }
        }
    }

    fn validate_resolver_behavior_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        methods: &[BehaviorMethod],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Behavior, name, span);
        if let Some(scoped) = self.collected_behavior_type_param_scope(&restored_name) {
            self.validate_collected_behavior_type_references(&restored_name, &scoped, span);
            for method in methods {
                if let Some(default_body) = &method.default_body {
                    self.validate_generic_expr_type_references(default_body, &scoped);
                }
            }
        }
    }

    fn validate_resolver_impl_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, body, .. } = method {
                let ast_key = Self::method_key(type_name, name);
                self.validate_resolver_method_type_references(
                    symbols,
                    &ast_key,
                    type_name,
                    body,
                    method.span(),
                );
            }
        }
    }

    fn validate_resolver_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_key = Self::validation_method_key(symbols, ast_key, type_name, span);
        self.validate_resolver_callable_type_references(&restored_key, body, span);
    }

    fn validate_resolver_function_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Value, name, span);
        self.validate_resolver_callable_type_references(&restored_name, body, span);
    }

    fn validate_resolver_callable_type_references(
        &mut self,
        restored_key: &str,
        body: &Expression,
        span: Span,
    ) {
        if let Some(scoped) = self.collected_value_type_param_scope(restored_key) {
            self.validate_collected_value_type_references(restored_key, &scoped, span);
            self.validate_generic_expr_type_references(body, &scoped);
        }
    }

    fn validation_symbol_name(
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| Self::resolver_symbol_name_for(symbols, namespace, name, span))
            .unwrap_or_else(|| name.to_string())
    }

    fn validation_method_key(
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| {
                Self::resolver_method_signature_name_for(symbols, ast_key, type_name, span)
            })
            .unwrap_or_else(|| ast_key.to_string())
    }

    fn collected_value_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn collected_type_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.structs
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
            .or_else(|| {
                self.enums
                    .get(name)
                    .map(|info| info.type_params.iter().cloned().collect())
            })
    }

    fn collected_behavior_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.behaviors
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn validate_collected_struct_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.structs.get(name).cloned() else {
            return;
        };
        for (_, ty) in &info.fields {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
    }

    fn validate_collected_enum_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.enums.get(name).cloned() else {
            return;
        };
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.validate_generic_type_ref_bounds(payload, scoped, span);
            }
        }
    }

    fn validate_collected_behavior_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.behaviors.get(name).cloned() else {
            return;
        };
        for method in &info.methods {
            for param in &method.params {
                self.validate_generic_type_ref_bounds(&param.ty, scoped, span);
            }
            if let Some(return_type) = &method.return_type {
                self.validate_generic_type_ref_bounds(return_type, scoped, span);
            }
        }
    }

    fn validate_collected_value_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let info = self
            .functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .cloned();
        let Some(info) = info else {
            return;
        };

        for (_, ty) in &info.params {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
        self.validate_generic_type_ref_bounds(&info.return_type, scoped, span);
    }

    #[cfg(test)]
    fn collect_self_type_context_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<SelfTypeContextValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_self_type_context_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<SelfTypeContextValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct { fields, .. } => {
                tasks.push(SelfTypeContextValidationTask::Struct { fields });
            }
            Declaration::Enum { variants, .. } => {
                tasks.push(SelfTypeContextValidationTask::Enum { variants });
            }
            Declaration::Function {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Function {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Method {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Behavior { methods, .. } => {
                tasks.push(SelfTypeContextValidationTask::Behavior { methods });
            }
            Declaration::ImplBlock {
                behavior_type_args,
                methods,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::ImplBlock {
                behavior_type_args,
                methods,
                span: *span,
            }),
            Declaration::Requires {
                behavior_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Requires {
                behavior_type_args,
                span: *span,
            }),
            Declaration::BehaviorExtends {
                parent_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::BehaviorExtends {
                parent_type_args,
                span: *span,
            }),
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(SelfTypeContextValidationTask::TopLevelExpr { expr });
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
        }
    }

    fn validate_self_type_context_tasks(&mut self, tasks: &[SelfTypeContextValidationTask<'_>]) {
        for task in tasks {
            match task {
                SelfTypeContextValidationTask::Struct { fields } => {
                    for field in *fields {
                        self.validate_self_type_ref(&field.ty, field.span, false);
                        if let Some(default) = &field.default {
                            self.validate_self_type_expr(default, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Enum { variants } => {
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_self_type_ref(payload, variant.span, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Function {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, false);
                }
                SelfTypeContextValidationTask::Method {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, true);
                }
                SelfTypeContextValidationTask::Behavior { methods } => {
                    for method in *methods {
                        let Some(default_body) = &method.default_body else {
                            self.validate_self_type_params(&method.params, true);
                            if let Some(return_type) = &method.return_type {
                                self.validate_self_type_ref(return_type, method.span, true);
                            }
                            continue;
                        };
                        self.validate_self_type_callable(
                            &method.params,
                            &method.return_type,
                            default_body,
                            method.span,
                            true,
                        );
                    }
                }
                SelfTypeContextValidationTask::ImplBlock {
                    behavior_type_args,
                    methods,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                    for method in *methods {
                        if let Declaration::Function {
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            self.validate_self_type_callable(
                                params,
                                return_type,
                                body,
                                *span,
                                true,
                            );
                        }
                    }
                }
                SelfTypeContextValidationTask::Requires {
                    behavior_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                }
                SelfTypeContextValidationTask::BehaviorExtends {
                    parent_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(parent_type_args, *span, false);
                }
                SelfTypeContextValidationTask::TopLevelExpr { expr } => {
                    self.validate_self_type_expr(expr, false);
                }
            }
        }
    }

    fn validate_self_type_callable(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
        allow_self_type: bool,
    ) {
        self.validate_self_type_params(params, allow_self_type);
        if let Some(return_type) = return_type {
            self.validate_self_type_ref(return_type, span, allow_self_type);
        }
        self.validate_self_type_expr(body, allow_self_type);
    }

    fn validate_self_type_params(&mut self, params: &[Param], allow_self_type: bool) {
        for param in params {
            self.validate_self_type_ref(&param.ty, param.span, allow_self_type);
        }
    }

    fn validate_self_type_refs(
        &mut self,
        ast_types: &[AstType],
        span: Span,
        allow_self_type: bool,
    ) {
        for ast_type in ast_types {
            self.validate_self_type_ref(ast_type, span, allow_self_type);
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
            | Expression::LoopControl { .. }
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

    fn validate_generic_type_arg_refs_allow_unknowns(&mut self, type_args: &[AstType], span: Span) {
        let scoped_type_params = HashSet::new();
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            &scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_arg_refs(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_arg_refs_with_unknowns(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        for type_arg in type_args {
            self.validate_generic_type_ref_bounds_with_unknowns(
                type_arg,
                scoped_type_params,
                span,
                reject_unknown,
            );
        }
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
                self.validate_generic_type_arg_refs_with_unknowns(
                    type_args,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );

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
                self.validate_generic_type_arg_refs_with_unknowns(
                    params,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
            | Expression::LoopControl { .. }
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
}

#[cfg(test)]
mod tests;
