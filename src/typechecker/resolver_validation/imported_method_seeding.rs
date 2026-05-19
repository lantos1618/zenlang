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

        self.seed_imported_method_signature(local_type_name, None, &[], signature, dependencies);
    }

    fn seed_imported_impl_method(
        &mut self,
        local_type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        method: &Declaration,
        public_only: bool,
        dependencies: &SourceModuleDependencies,
    ) {
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
            signature,
            dependencies,
        );
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        signature: ImportedMethodSignature<'_>,
        dependencies: &SourceModuleDependencies,
    ) {
        let key = Self::behavior_impl_method_key(
            local_type_name,
            signature.name,
            behavior,
            behavior_type_args,
        );
        self.methods
            .insert(key.clone(), signature.func_info(key.clone()));
        if let Some(template) = signature.generic_template() {
            self.generic_methods
                .insert(key, dependencies.apply_to_template(template));
        }
    }
}
