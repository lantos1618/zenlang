impl TypeChecker {
    fn seed_behavior_extends_for_imported_behavior(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        self.seed_behavior_extends_for_imported_behavior_inner(
            local_name,
            source_name,
            source_module,
            graph,
            &mut HashSet::new(),
        );
    }

    fn seed_behavior_extends_for_imported_behavior_inner(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        seen: &mut HashSet<String>,
    ) {
        if !seen.insert(source_name.to_string()) {
            return;
        }

        for decl in &source_module.program.declarations {
            let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            else {
                continue;
            };
            if behavior != source_name {
                continue;
            }

            if let Some(parent_decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(parent.as_str()))
            {
                self.seed_module_graph_import(parent, parent_decl);
                self.seed_behavior_extends_for_imported_behavior_inner(
                    parent,
                    parent,
                    source_module,
                    graph,
                    seen,
                );
            } else if let Some(binding) = source_module
                .imports
                .iter()
                .find(|binding| binding.local_name == *parent)
            {
                if let Some(parent_module) = graph.module(binding.source_module) {
                    if let Some(parent_decl) = parent_module
                        .program
                        .declarations
                        .iter()
                        .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
                    {
                        self.seed_module_graph_import(parent, parent_decl);
                        self.seed_behavior_extends_for_imported_behavior_inner(
                            parent,
                            binding.source_symbol.as_str(),
                            parent_module,
                            graph,
                            seen,
                        );
                    }
                }
            }

            let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
            let parents = self
                .behavior_extends
                .entry(local_name.to_string())
                .or_default();
            if parents
                .iter()
                .any(|existing| existing.key == parent_ref.key)
            {
                continue;
            }

            parents.push(parent_ref);
            self.behavior_extends_spans
                .entry(local_name.to_string())
                .or_insert(*span);
        }
    }

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

            let dependencies = Self::source_module_dependencies(source_module, graph);
            for method in methods {
                self.seed_imported_impl_method(
                    local_name,
                    Some(behavior),
                    behavior_type_args,
                    method,
                    false,
                    &dependencies,
                );
            }
            for default in self.behavior_default_methods_for_impl(
                local_name,
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
