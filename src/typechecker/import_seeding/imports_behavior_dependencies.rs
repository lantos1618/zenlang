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
            if let Some((source_symbol, imported_module)) =
                Self::imported_behavior_binding_target(behavior, source_module, graph)
            {
                self.seed_behavior_decl_for_imported_impl(
                    behavior,
                    source_symbol,
                    imported_module,
                    graph,
                );
            }

            let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
            self.behavior_impls
                .insert((local_name.to_string(), behavior_ref.key.clone()));
            self.behavior_refs_by_key
                .insert(behavior_ref.key.clone(), behavior_ref);
            if !type_args.is_empty() {
                let type_params = named_type_arg_names(type_args);
                if type_params.len() == type_args.len()
                    && !self.generic_behavior_impls.iter().any(|template| {
                        template.type_name == local_name
                            && template.type_params == type_params
                            && template.behavior == behavior.as_str()
                            && template.behavior_type_args.as_slice() == behavior_type_args.as_slice()
                    })
                {
                    self.generic_behavior_impls
                        .push(GenericBehaviorImplTemplate {
                            type_name: local_name.to_string(),
                            type_params,
                            behavior: behavior.to_string(),
                            behavior_type_args: behavior_type_args.to_vec(),
                        });
                }
            }

            let dependencies = Self::source_module_dependencies(source_module, graph);
            for method in methods {
                self.seed_imported_method_signature(
                    local_name,
                    Some(behavior),
                    behavior_type_args,
                    type_args,
                    method,
                    &dependencies,
                );
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
        matches!(
            module_declaration(source_module, behavior),
            Some(Declaration::Behavior { public: true, .. })
        ) || Self::imported_behavior_binding_target(behavior, source_module, graph).is_some_and(
            |(source_symbol, imported_module)| {
                matches!(
                    module_declaration(imported_module, source_symbol),
                    Some(Declaration::Behavior { public: true, .. })
                )
            },
        )
    }

    fn imported_behavior_binding_target<'a>(
        behavior: &str,
        source_module: &'a ResolvedModule,
        graph: &'a ResolvedModuleGraph,
    ) -> Option<(&'a str, &'a ResolvedModule)> {
        let binding = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)?;
        let imported_module = graph.module(binding.source_module)?;
        Some((binding.source_symbol.as_str(), imported_module))
    }

    fn seed_behavior_decl_for_imported_impl(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(behavior_decl) = module_declaration(source_module, source_name) {
            self.seed_declaration_info(local_name, behavior_decl);
            self.seed_behavior_extends_for_imported_behavior(
                local_name,
                source_name,
                source_module,
                graph,
            );
        }
    }

}
