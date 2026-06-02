impl TypeChecker {
    fn source_module_dependencies(
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> SourceModuleDependencies {
        let mut dependencies = SourceModuleDependencies {
            specialization_scope: Some(source_module.info.canonical_path.clone()),
            ..SourceModuleDependencies::default()
        };
        for binding in &source_module.imports {
            let Some(imported_module) = graph.module(binding.source_module) else {
                continue;
            };
            let Some(decl) = module_declaration(imported_module, binding.source_symbol.as_str())
            else {
                continue;
            };
            match decl {
                Declaration::Struct { .. } | Declaration::Enum { .. } => {
                    Self::insert_source_type_dependency(
                        &binding.local_name,
                        decl,
                        &mut dependencies,
                        Some(&imported_module.info.canonical_path),
                    );
                    Self::insert_source_import_type_method_dependencies(
                        &binding.local_name,
                        binding.source_symbol.as_str(),
                        imported_module,
                        graph,
                        &mut dependencies,
                    );
                }
                Declaration::Function { .. } => {
                    insert_callable_signature_scoped(
                        &binding.local_name,
                        decl,
                        &mut dependencies.functions,
                        &mut dependencies.generic_functions,
                        Some(&imported_module.info.canonical_path),
                    );
                    if let Some(template) =
                        dependencies.generic_functions.get_mut(binding.local_name.as_str())
                    {
                        template.dependencies =
                            Self::source_module_dependencies(imported_module, graph);
                    }
                }
                _ => {}
            }
        }

        for decl in &source_module.program.declarations {
            match decl {
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } => insert_callable_signature_scoped(
                    &method_signature_key(type_name, method_name),
                    decl,
                    &mut dependencies.methods,
                    &mut dependencies.generic_methods,
                    Some(&source_module.info.canonical_path),
                ),
                Declaration::Function { name, .. } => insert_callable_signature_scoped(
                    name,
                    decl,
                    &mut dependencies.functions,
                    &mut dependencies.generic_functions,
                    Some(&source_module.info.canonical_path),
                ),
                Declaration::Struct { name, .. } | Declaration::Enum { name, .. } => {
                    Self::insert_source_type_dependency(
                        name,
                        decl,
                        &mut dependencies,
                        Some(&source_module.info.canonical_path),
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    type_args,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Some(method_decl) = method.as_callable() {
                            insert_callable_signature_scoped(
                                &behavior_impl_method_signature_key_with_target_args(
                                    type_name,
                                    method_decl.name,
                                    behavior.as_deref(),
                                    behavior_type_args,
                                    type_args,
                                ),
                                method,
                                &mut dependencies.methods,
                                &mut dependencies.generic_methods,
                                Some(&source_module.info.canonical_path),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        dependencies
    }

}

impl TypeChecker {
    fn insert_source_type_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
        specialization_scope: Option<&str>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                public,
                ..
            } => {
                let mut info = struct_info_from_ast_fields(type_params, fields);
                if !*public {
                    info.specialization_scope = specialization_scope.map(str::to_string);
                }
                dependencies.structs.insert(local_name.to_string(), info);
            }
            Declaration::Enum {
                type_params,
                variants,
                public,
                ..
            } => {
                let mut info = enum_info_from_ast_variants(type_params, variants);
                if !*public {
                    info.specialization_scope = specialization_scope.map(str::to_string);
                }
                dependencies.enums.insert(local_name.to_string(), info);
            }
            _ => {}
        }
    }
}

impl TypeChecker {
    fn insert_source_import_type_method_dependencies(
        local_name: &str,
        source_name: &str,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        for decl in &imported_module.program.declarations {
            match decl {
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } if type_name == source_name => {
                    Self::insert_source_imported_type_method_dependency(
                        &method_signature_key(local_name, method_name),
                        decl,
                        imported_module,
                        graph,
                        dependencies,
                        &imported_module.info.canonical_path,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: None,
                    methods,
                    ..
                } if type_name == source_name => {
                    for method in methods {
                        let Some(name) = method.name() else {
                            continue;
                        };
                        Self::insert_source_imported_type_method_dependency(
                            &method_signature_key(local_name, name),
                            method,
                            imported_module,
                            graph,
                            dependencies,
                            &imported_module.info.canonical_path,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn insert_source_imported_type_method_dependency(
        key: &str,
        decl: &Declaration,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
        specialization_scope: &str,
    ) {
        if !decl.is_public() {
            return;
        }

        insert_callable_signature_scoped(
            key,
            decl,
            &mut dependencies.methods,
            &mut dependencies.generic_methods,
            Some(specialization_scope),
        );
        if let Some(template) = dependencies.generic_methods.get_mut(key) {
            template.dependencies = Self::source_module_dependencies(imported_module, graph);
        }
    }

}
