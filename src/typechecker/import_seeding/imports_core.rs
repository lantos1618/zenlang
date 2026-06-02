impl TypeChecker {
    pub(super) fn collect_module_graph_imports(
        &mut self,
        graph: &ResolvedModuleGraph,
        entry: &ResolvedModule,
    ) {
        for binding in &entry.imports {
            let Some(source_module) = graph.module(binding.source_module) else {
                self.push_error(
                    E0232,
                    format!(
                        "module graph import '{}' points at missing module {:?}",
                        binding.local_name, binding.source_module
                    ),
                    binding.span,
                );
                continue;
            };

            let Some(decl) = module_declaration(source_module, binding.source_symbol.as_str())
            else {
                self.push_error(
                    E0232,
                    format!(
                        "module graph import '{}' points at missing symbol '{}'",
                        binding.local_name, binding.source_symbol
                    ),
                    binding.span,
                );
                continue;
            };

            self.seed_declaration_info(binding.local_name.as_str(), decl);
            self.seed_imported_callable_signature_type_dependencies(decl, source_module, graph);
            if let Some(template) = self.generic_functions.get_mut(binding.local_name.as_str()) {
                template.dependencies = Self::source_module_dependencies(source_module, graph);
            }
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

        self.seed_transitive_behavior_impls(entry, graph);
    }
}

impl TypeChecker {
    pub(in crate::typechecker) fn seed_declaration_info(
        &mut self,
        local_name: &str,
        decl: &Declaration,
    ) {
        if let Some(callable) = decl.as_callable() {
            let key = match decl {
                Declaration::Method { type_name, .. } => method_signature_key(type_name, callable.name),
                _ => local_name.to_string(),
            };
            if matches!(decl, Declaration::Method { .. }) {
                insert_callable_signature(key, decl, &mut self.methods, &mut self.generic_methods);
            } else {
                insert_callable_signature(key, decl, &mut self.functions, &mut self.generic_functions);
            }
            return;
        }

        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(type_params, variants),
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    BehaviorInfo {
                        type_params: type_param_names(type_params),
                        type_param_bounds: type_param_bounds(type_params),
                        methods: methods.to_vec(),
                    },
                );
            }
            _ => {}
        }
    }
}

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

impl TypeChecker {
    fn seed_public_imported_method_signature(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        self.seed_imported_method_signature(local_type_name, None, &[], &[], method, dependencies);
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        target_type_args: &[AstType],
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        if behavior.is_none() && !method.is_public() {
            return;
        }

        let Some((method_name, info, template)) = callable_signature_from_declaration(method)
        else {
            return;
        };
        let key = behavior_impl_method_signature_key_with_target_args(
            local_type_name,
            method_name,
            behavior,
            behavior_type_args,
            target_type_args,
        );
        self.methods.insert(key.clone(), info);
        if let Some(mut template) = template {
            template.dependencies = dependencies.clone();
            self.generic_methods.insert(key, template);
        }
    }
}
