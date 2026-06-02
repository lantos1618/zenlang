use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::super::ast_type_substitution::substitute_ast_type_names;
use super::super::TypeChecker;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InferenceConflict {
    pub(crate) param: String,
    pub(crate) inferred: Type,
    pub(crate) actual: Type,
}

impl TypeChecker {
    pub(crate) fn infer_type_args_with_conflicts(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> (HashMap<String, Type>, Vec<InferenceConflict>) {
        let mut map = HashMap::new();
        let mut conflicts = Vec::new();
        for ((_name, param_ty), arg_ty) in param_types.iter().zip(arg_types.iter()) {
            self.match_type_param(param_ty, arg_ty, type_params, &mut map, &mut conflicts);
        }
        (map, conflicts)
    }

    pub(crate) fn infer_method_type_args(
        &self,
        method_name: &str,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> (HashMap<String, Type>, Vec<InferenceConflict>) {
        let mut map = HashMap::new();
        let mut conflicts = Vec::new();
        if param_types
            .first()
            .is_some_and(|(_, ty)| matches!(ty, AstType::SelfType))
        {
            if let (Some(receiver_name), Some(receiver_ty)) = (
                super::super::method_signature_receiver_name(method_name),
                arg_types.first(),
            ) {
                let receiver_type_args: Vec<_> = self
                    .type_params_for_type(receiver_name)
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .filter_map(|(idx, param)| {
                        let method_param = if type_params.contains(param) {
                            param
                        } else {
                            type_params.get(idx)?
                        };
                        Some(AstType::Named(method_param.clone()))
                    })
                    .collect();
                self.match_generic_type_with_args(
                    receiver_name,
                    &receiver_type_args,
                    receiver_ty,
                    type_params,
                    &mut map,
                    &mut conflicts,
                );
            }
        }
        for ((_name, param_ty), arg_ty) in param_types.iter().zip(arg_types.iter()) {
            self.match_type_param(param_ty, arg_ty, type_params, &mut map, &mut conflicts);
        }
        (map, conflicts)
    }

    pub(super) fn match_type_param(
        &self,
        param: &AstType,
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        match (param, actual) {
            (AstType::Named(name), _) if type_params.contains(name) => {
                if let Some(inferred) = map.get(name) {
                    if !self.types_compatible(inferred, actual) {
                        conflicts.push(InferenceConflict {
                            param: name.to_string(),
                            inferred: inferred.clone(),
                            actual: actual.clone(),
                        });
                    }
                } else {
                    map.insert(name.to_string(), actual.clone());
                }
            }
            (AstType::Ptr(inner), Type::Ptr(actual_inner))
            | (AstType::MutPtr(inner), Type::MutPtr(actual_inner))
            | (AstType::RawPtr(inner), Type::RawPtr(actual_inner))
            | (AstType::Slice(inner), Type::Slice(actual_inner)) => {
                self.match_type_param(inner, actual_inner, type_params, map, conflicts);
            }
            (
                AstType::Array { elem, .. },
                Type::Array {
                    elem: actual_elem, ..
                },
            ) => {
                self.match_type_param(elem, actual_elem, type_params, map, conflicts);
            }
            (
                AstType::Function { params, ret },
                Type::Function {
                    params: actual_params,
                    ret: actual_ret,
                },
            ) => {
                for (param, actual_param) in params.iter().zip(actual_params.iter()) {
                    self.match_type_param(param, actual_param, type_params, map, conflicts);
                }
                self.match_type_param(ret, actual_ret, type_params, map, conflicts);
            }
            (AstType::Generic { name, type_args }, _) => {
                self.match_generic_type_with_args(
                    name,
                    type_args,
                    actual,
                    type_params,
                    map,
                    conflicts,
                );
            }
            _ => {}
        }
    }
}
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
