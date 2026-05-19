//! Monomorphization naming helpers for generic type and callable instances.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::type_mangle_key;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn mangle_generic_type_name(&self, name: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            return name.to_string();
        }
        let suffix: Vec<String> = type_args
            .iter()
            .map(|arg| type_mangle_key(&self.resolve_type(arg)))
            .collect();
        format!("{}_{}", name, suffix.join("_"))
    }

    pub(crate) fn generic_function_mangled_name(
        &self,
        name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> String {
        let suffix: Vec<String> = type_params
            .iter()
            .filter_map(|param| substitutions.get(param).map(type_mangle_key))
            .collect();
        if suffix.is_empty() {
            name.to_string()
        } else {
            format!("{}_{}", name, suffix.join("_"))
        }
    }
}
