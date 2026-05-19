use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_behavior_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<BehaviorDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_behavior_declaration_task<'a>(
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

    pub(super) fn collect_behavior_declarations_from_tasks(
        &mut self,
        tasks: &[BehaviorDeclarationTask<'_>],
    ) {
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

    pub(super) fn validate_ast_precollection_tasks(
        &mut self,
        tasks: &AstPrecollectionValidationTasks<'_>,
    ) {
        self.validate_self_type_context_tasks(&tasks.self_type_contexts);

        if self.resolver_backed_collection {
            return;
        }

        self.validate_ast_behavior_extends_tasks(&tasks.behavior_associations);
    }

    #[cfg(test)]
    pub(super) fn collect_ast_precollection_validation_tasks(
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

    pub(super) fn push_behavior_extends_replay_task<'a>(
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
}
