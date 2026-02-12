//! Monomorphization helpers — generic type argument inference and substitution.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::TypeChecker;

impl TypeChecker {
    /// Infer type arguments for a generic function by matching actual arg types
    /// against parameter types containing type params.
    pub(crate) fn infer_type_args(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> HashMap<String, Type> {
        let mut map = HashMap::new();
        for ((_name, param_ty), arg_ty) in param_types.iter().zip(arg_types.iter()) {
            match_type_param(param_ty, arg_ty, type_params, &mut map);
        }
        map
    }

    /// Substitute type parameters in an AstType, returning a resolved Type.
    pub(crate) fn substitute_type(
        &self,
        ast_type: &AstType,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match ast_type {
            AstType::Named(name) => {
                if let Some(concrete) = substitutions.get(name) {
                    concrete.clone()
                } else {
                    self.resolve_type(ast_type)
                }
            }
            AstType::Ptr(inner) => Type::Ptr(Box::new(self.substitute_type(inner, substitutions))),
            AstType::MutPtr(inner) => {
                Type::MutPtr(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Slice(inner) => {
                Type::Slice(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Generic { name, type_args } => {
                let subst_args: Vec<AstType> = type_args
                    .iter()
                    .map(|a| substitute_ast_type(a, substitutions))
                    .collect();
                self.resolve_type(&AstType::Generic {
                    name: name.clone(),
                    type_args: subst_args,
                })
            }
            _ => self.resolve_type(ast_type),
        }
    }
}

/// Recursively match a parameter AstType against an actual Type to discover
/// type parameter bindings.
fn match_type_param(
    param: &AstType,
    actual: &Type,
    type_params: &[String],
    map: &mut HashMap<String, Type>,
) {
    match param {
        AstType::Named(name) if type_params.contains(name) => {
            map.entry(name.clone()).or_insert_with(|| actual.clone());
        }
        AstType::Ptr(inner) => {
            if let Type::Ptr(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::MutPtr(inner) => {
            if let Type::MutPtr(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::Slice(inner) => {
            if let Type::Slice(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::Generic { name: _, type_args } => {
            for arg in type_args {
                if let AstType::Named(n) = arg {
                    if type_params.contains(n) && !map.contains_key(n) {
                        map.entry(n.clone()).or_insert_with(|| actual.clone());
                    }
                }
            }
        }
        _ => {}
    }
}

/// Substitute type parameters in an AstType, returning a new AstType.
fn substitute_ast_type(ast_type: &AstType, substitutions: &HashMap<String, Type>) -> AstType {
    match ast_type {
        AstType::Named(name) => {
            if let Some(concrete) = substitutions.get(name) {
                type_to_ast(concrete)
            } else {
                ast_type.clone()
            }
        }
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_ast_type(inner, substitutions))),
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|a| substitute_ast_type(a, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

/// Convert a resolved Type back to an AstType (best-effort, for substitution).
pub(crate) fn type_to_ast(ty: &Type) -> AstType {
    match ty {
        Type::I8 => AstType::I8,
        Type::I16 => AstType::I16,
        Type::I32 => AstType::I32,
        Type::I64 => AstType::I64,
        Type::U8 => AstType::U8,
        Type::U16 => AstType::U16,
        Type::U32 => AstType::U32,
        Type::U64 => AstType::U64,
        Type::Usize => AstType::Usize,
        Type::F32 => AstType::F32,
        Type::F64 => AstType::F64,
        Type::Bool => AstType::Bool,
        Type::Void => AstType::Void,
        Type::Str => AstType::Str,
        Type::String => AstType::Named("String".into()),
        Type::Named(n) => AstType::Named(n.clone()),
        Type::Struct { name, .. } => AstType::Named(name.clone()),
        Type::Enum { name, .. } => AstType::Named(name.clone()),
        Type::Ptr(inner) => AstType::Ptr(Box::new(type_to_ast(inner))),
        Type::MutPtr(inner) => AstType::MutPtr(Box::new(type_to_ast(inner))),
        Type::RawPtr(inner) => AstType::RawPtr(Box::new(type_to_ast(inner))),
        Type::Slice(inner) => AstType::Slice(Box::new(type_to_ast(inner))),
        Type::Array { elem, size } => AstType::Array {
            elem: Box::new(type_to_ast(elem)),
            size: *size,
        },
        _ => AstType::Void,
    }
}
