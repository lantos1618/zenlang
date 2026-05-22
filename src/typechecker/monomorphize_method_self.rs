use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn generic_method_self_type(
        &mut self,
        method_name: &str,
        substitutions: &HashMap<String, Type>,
    ) -> Option<Type> {
        let receiver_name = super::method_signature_receiver_name(method_name)?;
        let method_params = self
            .generic_methods
            .get(method_name)
            .map(|template| template.type_params.clone())
            .unwrap_or_default();

        if let Some(info) = self.structs.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                &method_params,
                substitutions,
            ));
        }
        if let Some(info) = self.enums.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                &method_params,
                substitutions,
            ));
        }
        Some(self.resolve_type(&AstType::Named(receiver_name.to_string())))
    }

    fn generic_receiver_self_type(
        &mut self,
        receiver_name: &str,
        receiver_params: &[String],
        method_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        if receiver_params.is_empty() {
            return self.resolve_type(&AstType::Named(receiver_name.to_string()));
        }

        let type_args: Vec<AstType> = receiver_params
            .iter()
            .enumerate()
            .filter_map(|(idx, param)| {
                let key = receiver_substitution_key(param, idx, method_params, substitutions)?;
                substitutions.get(key).map(|ty| self.type_to_ast_ref(ty))
            })
            .collect();

        if type_args.len() == receiver_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        }
    }
}

fn receiver_substitution_key<'a>(
    receiver_param: &'a str,
    index: usize,
    method_params: &'a [String],
    substitutions: &HashMap<String, Type>,
) -> Option<&'a str> {
    if substitutions.contains_key(receiver_param) {
        Some(receiver_param)
    } else {
        method_params.get(index).map(String::as_str)
    }
}
