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
}
