use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

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
        let type_args = self.generic_definition_param_refs(generic_name);
        self.match_generic_type_with_args(
            generic_name,
            &type_args,
            actual,
            type_params,
            map,
            conflicts,
        );
    }

    pub(super) fn match_generic_type_with_args(
        &self,
        generic_name: &str,
        expected_type_args: &[AstType],
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        if self.match_remembered_generic_type_args(
            generic_name,
            expected_type_args,
            actual,
            type_params,
            map,
            conflicts,
        ) {
            return;
        }

        match actual {
            Type::Struct { name, fields }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                self.match_struct_shape(
                    generic_name,
                    expected_type_args,
                    fields,
                    type_params,
                    map,
                    conflicts,
                );
            }
            Type::Enum { name, variants }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                self.match_enum_shape(
                    generic_name,
                    expected_type_args,
                    variants,
                    type_params,
                    map,
                    conflicts,
                );
            }
            _ => {}
        }
    }

    fn match_remembered_generic_type_args(
        &self,
        generic_name: &str,
        expected_type_args: &[AstType],
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) -> bool {
        let actual_name = match actual {
            Type::Struct { name, .. } | Type::Enum { name, .. }
                if self.concrete_type_name_matches_generic(name, generic_name) =>
            {
                name
            }
            _ => return false,
        };
        let Some(actual_type_args) =
            self.remembered_specialized_type_args(actual_name, generic_name)
        else {
            return false;
        };
        if actual_type_args.len() != expected_type_args.len() {
            return false;
        }

        for (expected, actual) in expected_type_args.iter().zip(actual_type_args.iter()) {
            let actual = self.resolve_type(actual);
            self.match_type_param(expected, &actual, type_params, map, conflicts);
        }
        true
    }

    fn match_struct_shape(
        &self,
        generic_name: &str,
        expected_type_args: &[AstType],
        actual_fields: &[(String, Type)],
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        let Some((params, fields)) = self.generic_struct_inference_shape(generic_name) else {
            return;
        };
        let substitutions = ast_type_substitutions(&params, expected_type_args);
        for (expected, (_, actual)) in fields.iter().zip(actual_fields.iter()) {
            let expected = substitute_inference_ast_type(expected, &substitutions);
            self.match_type_param(&expected, actual, type_params, map, conflicts);
        }
    }

    fn match_enum_shape(
        &self,
        generic_name: &str,
        expected_type_args: &[AstType],
        actual_variants: &[(String, Option<Type>)],
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        let Some((params, variants)) = self.generic_enum_inference_shape(generic_name) else {
            return;
        };
        let substitutions = ast_type_substitutions(&params, expected_type_args);
        for (expected_payload, (_, actual_payload)) in variants.iter().zip(actual_variants.iter()) {
            if let (Some(expected), Some(actual)) = (expected_payload, actual_payload) {
                let expected = substitute_inference_ast_type(expected, &substitutions);
                self.match_type_param(&expected, actual, type_params, map, conflicts);
            }
        }
    }

    fn generic_definition_param_refs(&self, generic_name: &str) -> Vec<AstType> {
        self.structs
            .get(generic_name)
            .map(|info| &info.type_params)
            .or_else(|| self.enums.get(generic_name).map(|info| &info.type_params))
            .into_iter()
            .flatten()
            .map(|param| AstType::Named(param.clone()))
            .collect()
    }

    fn generic_struct_inference_shape(
        &self,
        generic_name: &str,
    ) -> Option<(Vec<String>, Vec<AstType>)> {
        self.structs.get(generic_name).map(|info| {
            (
                info.type_params.clone(),
                info.fields.iter().map(|(_, ty)| ty.clone()).collect(),
            )
        })
    }

    fn generic_enum_inference_shape(
        &self,
        generic_name: &str,
    ) -> Option<(Vec<String>, Vec<Option<AstType>>)> {
        self.enums.get(generic_name).map(|info| {
            (
                info.type_params.clone(),
                info.variants
                    .iter()
                    .map(|(_, payload)| payload.clone())
                    .collect(),
            )
        })
    }
}

fn ast_type_substitutions(params: &[String], args: &[AstType]) -> HashMap<String, AstType> {
    params.iter().cloned().zip(args.iter().cloned()).collect()
}

fn substitute_inference_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_inference_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_inference_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_inference_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_inference_ast_type(arg, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}
