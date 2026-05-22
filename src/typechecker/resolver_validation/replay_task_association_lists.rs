impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_resolver_behavior_association_list_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverBehaviorAssociationListTasks<'a> {
        Self::collect_resolver_validation_replay_tasks(program, symbols).behavior_associations
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
}
