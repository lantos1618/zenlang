use super::*;

impl TypeChecker {
    // ── Phase 1: Collect ──────────────────────────────────────────

    pub(super) fn collect_declarations(&mut self, decls: &[Declaration]) {
        let tasks = Self::collect_ast_declaration_collection_tasks(decls);
        self.collect_behavior_declarations_from_tasks(&tasks.behaviors);
        self.validate_ast_precollection_tasks(&tasks.precollection_validations);
        if !self.resolver_backed_collection {
            self.collect_ast_type_declarations_from_tasks(&tasks.types);
        }
        self.collect_callable_declarations_from_tasks(&tasks.callable);
        self.collect_impl_block_declarations_from_tasks(&tasks.impl_blocks);
        self.collect_ast_import_declarations_from_tasks(&tasks.imports);
    }

    pub(super) fn collect_ast_declaration_collection_tasks(
        decls: &[Declaration],
    ) -> AstDeclarationCollectionTasks<'_> {
        let mut tasks = AstDeclarationCollectionTasks::default();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks.behaviors);
            Self::push_ast_type_declaration_task(decl, &mut tasks.types);
            Self::push_callable_declaration_task(decl, &mut tasks.callable);
            Self::push_impl_block_declaration_task(decl, &mut tasks.impl_blocks);
            Self::push_ast_import_declaration_task(decl, &mut tasks.imports);
            Self::push_self_type_context_validation_task(
                decl,
                &mut tasks.precollection_validations.self_type_contexts,
            );
            Self::push_behavior_extends_replay_task(
                decl,
                &mut tasks
                    .precollection_validations
                    .behavior_associations
                    .extends,
            );
        }
        tasks
    }

    pub(super) fn collect_declarations_with_symbols(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        self.with_resolver_backed_collection(|checker| checker.collect_declarations(decls));

        let tasks = Self::collect_resolver_declaration_metadata_tasks(decls);
        self.collect_resolver_declaration_metadata(symbols, &tasks);
        self.collect_resolver_behavior_impl_metadata(&tasks, symbols);
        self.validate_resolver_collected_declaration_semantics(symbols, &tasks);
        self.clear_resolver_behavior_ref_state();
        self.refresh_resolver_type_behavior_impls(&tasks, symbols);
    }

    pub(super) fn collect_resolver_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        let mut tasks = ResolverDeclarationMetadataTasks::default();
        for decl in decls {
            let callable_handled = Self::push_resolver_callable_replay_tasks(
                decl,
                &mut tasks.callable,
                &mut tasks.type_references,
            );
            let type_handled = if callable_handled {
                false
            } else {
                Self::push_resolver_type_replay_tasks(
                    decl,
                    &mut tasks.types,
                    &mut tasks.type_references,
                )
            };
            let behavior_handled = if callable_handled || type_handled {
                false
            } else {
                Self::push_resolver_behavior_replay_tasks(
                    decl,
                    &mut tasks.behaviors,
                    &mut tasks.type_references,
                )
            };
            let behavior_impl_handled = if callable_handled || type_handled || behavior_handled {
                false
            } else {
                Self::push_resolver_behavior_impl_replay_tasks(
                    decl,
                    &mut tasks.behavior_associations.impls,
                    &mut tasks.type_references,
                )
            };
            if !callable_handled && !type_handled && !behavior_handled && !behavior_impl_handled {
                Self::push_resolver_type_reference_validation_task(
                    decl,
                    &mut tasks.type_references,
                );
            }
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
            Self::push_behavior_requires_replay_task(
                decl,
                &mut tasks.behavior_associations.requires,
            );
        }
        tasks
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_type_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_type_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
            }
            _ => {}
        }
    }

    pub(super) fn push_resolver_type_replay_tasks<'a>(
        decl: &'a Declaration,
        type_tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                true
            }
            Declaration::Enum { name, span, .. } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_behavior_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_behavior_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_behavior_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    ) {
        if let Declaration::Behavior { name, span, .. } = decl {
            tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
        }
    }

    pub(super) fn push_resolver_behavior_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior {
            name,
            methods,
            span,
            ..
        } = decl
        {
            behavior_tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
            type_reference_tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                name,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    pub(super) fn push_resolver_behavior_impl_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_impl_tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        let handled = Self::push_behavior_impl_block_declaration_task(decl, behavior_impl_tasks);
        if handled {
            let Declaration::ImplBlock {
                type_name, methods, ..
            } = decl
            else {
                return false;
            };
            type_reference_tasks
                .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
        }
        handled
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_behavior_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_behavior_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        {
            tasks.push(ResolverBehaviorImplBlockDeclarationTask {
                ast_type_name: type_name,
                behavior,
                behavior_type_args,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_callable_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverCallableDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_callable_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_callable_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Function { name, span, .. } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
            }
            Declaration::Method {
                type_name,
                method_name,
                span,
                ..
            } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
            }
            _ => {}
        }
    }

    pub(super) fn push_resolver_callable_replay_tasks<'a>(
        decl: &'a Declaration,
        callable_tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                callable_tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_resolver_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
            }
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
            }
            Declaration::Behavior {
                name,
                methods,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span: *span,
                });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    pub(super) fn collect_resolver_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.collect_resolver_callable_declaration_metadata(symbols, tasks);
        self.collect_resolver_type_declaration_metadata(symbols, tasks);
        self.collect_resolver_behavior_declaration_metadata_pass(symbols, tasks);
    }

    fn collect_resolver_behavior_declaration_metadata_pass(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.behaviors {
            self.collect_resolver_behavior_declaration(symbols, task.name, task.span);
        }
    }

    fn collect_resolver_type_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.types {
            match task {
                ResolverTypeDeclarationMetadataTask::Struct { name, fields, span } => {
                    self.collect_resolver_struct_declaration_metadata(symbols, name, fields, *span);
                }
                ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    self.collect_resolver_enum_declaration_metadata(symbols, name, *span);
                }
            }
        }
    }

    fn collect_resolver_callable_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.callable {
            match task {
                ResolverCallableDeclarationMetadataTask::Function { name, span } => {
                    self.collect_resolver_function_signature(symbols, name, *span);
                }
                ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span,
                } => {
                    self.collect_resolver_method_signature(symbols, type_name, method_name, *span);
                }
                ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods } => {
                    self.collect_resolver_type_impl_declaration_metadata(
                        symbols, type_name, methods,
                    );
                }
            }
        }
    }

    fn collect_resolver_type_impl_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, span, .. } = method {
                self.collect_resolver_method_signature(symbols, type_name, name, *span);
            }
        }
    }

    fn collect_resolver_struct_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_struct_fields(symbols, name, fields);
            },
        );
    }

    fn collect_resolver_enum_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_enum_variants(symbols, name);
            },
        );
    }

    fn collect_resolver_type_declaration_metadata_for(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        collect: impl FnOnce(&mut Self, &str),
    ) {
        let restored_name =
            self.collect_resolver_type_behavior_refs_for_declaration(symbols, name, span);
        collect(self, &restored_name);
    }
}
