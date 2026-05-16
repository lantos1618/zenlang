//! Resolver-symbol validation against resolver metadata.
#![allow(clippy::result_large_err)]

use super::*;

impl TypeChecker {
    pub(super) fn validate_resolver_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
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
                        expected_value_symbol(params, return_type, type_params, *public),
                        *span,
                    );
                    self.require_resolver_callable_locals(symbols, params, body, &mut scope_cursor);
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
                        &Self::method_key(type_name, method_name),
                        expected_value_symbol(params, return_type, type_params, *public),
                        *span,
                    );
                    self.require_resolver_callable_locals(symbols, params, body, &mut scope_cursor);
                }
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    public,
                    span,
                    ..
                } => {
                    if self
                        .require_resolver_struct_symbol(
                            symbols,
                            name,
                            expected_struct_symbol(type_params, fields, *public),
                            *span,
                        )
                        .is_none()
                    {
                        continue;
                    };
                    for field in fields {
                        if let Some(default) = &field.default {
                            self.require_resolver_scoped_expr_locals(
                                symbols,
                                default,
                                &mut scope_cursor,
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
                    self.require_resolver_enum_symbol(
                        symbols,
                        name,
                        expected_enum_symbol(type_params, variants, *public),
                        *span,
                    );
                    for variant in variants {
                        self.require_resolver_variant_symbol(
                            symbols,
                            &variant.name,
                            expected_variant_symbol(name, *public, &variant.payload),
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
                    if self
                        .require_resolver_behavior_symbol(
                            symbols,
                            name,
                            expected_behavior_symbol(type_params, methods, *public),
                            *span,
                        )
                        .is_none()
                    {
                        continue;
                    };
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            self.require_resolver_callable_locals(
                                symbols,
                                &method.params,
                                default_body,
                                &mut scope_cursor,
                            );
                        }
                    }
                }
                Declaration::Import {
                    names,
                    module_path,
                    span,
                } => {
                    self.require_resolver_module_symbol(
                        symbols,
                        expected_module_symbol(&module_path.join(".")),
                        *span,
                    );
                    for name in names {
                        self.require_resolver_import_symbol(
                            symbols,
                            name,
                            expected_import_symbol(&module_path.join(".")),
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
                                expected_behavior_edge(behavior, behavior_type_args),
                                *span,
                            );
                        }
                    }
                    self.validate_generic_type_arg_refs_allow_unknowns(behavior_type_args, *span);
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
                                &Self::method_key(type_name, name),
                                expected_value_symbol(params, return_type, type_params, *public),
                                *span,
                            );
                            self.require_resolver_callable_locals(
                                symbols,
                                params,
                                body,
                                &mut scope_cursor,
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
                            expected_behavior_edge(behavior, behavior_type_args),
                            *span,
                        );
                    }
                    self.validate_generic_type_arg_refs_allow_unknowns(behavior_type_args, *span);
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    span,
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    self.require_resolver_symbol(symbols, Namespace::Behavior, parent, *span);
                    self.validate_generic_type_arg_refs_allow_unknowns(parent_type_args, *span);
                    if let Some(symbol) = symbols.lookup(Namespace::Behavior, behavior) {
                        self.validate_resolver_behavior_parent_names(
                            symbol,
                            behavior,
                            expected_behavior_edge(parent, parent_type_args),
                            *span,
                        );
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.require_resolver_scoped_expr_locals(symbols, expr, &mut scope_cursor);
                }
                Declaration::Error { .. } => {}
            }
        }
        let replay_tasks = Self::collect_resolver_validation_replay_tasks(program, symbols);
        self.validate_no_extra_resolver_declaration_symbols(&replay_tasks, symbols);
        self.validate_no_extra_resolver_local_symbols(&replay_tasks, symbols);
        self.validate_resolver_behavior_association_lists(&replay_tasks);
        self.validate_stripped_resolver_import_symbols(&replay_tasks, symbols);
    }

    fn require_resolver_callable_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_parameter_locals(symbols, params, &mut locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut locals);
    }

    fn require_resolver_scoped_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut locals);
    }

    fn validate_no_extra_resolver_declaration_symbols(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let expected = &tasks.expected_symbols;
        for symbol in symbols.symbols() {
            if !expected.validate_imports
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
            if !expected
                .declarations
                .contains(&(symbol.namespace, symbol.name.clone()))
            {
                self.validate_extra_resolver_symbol(
                    symbol.namespace.diagnostic_name(),
                    &symbol.name,
                    ResolverSymbolPresenceValidation::extra_declaration_resolver_code(),
                    symbol.definition_span,
                );
            }
        }
    }

    fn validate_no_extra_resolver_local_symbols(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let expected = &tasks.expected_symbols;
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Local {
                continue;
            }
            if !expected
                .locals
                .contains(&(symbol.name.clone(), symbol.scope_id))
            {
                self.validate_extra_resolver_symbol(
                    "local",
                    &symbol.name,
                    ResolverSymbolPresenceValidation::extra_local_resolver_code(),
                    symbol.definition_span,
                );
            }
        }
    }

    fn validate_resolver_behavior_association_lists(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
    ) {
        for task in &tasks.behavior_associations.type_associations {
            self.validate_resolver_behavior_impl_list(
                task.symbol,
                task.name,
                &task.impl_edges,
                task.span,
            );
            self.validate_resolver_behavior_required_list(
                task.symbol,
                task.name,
                &task.required_edges,
                task.span,
            );
        }

        for task in &tasks.behavior_associations.behavior_parents {
            self.validate_resolver_behavior_parent_list(
                task.symbol,
                task.name,
                &task.parent_edges,
                task.span,
            );
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_behavior_association_list_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverBehaviorAssociationListTasks<'a> {
        Self::collect_resolver_validation_replay_tasks(program, symbols).behavior_associations
    }

    pub(super) fn collect_resolver_validation_replay_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverValidationReplayTasks<'a> {
        let declaration_tasks =
            Self::collect_resolver_validation_replay_declaration_tasks(program, symbols);
        let mut tasks = ResolverValidationReplayTasks {
            expected_symbols: declaration_tasks.expected_symbols,
            behavior_associations: ResolverBehaviorAssociationListTasks::default(),
        };

        for source in declaration_tasks.type_declarations {
            Self::push_resolver_type_behavior_association_list_task(
                source,
                &declaration_tasks.expected_associations,
                &mut tasks.behavior_associations.type_associations,
            );
        }
        for source in declaration_tasks.behavior_declarations {
            Self::push_resolver_behavior_parent_list_task(
                source,
                &declaration_tasks.expected_parents,
                &mut tasks.behavior_associations.behavior_parents,
            );
        }

        tasks
    }

    fn push_resolver_type_behavior_association_list_task<'a>(
        source: ResolverValidationBehaviorAssociationSource<'a>,
        expected: &ExpectedBehaviorAssociations,
        tasks: &mut Vec<ResolverTypeBehaviorAssociationListTask<'a>>,
    ) {
        tasks.push(ResolverTypeBehaviorAssociationListTask {
            symbol: source.symbol,
            name: source.name,
            impl_edges: expected.impls.owned_edges_for(source.name),
            required_edges: expected.required.owned_edges_for(source.name),
            span: source.span,
        });
    }

    fn push_resolver_behavior_parent_list_task<'a>(
        source: ResolverValidationBehaviorAssociationSource<'a>,
        expected: &ExpectedBehaviorEdges,
        tasks: &mut Vec<ResolverBehaviorParentListTask<'a>>,
    ) {
        tasks.push(ResolverBehaviorParentListTask {
            symbol: source.symbol,
            name: source.name,
            parent_edges: expected.owned_edges_for(source.name),
            span: source.span,
        });
    }

    pub(super) fn collect_resolver_validation_replay_declaration_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverValidationReplayDeclarationTasks<'a> {
        let mut tasks = ResolverValidationReplayDeclarationTasks::default();
        let mut scope_cursor = ResolverScopeCursor::default();

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name, params, body, ..
                } => {
                    push_expected_resolver_callable_symbol(
                        name.clone(),
                        params,
                        body,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    params,
                    body,
                    ..
                } => {
                    push_expected_resolver_callable_symbol(
                        method_signature_key(type_name, method_name),
                        params,
                        body,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Struct {
                    name, fields, span, ..
                } => {
                    for field in fields {
                        if let Some(default) = &field.default {
                            push_expected_resolver_scoped_expr_symbols(
                                default,
                                &mut scope_cursor,
                                &mut tasks.expected_symbols,
                            );
                        }
                    }
                    push_resolver_validation_association_source(
                        Namespace::Type,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.type_declarations,
                    );
                }
                Declaration::Enum {
                    name,
                    variants,
                    span,
                    ..
                } => {
                    push_expected_resolver_variant_symbols(variants, &mut tasks.expected_symbols);
                    push_resolver_validation_association_source(
                        Namespace::Type,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.type_declarations,
                    );
                }
                Declaration::Behavior {
                    name,
                    methods,
                    span,
                    ..
                } => {
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            expected_resolver_callable_locals(
                                &method.params,
                                default_body,
                                &mut scope_cursor,
                                &mut tasks.expected_symbols.locals,
                            );
                        }
                    }
                    push_resolver_validation_association_source(
                        Namespace::Behavior,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.behavior_declarations,
                    );
                }
                Declaration::Import {
                    names, module_path, ..
                } => {
                    push_expected_resolver_import_symbols(
                        names,
                        module_path,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(behavior),
                    methods,
                    behavior_type_args,
                    ..
                } => {
                    collect_expected_resolver_impl_method_symbols(
                        type_name,
                        methods,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                    push_expected_behavior_impl_edge(
                        &mut tasks.expected_associations,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    collect_expected_resolver_impl_method_symbols(
                        type_name,
                        methods,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    ..
                } => {
                    push_expected_behavior_required_edge(
                        &mut tasks.expected_associations,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    ..
                } => {
                    push_expected_behavior_parent_edge(
                        &mut tasks.expected_parents,
                        behavior,
                        parent,
                        parent_type_args,
                    );
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    push_expected_resolver_scoped_expr_symbols(
                        expr,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                _ => {}
            }
        }

        tasks
    }

    fn require_resolver_module_symbol(
        &mut self,
        symbols: &SymbolTable,
        expected: ExpectedModuleSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Module, &expected.name) else {
            self.require_resolver_symbol(symbols, Namespace::Module, &expected.name, span);
            return;
        };

        self.validate_resolver_visibility(
            "module",
            &expected.name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "module",
            &expected.name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "module",
            &expected.name,
            ValueSignatureAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "module",
            &expected.name,
            TypeParameterAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "module",
            &expected.name,
            FieldAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "module",
            &expected.name,
            VariantAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorAssociationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorDeclarationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "module",
            &expected.name,
            MutabilityAbsenceValidation::module_resolver_code(),
            span,
        );
    }

    fn validate_stripped_resolver_import_symbols(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        if tasks.expected_symbols.validate_imports {
            return;
        }

        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            self.validate_resolver_visibility(
                "import",
                &symbol.name,
                symbol.is_public,
                false,
                VisibilityValidation::import_resolver_code(),
                symbol.definition_span,
            );
            if symbol.import_source.is_none() {
                self.validate_resolver_source(
                    "import",
                    &symbol.name,
                    symbol.import_source.as_deref(),
                    Some("a module source"),
                    SourceValidation::stripped_import_resolver_code(),
                    symbol.definition_span,
                );
            } else if let Some(source) = symbol.import_source.as_deref() {
                self.require_resolver_module_symbol(
                    symbols,
                    expected_module_symbol(source),
                    symbol.definition_span,
                );
            }
            self.validate_resolver_import_absent_declaration_metadata(
                symbol,
                &symbol.name,
                symbol.definition_span,
            );
        }
    }

    pub(super) fn collect_resolver_imports(&mut self, symbols: &SymbolTable) {
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

    pub(super) fn collect_module_graph_imports(
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
        template.attach_source_dependencies(dependencies);
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
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    behavior_info_from_ast_methods(local_name.to_string(), type_params, methods),
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
                self.functions.insert(
                    local_name.to_string(),
                    func_info_from_ast_signature(
                        local_name.to_string(),
                        type_params,
                        params,
                        return_type,
                    ),
                );
                if let Some(template) =
                    generic_template_from_type_params(type_params, params, return_type, body, *span)
                {
                    self.generic_functions
                        .insert(local_name.to_string(), template);
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
                let key = Self::method_key(type_name, method_name);
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

            let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
            let parents = self
                .behavior_extends
                .entry(local_name.to_string())
                .or_default();
            if parents
                .iter()
                .any(|existing| existing.key == parent_ref.key)
            {
                continue;
            }

            parents.push(parent_ref);
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

            self.insert_behavior_impl_ref(local_name, behavior, behavior_type_args);

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
                self.seed_behavior_default_method_signature(local_name, &default);
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
                        &Self::method_key(type_name, method_name),
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
                                &Self::method_key(type_name, name),
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
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                dependencies.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
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
                    Self::insert_source_imported_type_method_dependency(
                        &Self::method_key(local_name, method_name),
                        decl,
                        imported_module,
                        graph,
                        dependencies,
                    );
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
                        Self::insert_source_imported_type_method_dependency(
                            &Self::method_key(local_name, name),
                            method,
                            imported_module,
                            graph,
                            dependencies,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn insert_source_imported_type_method_dependency(
        key: &str,
        decl: &Declaration,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        Self::insert_source_method_dependency(
            key,
            decl,
            &mut dependencies.methods,
            &mut dependencies.generic_methods,
        );
        if let Some(template) = dependencies.generic_methods.get_mut(key) {
            let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
            Self::attach_template_dependencies(template, nested_dependencies);
        }
    }

    fn insert_source_function_dependency(
        key: &str,
        decl: &Declaration,
        functions: &mut HashMap<String, FuncInfo>,
        generic_functions: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl) {
            Self::insert_source_callable_dependency(signature, functions, generic_functions);
        }
    }

    fn insert_source_method_dependency(
        key: &str,
        decl: &Declaration,
        methods: &mut HashMap<String, FuncInfo>,
        generic_methods: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl)
            .or_else(|| ImportedMethodSignature::from_method_declaration(key, decl))
        {
            Self::insert_source_callable_dependency(signature, methods, generic_methods);
        }
    }

    fn insert_source_callable_dependency(
        signature: ImportedMethodSignature<'_>,
        callables: &mut HashMap<String, FuncInfo>,
        generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        callables.insert(
            signature.name.to_string(),
            signature.func_info(signature.name.to_string()),
        );
        if let Some(template) = signature.generic_template() {
            generic_callables.insert(signature.name.to_string(), template);
        }
    }

    fn seed_imported_method_with_dependencies(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Method { method_name, .. } = method else {
            return;
        };
        let Some(signature) = ImportedMethodSignature::from_method_declaration(method_name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(local_type_name, signature, dependencies);
    }

    fn seed_imported_impl_method(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        public_only: bool,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Function { name, public, .. } = method else {
            return;
        };
        if public_only && !*public {
            return;
        }
        let Some(signature) = ImportedMethodSignature::from_function_declaration(name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(local_type_name, signature, dependencies);
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        signature: ImportedMethodSignature<'_>,
        dependencies: &SourceModuleDependencies,
    ) {
        let key = Self::method_key(local_type_name, signature.name);
        self.methods
            .insert(key.clone(), signature.func_info(key.clone()));
        if let Some(template) = signature.generic_template() {
            self.generic_methods
                .insert(key, dependencies.apply_to_template(template));
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
            self.validate_missing_resolver_symbol(
                namespace.diagnostic_name(),
                name,
                ResolverSymbolPresenceValidation::missing_resolver_code(),
                span,
            );
        }
    }

    fn require_resolver_import_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedImportSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Import, name) else {
            self.require_resolver_symbol(symbols, Namespace::Import, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "import",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "import",
            name,
            symbol.import_source.as_deref(),
            Some(expected.source.as_str()),
            SourceValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_import_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_import_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "import",
            name,
            ValueSignatureAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "import",
            name,
            TypeParameterAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "import",
            name,
            FieldAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "import",
            name,
            VariantAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "import",
            name,
            BehaviorAssociationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "import",
            name,
            BehaviorDeclarationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "import",
            name,
            MutabilityAbsenceValidation::import_resolver_code(),
            span,
        );
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
                expected_local_symbol(param.mutable, locals.current_scope_id),
                param.span,
            );
            locals.insert(param.name.clone(), param.mutable);
        }
    }

    fn require_resolver_child_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut child_locals = scope_cursor.child_scope(locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut child_locals);
    }

    fn require_resolver_pattern_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut pattern_locals = scope_cursor.child_scope(locals);
        self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, &mut pattern_locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut pattern_locals);
    }

    fn require_resolver_block_locals(
        &mut self,
        symbols: &SymbolTable,
        statements: &[ast::Statement],
        expr: Option<&Expression>,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
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
            self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut block_locals);
        }
    }

    fn require_resolver_closure_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut closure_locals = scope_cursor.child_scope(locals);
        self.require_resolver_parameter_locals(symbols, params, &mut closure_locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut closure_locals);
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
                        self.require_resolver_pattern_expr_locals(
                            symbols,
                            &arm.pattern,
                            guard,
                            scope_cursor,
                            locals,
                        );
                    }
                    self.require_resolver_pattern_expr_locals(
                        symbols,
                        &arm.pattern,
                        &arm.body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::Loop { body, .. } => {
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, then_body, scope_cursor, locals);
                if let Some(else_body) = else_body {
                    self.require_resolver_child_expr_locals(
                        symbols,
                        else_body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                self.require_resolver_block_locals(
                    symbols,
                    statements,
                    expr.as_deref(),
                    scope_cursor,
                    locals,
                );
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::Closure { params, body, .. } => {
                self.require_resolver_closure_locals(symbols, params, body, scope_cursor, locals);
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
            | Expression::LoopControl { .. }
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
                if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                    self.require_resolver_var_decl_local(symbols, name, *mutable, *span, locals);
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
                self.require_resolver_block_locals(symbols, stmts, None, scope_cursor, locals);
            }
        }
    }

    fn require_resolver_var_decl_local(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        mutable: bool,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(mutable, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), mutable);
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
                self.require_resolver_pattern_binding(symbols, name, *span, locals);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested, scope_cursor, locals);
                    } else {
                        self.require_resolver_pattern_binding(symbols, name, *span, locals);
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

    fn require_resolver_pattern_binding(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(false, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), false);
    }

    fn require_resolver_local_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedLocalSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup_in_scope(Namespace::Local, name, expected.scope_id)
        else {
            self.validate_missing_resolver_symbol(
                "local",
                name,
                ResolverSymbolPresenceValidation::missing_local_resolver_code(),
                span,
            );
            return;
        };

        self.validate_resolver_mutability(
            "local",
            name,
            symbol.is_mutable,
            expected.is_mutable,
            MutabilityValidation::resolver_code(),
            span,
        );

        self.validate_resolver_visibility(
            "local",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "local",
            name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "local",
            name,
            ValueSignatureAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "local",
            name,
            TypeParameterAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "local",
            name,
            FieldAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "local",
            name,
            VariantAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "local",
            name,
            BehaviorAssociationAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "local",
            name,
            BehaviorDeclarationAbsenceValidation::local_resolver_codes(),
            span,
        );
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
            self.validate_resolver_visibility(
                namespace.diagnostic_name(),
                name,
                symbol.is_public,
                expected_is_public,
                VisibilityValidation::type_like_resolver_code(),
                span,
            );
        }

        self.validate_resolver_type_parameters(
            symbol,
            namespace.diagnostic_name(),
            name,
            &expected.type_params,
            TypeParameterValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_type_like_absent_value_metadata(symbol, namespace, name, span);

        Some(symbol)
    }

    fn require_resolver_struct_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedStructSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_fields(symbol, Namespace::Type, name, &expected.fields, span);
        self.validate_resolver_struct_absent_enum_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_enum_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedEnumSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_variant_names(symbol, name, &expected.variant_names, span);
        self.validate_resolver_enum_absent_struct_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_variant_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedVariantSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup_variant(&expected.owner_name, name) else {
            if let Some(symbol) = symbols.lookup(Namespace::Variant, name) {
                self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
                return None;
            }
            self.require_resolver_symbol(symbols, Namespace::Variant, name, span);
            return None;
        };

        self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
        self.validate_resolver_variant_visibility(symbol, name, expected.is_public, span);
        self.validate_resolver_variant_payload(symbol, name, expected.payload, span);
        self.validate_resolver_variant_absent_other_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_behavior_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedBehaviorSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Behavior,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_behavior_methods(symbol, name, &expected.methods, span);
        self.validate_resolver_behavior_absent_type_metadata(symbol, name, span);

        Some(symbol)
    }

    fn validate_resolver_absent_value_signature_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: ValueSignatureAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_type_parameter_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: TypeParameterAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_field_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: FieldAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_variant_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: VariantAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_association_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorAssociationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorDeclarationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_mutability_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: MutabilityAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_source_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: SourceAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_source(
            symbol_kind,
            name,
            symbol.import_source.as_deref(),
            None,
            validation.source_validation(),
            span,
        );
    }

    fn validate_resolver_source(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<&str>,
        expected: Option<&str>,
        validation: SourceValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_mutability(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<bool>,
        expected: bool,
        validation: MutabilityValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_extra_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    fn validate_missing_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    pub(super) fn validate_resolver_symbol_presence(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            validation.code,
            validation.message(symbol_kind, name),
            span,
        ));
    }

    fn validate_resolver_absent_metadata_entry(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entry: AbsentMetadataEntry,
        span: Span,
    ) {
        if entry.present {
            self.diagnostics.push(Diagnostic::error(
                entry.code,
                entry.message(symbol_kind, name),
                span,
            ));
        }
    }

    fn validate_resolver_absent_metadata<const N: usize>(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: impl AbsentMetadataValidation<N>,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_metadata_entries(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entries: &[AbsentMetadataEntry],
        span: Span,
    ) {
        for entry in entries {
            self.validate_resolver_absent_metadata_entry(symbol_kind, name, *entry, span);
        }
    }

    fn validate_resolver_visibility(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: bool,
        expected: bool,
        validation: VisibilityValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_count(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<usize>,
        expected: usize,
        validation: CountValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_type_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        expected: &[ExpectedTypeParameter],
        validation: TypeParameterValidation,
        span: Span,
    ) {
        let expected = ExpectedTypeParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            symbol_kind,
            name,
            symbol.type_parameter_count,
            expected.count,
            validation.count_validation(),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_names.as_deref(),
            &expected.names,
            format_type_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bounds.as_deref(),
            &expected.bounds,
            format_type_parameter_bounds,
            validation.bound_code,
            |actual, expected| validation.bound_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bound_refs.as_deref(),
            &expected.bound_refs,
            format_type_parameter_bound_refs,
            validation.bound_ref_code,
            |actual, expected| validation.bound_ref_message(symbol_kind, name, actual, expected),
            span,
        );
    }

    fn validate_resolver_metadata_list<T: PartialEq>(
        &mut self,
        actual: Option<&[T]>,
        expected: &[T],
        display: impl Fn(Option<&[T]>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != Some(expected) {
            let actual_display = display(actual);
            let expected_display = display(Some(expected));
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
                span,
            ));
        }
    }

    fn validate_resolver_metadata_value<T: PartialEq + ?Sized>(
        &mut self,
        actual: Option<&T>,
        expected: Option<&T>,
        display: impl Fn(Option<&T>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != expected {
            let actual_display = display(actual);
            let expected_display = display(expected);
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
                span,
            ));
        }
    }

    fn validate_resolver_type_like_absent_value_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            SourceAbsenceValidation::type_like_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            ValueSignatureAbsenceValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            MutabilityAbsenceValidation::type_like_resolver_code(),
            span,
        );
    }

    fn validate_resolver_fields(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        expected_fields: &[ExpectedField],
        span: Span,
    ) {
        let expected = ExpectedFieldMetadata::from_fields(expected_fields);
        self.validate_resolver_count(
            namespace.diagnostic_name(),
            name,
            symbol.field_count,
            expected.count,
            CountValidation::field_resolver_code(),
            span,
        );
        let validation = FieldValidation::resolver_codes();
        let symbol_kind = namespace.diagnostic_name();
        self.validate_resolver_metadata_list(
            symbol.field_types.as_deref(),
            &expected.typed,
            format_field_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(symbol_kind, name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.field_type_names.as_deref(),
            &expected.display,
            format_field_type_names,
            validation.display_code,
            |actual, expected| validation.display_message(symbol_kind, name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_variant_names: &[String],
        span: Span,
    ) {
        let validation = VariantNameValidation::resolver_code();
        self.validate_resolver_metadata_list(
            symbol.variant_names.as_deref(),
            expected_variant_names,
            format_variant_names,
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_struct_absent_enum_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_variant_metadata(
            symbol,
            "type",
            name,
            VariantAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_enum_absent_struct_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "type",
            name,
            FieldAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_variant_payload(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_payload: ExpectedVariantPayloadType,
        span: Span,
    ) {
        let expected = ExpectedVariantPayloadMetadata::from_payload(expected_payload);
        self.validate_resolver_count(
            "variant",
            name,
            symbol.variant_payload_count,
            expected.count,
            CountValidation::variant_payload_resolver_code(),
            span,
        );
        let validation = VariantPayloadValidation::resolver_codes();
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type.as_ref(),
            expected.typed.as_ref(),
            |value| optional_ast_type_display(value, "none"),
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type_name.as_deref(),
            expected.display.as_deref(),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_owner_name(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_owner_name: &str,
        span: Span,
    ) {
        let validation = VariantOwnerValidation::resolver_code();
        self.validate_resolver_metadata_value(
            symbol.variant_owner_name.as_deref(),
            Some(expected_owner_name),
            |value| resolver_metadata_display(value).to_string(),
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_visibility(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_is_public: bool,
        span: Span,
    ) {
        self.validate_resolver_visibility(
            "variant",
            name,
            symbol.is_public,
            expected_is_public,
            VisibilityValidation::variant_resolver_code(),
            span,
        );
    }

    fn validate_resolver_variant_absent_other_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "variant",
            name,
            SourceAbsenceValidation::variant_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "variant",
            name,
            ValueSignatureAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "variant",
            name,
            TypeParameterAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "variant",
            name,
            FieldAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "variant",
            name,
            BehaviorAssociationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "variant",
            name,
            BehaviorDeclarationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_metadata_entries(
            "variant",
            name,
            &[AbsentMetadataEntry::new(
                symbol.variant_names.is_some(),
                "E0338",
                "variant names",
            )],
            span,
        );
        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "variant",
            name,
            MutabilityAbsenceValidation::variant_resolver_code(),
            span,
        );
    }

    fn validate_resolver_behavior_methods(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_methods: &[ExpectedBehaviorMethod],
        span: Span,
    ) {
        let expected = ExpectedBehaviorMethodMetadata::from_methods(expected_methods);
        let validation = BehaviorMethodValidation::resolver_codes();
        self.validate_resolver_metadata_list(
            symbol.behavior_method_signatures.as_deref(),
            &expected.signatures,
            format_behavior_method_signatures,
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.behavior_method_types.as_deref(),
            &expected.typed,
            format_behavior_method_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_behavior_absent_type_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "behavior",
            name,
            FieldAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "behavior",
            name,
            VariantAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "behavior",
            name,
            BehaviorAssociationAbsenceValidation::behavior_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_behavior_parent_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_parent_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    pub(super) fn validate_resolver_behavior_ref_contains_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::Contains),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_list_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::List),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_contains(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        if !actual.contains_display(&expected.display) {
            let actual = format_behavior_ref_names(actual.names);
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.contains_name_message(name, &actual, &expected.display),
                span,
            ));
        }
        if !actual.contains_metadata(&expected.metadata) {
            let actual = format_behavior_refs(actual.refs);
            let expected_ref =
                behavior_ref_display(&expected.metadata.name, &expected.metadata.type_args);
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.contains_ref_message(name, &actual, &expected_ref),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_ref_list(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        let expected = ExpectedBehaviorEdgeMetadata::from_edges(expected);
        if !actual.names_match(&expected.names) {
            let actual = format_behavior_ref_names(actual.names);
            let expected_names = format_behavior_ref_names(Some(&expected.names));
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.list_name_message(name, &actual, &expected_names),
                span,
            ));
        }
        if !actual.refs_match(&expected.refs) {
            let actual = format_behavior_refs(actual.refs);
            let expected_refs = format_behavior_refs(Some(&expected.refs));
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.list_ref_message(name, &actual, &expected_refs),
                span,
            ));
        }
    }

    fn require_resolver_value_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedValueSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.require_resolver_symbol(symbols, Namespace::Value, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "value",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_value_parameters(symbol, name, &expected.signature.params, span);
        self.validate_resolver_value_return_type(
            symbol,
            name,
            &expected.signature.return_type,
            span,
        );

        self.validate_resolver_type_parameters(
            symbol,
            "value",
            name,
            &expected.signature.type_params,
            TypeParameterValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_value_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_value_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedParameter],
        span: Span,
    ) {
        let expected = ExpectedParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            "value",
            name,
            symbol.parameter_count,
            expected.count,
            CountValidation::value_parameter_resolver_code(),
            span,
        );

        let validation = ValueParameterValidation::resolver_codes();

        self.validate_resolver_metadata_list(
            symbol.parameter_names.as_deref(),
            &expected.names,
            format_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_type_names.as_deref(),
            &expected.display_types,
            format_parameter_type_names,
            validation.display_type_code,
            |actual, expected| validation.display_type_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_types.as_deref(),
            &expected.typed_types,
            format_ast_type_list,
            validation.typed_type_code,
            |actual, expected| validation.typed_type_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_value_return_type(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &ExpectedReturnMetadata,
        span: Span,
    ) {
        let validation = ReturnValidation::resolver_codes();

        self.validate_resolver_metadata_value(
            symbol.return_type_name.as_deref(),
            Some(expected.display.as_str()),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.return_type.as_ref(),
            Some(&expected.typed),
            resolver_ast_type_metadata_display,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_value_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "value",
            name,
            SourceAbsenceValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "value",
            name,
            FieldAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "value",
            name,
            VariantAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "value",
            name,
            BehaviorAssociationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "value",
            name,
            BehaviorDeclarationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "value",
            name,
            MutabilityAbsenceValidation::value_resolver_code(),
            span,
        );
    }
}
