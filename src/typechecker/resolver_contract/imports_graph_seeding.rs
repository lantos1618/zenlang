impl TypeChecker {
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
}
