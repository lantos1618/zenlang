impl TypeChecker {
    fn require_resolver_module_symbol(
        &mut self,
        symbols: &SymbolTable,
        expected: ExpectedModuleSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Module, &expected.name) else {
            self.require_resolver_symbol(symbols, Namespace::Module, &expected.name, span);
            return;
        };

        self.validate_resolver_visibility(
            "module",
            &expected.name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "module",
            &expected.name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "module",
            &expected.name,
            ValueSignatureAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "module",
            &expected.name,
            TypeParameterAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "module",
            &expected.name,
            FieldAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "module",
            &expected.name,
            VariantAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorAssociationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorDeclarationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "module",
            &expected.name,
            MutabilityAbsenceValidation::module_resolver_code(),
            span,
        );
    }

    fn validate_stripped_resolver_import_symbols(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        if tasks.expected_symbols.validate_imports {
            return;
        }

        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            self.validate_resolver_visibility(
                "import",
                &symbol.name,
                symbol.is_public,
                false,
                VisibilityValidation::import_resolver_code(),
                symbol.definition_span,
            );
            if symbol.import_source.is_none() {
                self.validate_resolver_source(
                    "import",
                    &symbol.name,
                    symbol.import_source.as_deref(),
                    Some("a module source"),
                    SourceValidation::stripped_import_resolver_code(),
                    symbol.definition_span,
                );
            } else if let Some(source) = symbol.import_source.as_deref() {
                self.require_resolver_module_symbol(
                    symbols,
                    expected_module_symbol(source),
                    symbol.definition_span,
                );
            }
            self.validate_resolver_import_absent_declaration_metadata(
                symbol,
                &symbol.name,
                symbol.definition_span,
            );
        }
    }

    pub(super) fn collect_resolver_imports(&mut self, symbols: &SymbolTable) {
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            let Some(source) = &symbol.import_source else {
                continue;
            };
            self.imports
                .entry(symbol.name.clone())
                .or_insert_with(|| source.split('.').map(str::to_string).collect());
        }
    }

    pub(super) fn collect_module_graph_imports(
        &mut self,
        graph: &ResolvedModuleGraph,
        entry: &ResolvedModule,
    ) {
        for binding in &entry.imports {
            let Some(source_module) = graph.module(binding.source_module) else {
                self.diagnostics.push(Diagnostic::error(
                    "E0233",
                    format!(
                        "module graph import '{}' points at missing module {:?}",
                        binding.local_name, binding.source_module
                    ),
                    binding.span,
                ));
                continue;
            };

            let Some(decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                self.diagnostics.push(Diagnostic::error(
                    "E0234",
                    format!(
                        "module graph import '{}' points at missing symbol '{}'",
                        binding.local_name, binding.source_symbol
                    ),
                    binding.span,
                ));
                continue;
            };

            self.seed_module_graph_import(binding.local_name.as_str(), decl);
            self.seed_imported_callable_signature_type_dependencies(decl, source_module, graph);
            self.seed_imported_generic_function_dependencies(
                binding.local_name.as_str(),
                decl,
                source_module,
                graph,
            );
            if matches!(decl, Declaration::Behavior { .. }) {
                self.seed_behavior_extends_for_imported_behavior(
                    binding.local_name.as_str(),
                    binding.source_symbol.as_str(),
                    source_module,
                    graph,
                );
            }
            self.seed_public_methods_for_imported_type(
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
            self.seed_behavior_impls_for_imported_type(
                binding.local_name.as_str(),
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
        }
    }

}
