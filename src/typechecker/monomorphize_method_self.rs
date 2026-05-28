use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn generic_method_self_type(
        &self,
        method_name: &str,
        substitutions: &HashMap<String, Type>,
    ) -> Option<Type> {
        let receiver_name = super::method_signature_receiver_name(method_name)?;
        let method_params = self
            .generic_methods
            .get(method_name)
            .map(|template| template.type_params.clone())
            .unwrap_or_default();

        let Some(receiver_params) = self
            .type_params_for_type(receiver_name)
            .filter(|params| !params.is_empty())
        else {
            return Some(self.resolve_type(&AstType::Named(receiver_name.to_string())));
        };

        let type_args: Vec<AstType> = receiver_params
            .iter()
            .enumerate()
            .filter_map(|(idx, param)| {
                let key = if substitutions.contains_key(param) {
                    param
                } else {
                    method_params.get(idx)?
                };
                substitutions.get(key).map(|ty| self.type_to_ast_ref(ty))
            })
            .collect();

        Some(if type_args.len() == receiver_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        })
    }
}
