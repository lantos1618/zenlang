impl TypeChecker {
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
}
