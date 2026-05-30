impl TypeChecker {
    fn seed_behavior_impls_for_imported_type(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        for decl in &source_module.program.declarations {
            if let Declaration::ImplBlock { type_name, .. } = decl {
                if type_name == source_name {
                    self.seed_imported_behavior_impl_block(local_name, decl, source_module, graph);
                }
            }
        }
    }

    // Seed every public behavior impl reachable through the module being checked,
    // following imports transitively. A bound like `Alloc: Allocator` can be
    // satisfied by an impl (e.g. `Mallocator.implements(Allocator)`) that the
    // entry file never imports directly but reaches through the collection it
    // does import — so the impl must be visible wherever its type is reachable,
    // not only when the entry imports it by name.
    pub(super) fn seed_transitive_behavior_impls(
        &mut self,
        entry: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let mut seen = HashSet::new();
        let mut queue: Vec<_> = entry.imports.iter().map(|b| b.source_module).collect();
        while let Some(id) = queue.pop() {
            let newly_seen = seen.insert(id);
            let Some(module) = graph.module(id).filter(|_| newly_seen) else {
                continue;
            };
            for binding in &module.imports {
                queue.push(binding.source_module);
            }
            for decl in &module.program.declarations {
                if let Declaration::ImplBlock { type_name, .. } = decl {
                    // Within its defining module a type's local name is its declared
                    // (canonical) name, which is how monomorphization refers to it.
                    self.seed_imported_behavior_impl_block(type_name, decl, module, graph);
                }
            }
        }
    }

    fn seed_imported_behavior_impl_block(
        &mut self,
        local_name: &str,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Declaration::ImplBlock {
            type_args,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            ..
        } = decl
        else {
            return;
        };
        if !self.imported_behavior_impl_is_public(behavior, source_module, graph) {
            return;
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

        self.seed_generic_behavior_impl_template(local_name, behavior, behavior_type_args, type_args);

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

    fn seed_generic_behavior_impl_template(
        &mut self,
        local_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        type_args: &[AstType],
    ) {
        if type_args.is_empty() {
            return;
        }
        let type_params = named_type_arg_names(type_args);
        if type_params.len() != type_args.len() {
            return;
        }
        let already_seeded = self.generic_behavior_impls.iter().any(|template| {
            template.type_name == local_name
                && template.type_params == type_params
                && template.behavior == behavior
                && template.behavior_type_args.as_slice() == behavior_type_args
        });
        if already_seeded {
            return;
        }
        self.generic_behavior_impls.push(GenericBehaviorImplTemplate {
            type_name: local_name.to_string(),
            type_params,
            behavior: behavior.to_string(),
            behavior_type_args: behavior_type_args.to_vec(),
        });
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
