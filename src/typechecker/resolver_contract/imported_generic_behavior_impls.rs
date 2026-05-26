impl TypeChecker {
    fn seed_imported_generic_behavior_impl_template(
        &mut self,
        local_type_name: &str,
        target_type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
    ) {
        if target_type_args.is_empty() {
            return;
        }
        let type_params = named_type_arg_names(target_type_args);
        if type_params.len() != target_type_args.len() {
            return;
        }
        if self.generic_behavior_impls.iter().any(|template| {
            template.type_name == local_type_name
                && template.type_params == type_params
                && template.behavior == behavior
                && template.behavior_type_args == behavior_type_args
        }) {
            return;
        }

        self.generic_behavior_impls
            .push(GenericBehaviorImplTemplate {
                type_name: local_type_name.to_string(),
                type_params,
                behavior: behavior.to_string(),
                behavior_type_args: behavior_type_args.to_vec(),
            });
    }
}
