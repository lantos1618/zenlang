//! Reconstructs generic AstType references from already-specialized typed shapes.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::{concrete_name_matches_generic, type_to_ast};
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn generic_type_args_from_type(
        &self,
        concrete_name: &str,
        ty: &Type,
    ) -> Option<(String, Vec<AstType>)> {
        if let Some((name, params)) = self
            .structs
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && concrete_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            self.match_generic_type_params(&name, ty, &params, &mut inferred, &mut conflicts);
            let type_args = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect::<Vec<_>>();
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        if let Some((name, params)) = self
            .enums
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && concrete_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            self.match_generic_type_params(&name, ty, &params, &mut inferred, &mut conflicts);
            let type_args = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect::<Vec<_>>();
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        None
    }

    pub(crate) fn type_to_ast_ref(&self, ty: &Type) -> AstType {
        match ty {
            Type::Struct { name, .. } | Type::Enum { name, .. } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    AstType::Generic {
                        name: generic_name,
                        type_args,
                    }
                } else {
                    type_to_ast(ty)
                }
            }
            _ => type_to_ast(ty),
        }
    }
}
