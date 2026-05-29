use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::type_to_ast;
use super::super::TypeChecker;

impl TypeChecker {
    pub(crate) fn generic_type_args_from_type(
        &self,
        concrete_name: &str,
        ty: &Type,
    ) -> Option<(String, Vec<AstType>)> {
        if let (Some(generic_name), Some(type_args)) = (
            self.specialized_type_generic_names.get(concrete_name),
            self.specialized_type_args.get(concrete_name),
        ) {
            return Some((generic_name.clone(), type_args.clone()));
        }

        let mut candidates = self
            .structs
            .iter()
            .map(|(name, info)| (0u8, name.as_str(), info.type_params.as_slice()))
            .chain(
                self.enums
                    .iter()
                    .map(|(name, info)| (1u8, name.as_str(), info.type_params.as_slice())),
            )
            .filter(|(_, name, params)| {
                concrete_name != *name
                    && self.concrete_type_name_matches_generic(concrete_name, name)
                    && !params.is_empty()
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
        for (_, name, params) in candidates {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            let expected_type_args: Vec<_> = params
                .iter()
                .map(|param| AstType::Named(param.clone()))
                .collect();
            self.match_generic_type_with_args(
                name,
                &expected_type_args,
                ty,
                params,
                &mut inferred,
                &mut conflicts,
            );
            let type_args: Vec<_> = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect();
            if type_args.len() == params.len() {
                return Some((name.to_string(), type_args));
            }
        }

        None
    }

    pub(crate) fn type_to_ast_ref(&self, ty: &Type) -> AstType {
        let (Type::Struct { name, .. } | Type::Enum { name, .. }) = ty else {
            return type_to_ast(ty);
        };

        if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty) {
            return AstType::Generic {
                name: generic_name,
                type_args,
            };
        }

        type_to_ast(ty)
    }
}
