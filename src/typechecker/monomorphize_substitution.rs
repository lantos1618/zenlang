//! Monomorphization helpers for substituting generic AstType references.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::substitute_ast_type;
use super::TypeChecker;

impl TypeChecker {
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
            AstType::RawPtr(inner) => {
                Type::RawPtr(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Slice(inner) => {
                Type::Slice(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Array { elem, size } => Type::Array {
                elem: Box::new(self.substitute_type(elem, substitutions)),
                size: *size,
            },
            AstType::Function { params, ret } => Type::Function {
                params: params
                    .iter()
                    .map(|param| self.substitute_type(param, substitutions))
                    .collect(),
                ret: Box::new(self.substitute_type(ret, substitutions)),
            },
            AstType::Generic { name, type_args } => {
                let subst_args: Vec<AstType> = type_args
                    .iter()
                    .map(|arg| substitute_ast_type(arg, substitutions))
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
