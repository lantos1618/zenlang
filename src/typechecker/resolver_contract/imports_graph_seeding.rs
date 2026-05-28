impl TypeChecker {
    pub(in crate::typechecker) fn seed_declaration_info(
        &mut self,
        local_name: &str,
        decl: &Declaration,
    ) {
        if let Some(callable) = decl.as_callable() {
            let key = match decl {
                Declaration::Method { type_name, .. } => method_signature_key(type_name, callable.name),
                _ => local_name.to_string(),
            };
            if matches!(decl, Declaration::Method { .. }) {
                insert_callable_signature(key, decl, &mut self.methods, &mut self.generic_methods);
            } else {
                insert_callable_signature(key, decl, &mut self.functions, &mut self.generic_functions);
            }
            return;
        }

        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(type_params, variants),
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    BehaviorInfo {
                        type_params: type_param_names(type_params),
                        type_param_bounds: type_param_bounds(type_params),
                        methods: methods.to_vec(),
                    },
                );
            }
            _ => {}
        }
    }
}
