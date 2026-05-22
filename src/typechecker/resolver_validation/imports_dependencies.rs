impl TypeChecker {
    fn seed_imported_callable_signature_type_dependencies(
        &mut self,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let mut type_names = HashSet::new();
        match decl {
            Declaration::Function {
                params,
                return_type,
                ..
            }
            | Declaration::Method {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    collect_ast_type_names(&param.ty, &mut type_names);
                }
                if let Some(return_type) = return_type {
                    collect_ast_type_names(return_type, &mut type_names);
                }
            }
            _ => return,
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

    fn seed_imported_generic_function_dependencies(
        &mut self,
        local_name: &str,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Declaration::Function { type_params, .. } = decl else {
            return;
        };
        if type_params.is_empty() {
            return;
        }
        let dependencies = Self::source_module_dependencies(source_module, graph);
        let Some(template) = self.generic_functions.get_mut(local_name) else {
            return;
        };
        Self::attach_template_dependencies(template, dependencies);
    }

    fn attach_template_dependencies(
        template: &mut GenericFunctionTemplate,
        dependencies: SourceModuleDependencies,
    ) {
        template.attach_source_dependencies(dependencies);
    }

    fn seed_public_methods_for_imported_type(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let dependencies = Self::source_module_dependencies(source_module, graph);

        for decl in &source_module.program.declarations {
            let Declaration::Method {
                type_name: method_type,
                public,
                ..
            } = decl
            else {
                continue;
            };

            if method_type == type_name && *public {
                self.seed_imported_method_with_dependencies(type_name, decl, &dependencies);
            }
        }
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name: impl_type,
                behavior: None,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if impl_type != type_name {
                continue;
            }
            for method in methods {
                self.seed_imported_impl_method(type_name, None, &[], method, true, &dependencies);
            }
        }
    }

}
