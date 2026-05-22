struct ResolverImplBlockEntry<'a> {
    type_name: &'a str,
    type_args: &'a [AstType],
    behavior: &'a Option<String>,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
    span: Span,
}

impl TypeChecker {
    fn validate_resolver_impl_block_entry_symbols(
        &mut self,
        symbols: &SymbolTable,
        entry: ResolverImplBlockEntry<'_>,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let type_symbol = symbols.lookup(Namespace::Type, entry.type_name);
        if type_symbol.is_none() {
            self.require_resolver_symbol(symbols, Namespace::Type, entry.type_name, entry.span);
        }
        if let Some(behavior) = entry.behavior {
            self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, entry.span);
            if let Some(symbol) = type_symbol {
                self.validate_resolver_behavior_impl_names(
                    symbol,
                    entry.type_name,
                    expected_behavior_edge(behavior, entry.behavior_type_args),
                    entry.span,
                );
            }
        }
        self.validate_generic_type_arg_refs_allow_unknowns(entry.behavior_type_args, entry.span);
        for method in entry.methods {
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
                let method_key = Self::behavior_impl_method_key_with_target_args(
                    entry.type_name,
                    name,
                    entry.behavior.as_deref(),
                    entry.behavior_type_args,
                    entry.type_args,
                );
                self.require_resolver_value_symbol(
                    symbols,
                    &method_key,
                    expected_value_symbol(params, return_type, type_params, *public),
                    *span,
                );
                self.require_resolver_callable_locals(symbols, params, body, scope_cursor);
            }
        }
    }

    fn validate_resolver_requires_entry_symbols(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let type_symbol = symbols.lookup(Namespace::Type, type_name);
        if type_symbol.is_none() {
            self.require_resolver_symbol(symbols, Namespace::Type, type_name, span);
        }
        self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, span);
        if let Some(symbol) = type_symbol {
            self.validate_resolver_behavior_required_names(
                symbol,
                type_name,
                expected_behavior_edge(behavior, behavior_type_args),
                span,
            );
        }
        self.validate_generic_type_arg_refs_allow_unknowns(behavior_type_args, span);
    }

    fn validate_resolver_behavior_extends_entry_symbols(
        &mut self,
        symbols: &SymbolTable,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
    ) {
        self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, span);
        self.require_resolver_symbol(symbols, Namespace::Behavior, parent, span);
        self.validate_generic_type_arg_refs_allow_unknowns(parent_type_args, span);
        if let Some(symbol) = symbols.lookup(Namespace::Behavior, behavior) {
            self.validate_resolver_behavior_parent_names(
                symbol,
                behavior,
                expected_behavior_edge(parent, parent_type_args),
                span,
            );
        }
    }
}
