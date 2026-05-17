impl TypeChecker {
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
        let behavior_associations =
            Self::collect_resolver_behavior_association_list_tasks_from_declaration_tasks(
                &declaration_tasks,
            );

        ResolverValidationReplayTasks {
            expected_symbols: declaration_tasks.expected_symbols,
            behavior_associations,
        }
    }

    pub(super) fn collect_resolver_behavior_association_list_tasks_from_declaration_tasks<'a>(
        declaration_tasks: &ResolverValidationReplayDeclarationTasks<'a>,
    ) -> ResolverBehaviorAssociationListTasks<'a> {
        let mut tasks = ResolverBehaviorAssociationListTasks::default();

        for source in &declaration_tasks.type_declarations {
            Self::push_resolver_type_behavior_association_list_task(
                source,
                &declaration_tasks.expected_associations,
                &mut tasks.type_associations,
            );
        }
        for source in &declaration_tasks.behavior_declarations {
            Self::push_resolver_behavior_parent_list_task(
                source,
                &declaration_tasks.expected_parents,
                &mut tasks.behavior_parents,
            );
        }

        tasks
    }

    fn push_resolver_type_behavior_association_list_task<'a>(
        source: &ResolverValidationBehaviorAssociationSource<'a>,
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
        source: &ResolverValidationBehaviorAssociationSource<'a>,
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
                        Some(behavior),
                        behavior_type_args,
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
                        None,
                        &[],
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

}
