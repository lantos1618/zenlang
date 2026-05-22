impl TypeChecker {
    fn seed_behavior_impls_for_imported_type(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name,
                type_args,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if type_name != source_name {
                continue;
            }
            if !self.imported_behavior_impl_is_public(behavior, source_module, graph) {
                continue;
            }

            self.seed_behavior_decl_for_imported_impl(behavior, behavior, source_module, graph);
            self.seed_behavior_decl_for_imported_impl_from_imports(behavior, source_module, graph);

            self.insert_behavior_impl_ref(local_name, behavior, behavior_type_args);
            self.seed_imported_generic_behavior_impl_template(
                local_name,
                type_args,
                behavior,
                behavior_type_args,
            );

            let dependencies = Self::source_module_dependencies(source_module, graph);
            for method in methods {
                self.seed_imported_impl_method(ImportedImplMethodSeed {
                    local_type_name: local_name,
                    behavior: Some(behavior),
                    behavior_type_args,
                    target_type_args: type_args,
                    method,
                    public_only: false,
                    dependencies: &dependencies,
                });
            }
            for default in self.behavior_default_methods_for_impl(
                local_name,
                type_args,
                behavior,
                behavior_type_args,
                methods,
            ) {
                self.seed_behavior_default_method_signature(local_name, &default);
            }
        }
    }

    fn imported_behavior_impl_is_public(
        &self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> bool {
        if let Some(Declaration::Behavior { public, .. }) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(behavior))
        {
            return *public;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return false;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return false;
        };
        matches!(
            imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str())),
            Some(Declaration::Behavior { public: true, .. })
        )
    }

    fn seed_behavior_decl_for_imported_impl(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(behavior_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(source_name))
        {
            self.seed_module_graph_import(local_name, behavior_decl);
            self.seed_behavior_extends_for_imported_behavior(
                local_name,
                source_name,
                source_module,
                graph,
            );
        }
    }

    fn seed_behavior_decl_for_imported_impl_from_imports(
        &mut self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };

        self.seed_behavior_decl_for_imported_impl(
            behavior,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }
}
