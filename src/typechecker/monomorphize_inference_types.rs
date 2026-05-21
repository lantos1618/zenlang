use std::collections::HashMap;

use crate::ast::typed::Type;

use super::monomorphize_inference::InferenceConflict;
use super::TypeChecker;

impl TypeChecker {
    pub(super) fn match_generic_type_params(
        &self,
        generic_name: &str,
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        match actual {
            Type::Struct {
                name: actual_name,
                fields: actual_fields,
            } if super::monomorphize::concrete_name_matches_generic(actual_name, generic_name) => {
                if let Some(info) = self.structs.get(generic_name) {
                    for ((_, expected), (_, actual)) in info.fields.iter().zip(actual_fields.iter())
                    {
                        self.match_type_param(expected, actual, type_params, map, conflicts);
                    }
                }
            }
            Type::Enum {
                name: actual_name,
                variants: actual_variants,
            } if super::monomorphize::concrete_name_matches_generic(actual_name, generic_name) => {
                if let Some(info) = self.enums.get(generic_name) {
                    for ((_, expected_payload), (_, actual_payload)) in
                        info.variants.iter().zip(actual_variants.iter())
                    {
                        if let (Some(expected), Some(actual)) = (expected_payload, actual_payload) {
                            self.match_type_param(expected, actual, type_params, map, conflicts);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
