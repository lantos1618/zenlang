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
