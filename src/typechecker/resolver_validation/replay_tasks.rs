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
}
