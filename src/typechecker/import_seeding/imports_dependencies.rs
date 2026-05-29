impl TypeChecker {
    fn seed_imported_callable_signature_type_dependencies(
        &mut self,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let mut type_names = HashSet::new();
        let Some(callable) = decl.as_callable() else {
            return;
        };
        for param in callable.params {
            collect_ast_type_names(&param.ty, &mut type_names);
        }
        if let Some(return_type) = callable.return_type {
            collect_ast_type_names(return_type, &mut type_names);
        }

        for type_name in type_names {
            self.seed_imported_type_dependency(&type_name, source_module, graph);
        }
    }

    fn seed_imported_type_dependency(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(type_decl) = public_type_declaration(source_module, type_name) {
            self.seed_declaration_info(type_name, type_decl);
            self.seed_public_methods_for_imported_type(type_name, source_module, graph);
            self.seed_behavior_impls_for_imported_type(type_name, type_name, source_module, graph);
            return;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == type_name)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };
        let Some(type_decl) = public_type_declaration(imported_module, binding.source_symbol.as_str())
        else {
            return;
        };
        self.seed_declaration_info(type_name, type_decl);
        self.seed_public_methods_for_imported_type(
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
        self.seed_behavior_impls_for_imported_type(
            type_name,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }

    fn seed_public_methods_for_imported_type(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let dependencies = Self::source_module_dependencies(source_module, graph);

        for decl in &source_module.program.declarations {
            match decl {
                Declaration::Method {
                    type_name: method_type,
                    ..
                } if method_type == type_name => {
                    self.seed_public_imported_method_signature(type_name, decl, &dependencies);
                }
                Declaration::ImplBlock {
                    type_name: impl_type,
                    behavior: None,
                    methods,
                    ..
                } if impl_type == type_name => {
                    for method in methods {
                        self.seed_public_imported_method_signature(type_name, method, &dependencies);
                    }
                }
                _ => {}
            }
        }
    }
}
