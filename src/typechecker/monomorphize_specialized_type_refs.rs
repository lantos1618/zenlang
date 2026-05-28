use crate::ast::typed::Type;
use crate::error::Span;

use super::TypeChecker;

impl TypeChecker {
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
                for payload in variants.iter().filter_map(|(_, payload)| payload.as_ref()) {
                    self.ensure_specialized_type_refs_for_type(payload, span);
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
