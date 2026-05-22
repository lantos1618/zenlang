use super::behavior_impl_validation::BehaviorImplValidationInput;
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

    pub(super) fn validate_behavior_association_tasks<'a>(
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
            self.validate_collected_behavior_impl_declaration(symbols, task);
        }
    }

    pub(super) fn push_behavior_requires_replay_task<'a>(
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
        task: &ResolverBehaviorImplBlockDeclarationTask<'_>,
    ) {
        let restored_type_name = symbols
            .map(|symbols| {
                self.resolver_impl_type_name_for(
                    symbols,
                    task.ast_type_name,
                    task.methods,
                    Some((task.behavior, task.behavior_type_args)),
                )
            })
            .unwrap_or_else(|| task.ast_type_name.to_string());
        self.check_behavior_impl(BehaviorImplValidationInput {
            type_name: &restored_type_name,
            type_args: task.type_args,
            behavior: task.behavior,
            behavior_type_args: task.behavior_type_args,
            methods: task.methods,
            span: task.span,
            symbols,
        });
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

    pub(super) fn validate_collected_behavior_extends_semantics(&mut self) {
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
}
