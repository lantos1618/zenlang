use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::ast_type_substitution::substitute_ast_type_names;
use super::monomorphize_inference::InferenceConflict;
use super::TypeChecker;

impl TypeChecker {
    pub(super) fn match_generic_type_with_args(
        &self,
        generic_name: &str,
        expected_type_args: &[AstType],
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        let actual_name = match actual {
            Type::Struct { name, .. } | Type::Enum { name, .. }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                Some(name)
            }
            _ => None,
        };
        if let Some((source_name, actual_type_args)) = actual_name.and_then(|name| {
            self.specialized_type_generic_names
                .get(name)
                .zip(self.specialized_type_args.get(name))
        }) {
            if source_name == generic_name && actual_type_args.len() == expected_type_args.len() {
                for (expected, actual) in expected_type_args.iter().zip(actual_type_args.iter()) {
                    let actual = self.resolve_type(actual);
                    self.match_type_param(expected, &actual, type_params, map, conflicts);
                }
                return;
            }
        }

        match actual {
            Type::Struct { name, fields }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                let Some(info) = self.structs.get(generic_name) else {
                    return;
                };
                self.match_inference_shape_items(
                    &info.type_params,
                    expected_type_args,
                    info.fields
                        .iter()
                        .zip(fields.iter())
                        .map(|((_, expected), (_, actual))| (expected, actual)),
                    type_params,
                    map,
                    conflicts,
                );
            }
            Type::Enum { name, variants }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                let Some(info) = self.enums.get(generic_name) else {
                    return;
                };
                self.match_inference_shape_items(
                    &info.type_params,
                    expected_type_args,
                    info.variants.iter().zip(variants.iter()).filter_map(
                        |((_, expected_payload), (_, actual_payload))| {
                            Some((expected_payload.as_ref()?, actual_payload.as_ref()?))
                        },
                    ),
                    type_params,
                    map,
                    conflicts,
                );
            }
            _ => {}
        }
    }

    fn match_inference_shape_items<'a>(
        &self,
        shape_params: &[String],
        expected_type_args: &[AstType],
        items: impl IntoIterator<Item = (&'a AstType, &'a Type)>,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        let substitutions: HashMap<String, AstType> = shape_params
            .iter()
            .cloned()
            .zip(expected_type_args.iter().cloned())
            .collect();
        for (expected, actual) in items {
            let expected =
                substitute_ast_type_names(expected, &|name| substitutions.get(name).cloned());
            self.match_type_param(&expected, actual, type_params, map, conflicts);
        }
    }
}
