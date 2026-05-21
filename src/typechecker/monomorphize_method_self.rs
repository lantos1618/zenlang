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
        if let Some(info) = self.structs.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        if let Some(info) = self.enums.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        Some(self.resolve_type(&AstType::Named(receiver_name.to_string())))
    }

    fn generic_receiver_self_type(
        &mut self,
        receiver_name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        if type_params.is_empty() {
            return self.resolve_type(&AstType::Named(receiver_name.to_string()));
        }

        let type_args: Vec<AstType> = type_params
            .iter()
            .filter_map(|param| substitutions.get(param).map(|ty| self.type_to_ast_ref(ty)))
            .collect();
        if type_args.len() == type_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        }
    }
}
