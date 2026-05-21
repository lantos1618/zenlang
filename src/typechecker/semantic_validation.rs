use super::*;

impl TypeChecker {
    pub(super) fn validate_collected_declaration_semantics(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_declaration_semantic_validation_tasks(decls);
            self.validate_resolver_declaration_semantics_from_semantic_tasks(&tasks, symbols);
            return;
        }

        let tasks = Self::collect_ast_declaration_validation_tasks(decls);
        self.validate_ast_declaration_semantics_from_tasks(&tasks, symbols);
    }

    pub(super) fn validate_ast_declaration_semantics_from_tasks(
        &mut self,
        tasks: &AstDeclarationValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        self.validate_behavior_association_tasks(tasks, symbols);
        self.validate_ast_type_reference_tasks(&tasks.type_references);
        self.validate_ast_struct_field_default_tasks(&tasks.struct_field_defaults);
    }

    pub(super) fn validate_resolver_declaration_semantics_from_semantic_tasks(
        &mut self,
        tasks: &ResolverDeclarationSemanticValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        self.validate_behavior_association_tasks(tasks, symbols);
        self.validate_resolver_type_reference_task_list(&tasks.type_references, symbols);
        self.validate_resolver_struct_field_default_task_list(&tasks.struct_defaults, symbols);
    }

    pub(super) fn collect_ast_declaration_validation_tasks(
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
    pub(super) fn collect_behavior_association_validation_tasks(
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
}
