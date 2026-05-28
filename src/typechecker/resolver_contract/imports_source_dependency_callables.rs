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
