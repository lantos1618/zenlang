struct ImportedImplMethodSeed<'a> {
    local_type_name: &'a str,
    behavior: Option<&'a str>,
    behavior_type_args: &'a [AstType],
    target_type_args: &'a [AstType],
    method: &'a Declaration,
    public_only: bool,
    dependencies: &'a SourceModuleDependencies,
}

impl TypeChecker {
    fn seed_imported_method_with_dependencies(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Method { method_name, .. } = method else {
            return;
        };
        let Some(signature) = ImportedMethodSignature::from_method_declaration(method_name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(local_type_name, None, &[], &[], signature, dependencies);
    }

    fn seed_imported_impl_method(&mut self, seed: ImportedImplMethodSeed<'_>) {
        let ImportedImplMethodSeed {
            local_type_name,
            behavior,
            behavior_type_args,
            target_type_args,
            method,
            public_only,
            dependencies,
        } = seed;
        let Declaration::Function { name, public, .. } = method else {
            return;
        };
        if public_only && !*public {
            return;
        }
        let Some(signature) = ImportedMethodSignature::from_function_declaration(name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(
            local_type_name,
            behavior,
            behavior_type_args,
            target_type_args,
            signature,
            dependencies,
        );
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        target_type_args: &[AstType],
        signature: ImportedMethodSignature<'_>,
        dependencies: &SourceModuleDependencies,
    ) {
        let key = Self::behavior_impl_method_key_with_target_args(
            local_type_name,
            signature.name,
            behavior,
            behavior_type_args,
            target_type_args,
        );
        self.methods
            .insert(key.clone(), signature.func_info(key.clone()));
        if let Some(template) = signature.generic_template() {
            let template = if let Some(scope) = &dependencies.specialization_scope {
                template.with_specialization_scope(scope.clone())
            } else {
                template
            };
            self.generic_methods
                .insert(key, dependencies.apply_to_template(template));
        }
    }
}
