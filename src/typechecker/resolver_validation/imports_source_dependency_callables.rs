impl TypeChecker {
    fn insert_source_import_type_method_dependencies(
        local_name: &str,
        source_name: &str,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        let specialization_scope = Self::source_module_specialization_scope(imported_module);
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
                        &specialization_scope,
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
                            &specialization_scope,
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
        Self::insert_source_method_dependency(
            key,
            decl,
            &mut dependencies.methods,
            &mut dependencies.generic_methods,
            Some(specialization_scope),
        );
        if let Some(template) = dependencies.generic_methods.get_mut(key) {
            let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
            Self::attach_template_dependencies(template, nested_dependencies);
        }
    }

    fn insert_source_function_dependency(
        key: &str,
        decl: &Declaration,
        functions: &mut HashMap<String, FuncInfo>,
        generic_functions: &mut HashMap<String, GenericFunctionTemplate>,
        specialization_scope: Option<&str>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl) {
            Self::insert_source_callable_dependency(
                signature,
                functions,
                generic_functions,
                specialization_scope,
            );
        }
    }

    fn insert_source_method_dependency(
        key: &str,
        decl: &Declaration,
        methods: &mut HashMap<String, FuncInfo>,
        generic_methods: &mut HashMap<String, GenericFunctionTemplate>,
        specialization_scope: Option<&str>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl)
            .or_else(|| ImportedMethodSignature::from_method_declaration(key, decl))
        {
            Self::insert_source_callable_dependency(
                signature,
                methods,
                generic_methods,
                specialization_scope,
            );
        }
    }

    fn insert_source_callable_dependency(
        signature: ImportedMethodSignature<'_>,
        callables: &mut HashMap<String, FuncInfo>,
        generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
        specialization_scope: Option<&str>,
    ) {
        callables.insert(
            signature.name.to_string(),
            signature.func_info(signature.name.to_string()),
        );
        if let Some(template) = signature.generic_template() {
            let template = if let Some(scope) = specialization_scope {
                template.with_specialization_scope(scope.to_string())
            } else {
                template
            };
            generic_callables.insert(signature.name.to_string(), template);
        }
    }
}
