use super::*;

impl TypeChecker {
    pub(super) fn collect_resolver_behavior_impl_metadata(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let impl_tasks = self.resolver_behavior_impl_block_tasks(tasks, symbols);

        self.with_resolver_backed_collection(|checker| {
            for task in &impl_tasks {
                checker.collect_resolver_behavior_impl_method_signatures(
                    symbols,
                    task.ast_type_name,
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }

            checker.validate_collected_behavior_extends_semantics();

            for task in &impl_tasks {
                checker.collect_behavior_default_method_signatures(
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        });
    }

    pub(super) fn validate_resolver_collected_declaration_semantics(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.with_resolver_backed_collection(|checker| {
            checker.validate_resolver_declaration_semantics_from_tasks(tasks, Some(symbols));
        });
    }

    pub(super) fn clear_resolver_behavior_ref_state(&mut self) {
        self.resolver_behavior_impl_refs.clear();
        self.resolver_behavior_required_refs.clear();
        self.resolver_missing_behavior_impl_refs.clear();
        self.resolver_missing_behavior_required_refs.clear();
    }

    pub(super) fn refresh_resolver_type_behavior_impls(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        for task in self.resolver_type_behavior_refresh_tasks(tasks, symbols) {
            self.collect_resolver_type_behavior_impls(symbols, &task.restored_name);
        }
    }

    pub(super) fn with_resolver_backed_collection(&mut self, collect: impl FnOnce(&mut Self)) {
        let previous = self.resolver_backed_collection;
        self.resolver_backed_collection = true;
        collect(self);
        self.resolver_backed_collection = previous;
    }

    fn resolver_behavior_impl_block_tasks<'a>(
        &self,
        tasks: &'a ResolverDeclarationMetadataTasks<'a>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverBehaviorImplBlockTask<'a>> {
        let mut impl_tasks = Vec::new();
        for raw_task in &tasks.behavior_associations.impls {
            let restored_type_name = self.resolver_impl_type_name_for(
                symbols,
                raw_task.ast_type_name,
                raw_task.methods,
                Some((raw_task.behavior, raw_task.behavior_type_args)),
            );
            impl_tasks.push(ResolverBehaviorImplBlockTask {
                ast_type_name: raw_task.ast_type_name,
                restored_type_name,
                behavior: raw_task.behavior,
                behavior_type_args: raw_task.behavior_type_args,
                methods: raw_task.methods,
            });
        }
        impl_tasks
    }

    pub(super) fn resolver_type_behavior_refresh_tasks(
        &self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverTypeBehaviorRefreshTask> {
        let mut refresh_tasks = Vec::new();
        for type_task in &tasks.types {
            match type_task {
                ResolverTypeDeclarationMetadataTask::Struct { name, span, .. }
                | ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    let restored_name =
                        Self::resolver_symbol_name_for(symbols, Namespace::Type, name, *span);
                    refresh_tasks.push(ResolverTypeBehaviorRefreshTask { restored_name });
                }
            }
        }
        refresh_tasks
    }

    pub(super) fn collect_resolver_type_behavior_refs_for_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) -> String {
        let restored_name = Self::resolver_symbol_name_for(symbols, Namespace::Type, name, span);
        self.collect_resolver_type_behavior_impl_refs(symbols, &restored_name);
        self.collect_resolver_type_behavior_requires(symbols, &restored_name);
        restored_name
    }

    pub(super) fn collect_resolver_behavior_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name =
            Self::resolver_symbol_name_for(symbols, Namespace::Behavior, name, span);
        self.rekey_behavior_declaration(name, &restored_name);
        self.collect_resolver_behavior_methods(symbols, &restored_name);
        self.collect_resolver_behavior_parents(symbols, &restored_name);
    }

    fn rekey_behavior_declaration(&mut self, old_name: &str, new_name: &str) {
        if old_name == new_name {
            return;
        }
        if let Some(info) = self.behaviors.remove(old_name) {
            self.behaviors.insert(
                new_name.to_string(),
                BehaviorInfo {
                    name: new_name.to_string(),
                    ..info
                },
            );
        }
    }
}
