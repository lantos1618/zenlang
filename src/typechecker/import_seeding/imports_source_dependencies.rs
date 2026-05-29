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
