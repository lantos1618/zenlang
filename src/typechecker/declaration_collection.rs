use super::*;

impl TypeChecker {
    // ── Phase 1: Collect ──────────────────────────────────────────

    pub(super) fn collect_declarations(&mut self, decls: &[Declaration]) {
        let tasks = Self::collect_ast_declaration_collection_tasks(decls);
        self.collect_ast_declarations_from_tasks(&tasks);
    }

    pub(super) fn collect_ast_declarations_from_tasks(
        &mut self,
        tasks: &AstDeclarationCollectionTasks<'_>,
    ) {
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
            Self::push_ast_declaration_collection_tasks(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn collect_declarations_with_symbols(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        let tasks = Self::collect_declaration_collection_replay_tasks(decls);

        self.with_resolver_backed_collection(|checker| {
            checker.collect_ast_declarations_from_tasks(&tasks.ast);
        });
        self.collect_resolver_declarations_from_tasks(
            &tasks.resolver,
            &tasks.resolver_semantics,
            symbols,
        );
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
