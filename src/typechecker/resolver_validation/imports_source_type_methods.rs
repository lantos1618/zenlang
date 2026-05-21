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
                    public,
                    ..
                } if type_name == source_name && *public => {
                    Self::insert_source_imported_type_method_dependency(
                        &Self::method_key(local_name, method_name),
                        decl,
                        imported_module,
                        graph,
                        dependencies,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: None,
                    methods,
                    ..
                } if type_name == source_name => {
                    for method in methods {
                        let Declaration::Function { name, public, .. } = method else {
                            continue;
                        };
                        if !*public {
                            continue;
                        }
                        Self::insert_source_imported_type_method_dependency(
                            &Self::method_key(local_name, name),
                            method,
                            imported_module,
                            graph,
                            dependencies,
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
    ) {
        Self::insert_source_method_dependency(
            key,
            decl,
            &mut dependencies.methods,
            &mut dependencies.generic_methods,
        );
        if let Some(template) = dependencies.generic_methods.get_mut(key) {
            let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
            Self::attach_template_dependencies(template, nested_dependencies);
        }
    }
}
