//! Recover source generic names and arguments from specialized concrete types.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::type_to_ast;
use super::TypeChecker;

impl TypeChecker {
    pub(super) fn generic_type_args_from_type(
        &self,
        concrete_name: &str,
        ty: &Type,
    ) -> Option<(String, Vec<AstType>)> {
        if let Some((name, params)) = self
            .structs
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && self.concrete_type_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let type_args = self.infer_specialized_type_args(&name, ty, &params);
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        if let Some((name, params)) = self
            .enums
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && self.concrete_type_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let type_args = self.infer_specialized_type_args(&name, ty, &params);
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        None
    }

    pub(crate) fn remembered_specialized_type_args(
        &self,
        concrete_name: &str,
        generic_name: &str,
    ) -> Option<Vec<AstType>> {
        let source_name = self.specialized_type_generic_names.get(concrete_name)?;
        if source_name != generic_name {
            return None;
        }
        let type_args = self.specialized_type_args.get(concrete_name)?;
        Some(type_args.clone())
    }

    fn infer_specialized_type_args(
        &self,
        generic_name: &str,
        ty: &Type,
        params: &[String],
    ) -> Vec<AstType> {
        let mut inferred = HashMap::new();
        let mut conflicts = Vec::new();
        self.match_generic_type_params(generic_name, ty, params, &mut inferred, &mut conflicts);
        params
            .iter()
            .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
            .collect()
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
