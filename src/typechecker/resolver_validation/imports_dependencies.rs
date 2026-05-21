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

    fn seed_module_graph_import(&mut self, local_name: &str, decl: &Declaration) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    behavior_info_from_ast_methods(local_name.to_string(), type_params, methods),
                );
            }
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.functions.insert(
                    local_name.to_string(),
                    func_info_from_ast_signature(
                        local_name.to_string(),
                        type_params,
                        params,
                        return_type,
                    ),
                );
                if let Some(template) =
                    generic_template_from_type_params(type_params, params, return_type, body, *span)
                {
                    self.generic_functions
                        .insert(local_name.to_string(), template);
                }
            }
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                let key = Self::method_key(type_name, method_name);
                self.methods.insert(
                    key.clone(),
                    func_info_from_ast_signature(key.clone(), type_params, params, return_type),
                );
                if let Some(template) =
                    generic_template_from_type_params(type_params, params, return_type, body, *span)
                {
                    self.generic_methods.insert(key, template);
                }
            }
            _ => {}
        }
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
