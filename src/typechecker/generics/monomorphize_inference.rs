use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

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
            | (AstType::Future(inner), Type::Future(actual_inner))
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
