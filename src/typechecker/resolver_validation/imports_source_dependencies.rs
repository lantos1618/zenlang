impl TypeChecker {
    fn source_module_dependencies(
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> SourceModuleDependencies {
        let source_scope = Self::source_module_specialization_scope(source_module);
        let mut dependencies = SourceModuleDependencies {
            specialization_scope: Some(source_scope.clone()),
            ..SourceModuleDependencies::default()
        };
        for binding in &source_module.imports {
            let Some(imported_module) = graph.module(binding.source_module) else {
                continue;
            };
            let Some(decl) = imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                continue;
            };
            let imported_scope = Self::source_module_specialization_scope(imported_module);
            Self::insert_source_import_dependency(
                &binding.local_name,
                decl,
                &mut dependencies,
                Some(&imported_scope),
            );
            if matches!(decl, Declaration::Struct { .. } | Declaration::Enum { .. }) {
                Self::insert_source_import_type_method_dependencies(
                    &binding.local_name,
                    binding.source_symbol.as_str(),
                    imported_module,
                    graph,
                    &mut dependencies,
                );
            } else if matches!(
                decl,
                Declaration::Function { type_params, .. } if !type_params.is_empty()
            ) {
                let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
                if let Some(template) = dependencies
                    .generic_functions
                    .get_mut(binding.local_name.as_str())
                {
                    Self::attach_template_dependencies(template, nested_dependencies);
                }
            }
        }

        for decl in &source_module.program.declarations {
            match decl {
                Declaration::Struct { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Enum { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Function { name, .. } => {
                    Self::insert_source_function_dependency(
                        name,
                        decl,
                        &mut dependencies.functions,
                        &mut dependencies.generic_functions,
                        Some(&source_scope),
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } => {
                    Self::insert_source_method_dependency(
                        &Self::method_key(type_name, method_name),
                        decl,
                        &mut dependencies.methods,
                        &mut dependencies.generic_methods,
                        Some(&source_scope),
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Declaration::Function { name, .. } = method {
                            Self::insert_source_method_dependency(
                                &Self::behavior_impl_method_key(
                                    type_name,
                                    name,
                                    behavior.as_deref(),
                                    behavior_type_args,
                                ),
                                method,
                                &mut dependencies.methods,
                                &mut dependencies.generic_methods,
                                Some(&source_scope),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        dependencies
    }

    fn source_module_specialization_scope(source_module: &ResolvedModule) -> String {
        source_module.info.canonical_path.clone()
    }

    fn insert_source_import_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
        specialization_scope: Option<&str>,
    ) {
        match decl {
            Declaration::Struct { .. } | Declaration::Enum { .. } => {
                Self::insert_source_type_dependency(local_name, decl, dependencies);
            }
            Declaration::Function { .. } => {
                Self::insert_source_function_dependency(
                    local_name,
                    decl,
                    &mut dependencies.functions,
                    &mut dependencies.generic_functions,
                    specialization_scope,
                );
            }
            _ => {}
        }
    }

    fn insert_source_type_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                dependencies.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                dependencies.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
                );
            }
            _ => {}
        }
    }

}
