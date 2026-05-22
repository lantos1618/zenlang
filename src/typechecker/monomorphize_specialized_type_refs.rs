//! Ensure nested generic type references are emitted during monomorphization.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;
use crate::error::Span;

use super::monomorphize_types::substitute_ast_type;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn ensure_specialized_type_refs(
        &mut self,
        ast_type: &AstType,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        match substitute_ast_type(ast_type, substitutions) {
            AstType::Generic { name, type_args } => {
                for type_arg in &type_args {
                    self.ensure_specialized_type_refs(type_arg, substitutions, span);
                }
                if self.structs.contains_key(&name) {
                    self.specialize_generic_struct(&name, &type_args, span);
                } else if self.enums.contains_key(&name) {
                    self.specialize_generic_enum(&name, &type_args, span);
                }
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.ensure_specialized_type_refs(&inner, substitutions, span);
            }
            AstType::Array { elem, .. } => {
                self.ensure_specialized_type_refs(&elem, substitutions, span);
            }
            AstType::Function { params, ret } => {
                for param in &params {
                    self.ensure_specialized_type_refs(param, substitutions, span);
                }
                self.ensure_specialized_type_refs(&ret, substitutions, span);
            }
            _ => {}
        }
    }

    pub(crate) fn ensure_specialized_type_refs_for_type(&mut self, ty: &Type, span: Span) {
        match ty {
            Type::Struct { name, fields } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    self.specialize_generic_struct(&generic_name, &type_args, span);
                }
                for (_, field_type) in fields {
                    self.ensure_specialized_type_refs_for_type(field_type, span);
                }
            }
            Type::Enum { name, variants } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    self.specialize_generic_enum(&generic_name, &type_args, span);
                }
                for (_, payload) in variants {
                    if let Some(payload) = payload {
                        self.ensure_specialized_type_refs_for_type(payload, span);
                    }
                }
            }
            Type::Array { elem, .. }
            | Type::Slice(elem)
            | Type::Ptr(elem)
            | Type::MutPtr(elem)
            | Type::RawPtr(elem) => self.ensure_specialized_type_refs_for_type(elem, span),
            Type::Function { params, ret } => {
                for param in params {
                    self.ensure_specialized_type_refs_for_type(param, span);
                }
                self.ensure_specialized_type_refs_for_type(ret, span);
            }
            _ => {}
        }
    }
}
