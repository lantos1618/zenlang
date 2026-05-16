use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::TypeChecker;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InferenceConflict {
    pub(crate) param: String,
    pub(crate) inferred: Type,
    pub(crate) actual: Type,
}

impl TypeChecker {
    /// Infer type arguments for a generic function by matching actual arg types
    /// against parameter types containing type params.
    #[cfg(test)]
    pub(crate) fn infer_type_args(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> HashMap<String, Type> {
        self.infer_type_args_with_conflicts(type_params, param_types, arg_types)
            .0
    }

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
        let (mut map, mut conflicts) =
            self.infer_type_args_with_conflicts(type_params, param_types, arg_types);
        if let (Some(receiver_name), Some(receiver_ty)) = (
            super::method_signature_receiver_name(method_name),
            arg_types.first(),
        ) {
            self.match_generic_type_params(
                receiver_name,
                receiver_ty,
                type_params,
                &mut map,
                &mut conflicts,
            );
        }
        (map, conflicts)
    }

    fn match_type_param(
        &self,
        param: &AstType,
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        match param {
            AstType::Named(name) if type_params.contains(name) => {
                self.set_inferred_type_param(name, actual, map, conflicts);
            }
            AstType::Ptr(inner) => {
                if let Type::Ptr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::MutPtr(inner) => {
                if let Type::MutPtr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::RawPtr(inner) => {
                if let Type::RawPtr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::Slice(inner) => {
                if let Type::Slice(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::Array { elem, .. } => {
                if let Type::Array {
                    elem: actual_elem, ..
                } = actual
                {
                    self.match_type_param(elem, actual_elem, type_params, map, conflicts);
                }
            }
            AstType::Function { params, ret } => {
                if let Type::Function {
                    params: actual_params,
                    ret: actual_ret,
                } = actual
                {
                    for (param, actual_param) in params.iter().zip(actual_params.iter()) {
                        self.match_type_param(param, actual_param, type_params, map, conflicts);
                    }
                    self.match_type_param(ret, actual_ret, type_params, map, conflicts);
                }
            }
            AstType::Generic { name, .. } => {
                self.match_generic_type_params(name, actual, type_params, map, conflicts);
            }
            _ => {}
        }
    }

    fn set_inferred_type_param(
        &self,
        name: &str,
        actual: &Type,
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        if let Some(inferred) = map.get(name) {
            if !self.types_compatible(inferred, actual) {
                conflicts.push(InferenceConflict {
                    param: name.to_string(),
                    inferred: inferred.clone(),
                    actual: actual.clone(),
                });
            }
            return;
        }

        map.insert(name.to_string(), actual.clone());
    }

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
