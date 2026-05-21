impl TypeChecker {
    fn seed_imported_type_dependency(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(type_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(type_name))
        {
            if !matches!(
                type_decl,
                Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
            ) {
                return;
            }
            self.seed_module_graph_import(type_name, type_decl);
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
        let Some(type_decl) = imported_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
        else {
            return;
        };
        if !matches!(
            type_decl,
            Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
        ) {
            return;
        }
        self.seed_module_graph_import(type_name, type_decl);
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
}
