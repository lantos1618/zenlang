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
                            let method_key = Self::behavior_impl_method_key(
                                type_name,
                                name,
                                behavior.as_deref(),
                                behavior_type_args,
                            );
                            self.require_resolver_value_symbol(
                                symbols,
                                &method_key,
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
                Declaration::Derive { .. } | Declaration::Error { .. } => {}
            }
        }
        let replay_tasks = Self::collect_resolver_validation_replay_tasks(program, symbols);
        self.validate_no_extra_resolver_declaration_symbols(&replay_tasks, symbols);
        self.validate_no_extra_resolver_local_symbols(&replay_tasks, symbols);
        self.validate_resolver_behavior_association_lists(&replay_tasks);
        self.validate_stripped_resolver_import_symbols(&replay_tasks, symbols);
    }

}
