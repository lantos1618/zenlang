//! Monomorphization helpers for generic struct and enum specializations.

use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::Span;

use super::monomorphize_types::substitute_ast_type;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn specialize_generic_struct(
        &mut self,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Type> {
        let Some(info) = self.structs.get(name).cloned() else {
            return HashMap::new();
        };
        let substitutions =
            self.type_param_substitutions(&info.type_params, type_args, "struct", name, span);
        self.check_generic_bounds(&info.type_param_bounds, &substitutions, span);
        for (_, field_type) in &info.fields {
            self.ensure_specialized_type_refs(field_type, &substitutions, span);
        }
        let fields: Vec<(String, Type)> = info
            .fields
            .iter()
            .map(|(field_name, field_type)| {
                (
                    field_name.clone(),
                    self.substitute_type(field_type, &substitutions),
                )
            })
            .collect();
        for (_, field_type) in &fields {
            self.ensure_specialized_type_refs_for_type(field_type, span);
        }
        let field_map = fields.iter().cloned().collect();
        let mangled = self.mangle_generic_type_name(name, type_args);
        if self.specialized_types_seen.insert(mangled.clone()) {
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: TypeDefKind::Struct { fields },
                methods: Vec::new(),
                span,
            });
        }
        field_map
    }

    pub(crate) fn specialize_generic_enum(
        &mut self,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Option<Type>> {
        let Some(info) = self.enums.get(name).cloned() else {
            return HashMap::new();
        };
        let substitutions =
            self.type_param_substitutions(&info.type_params, type_args, "enum", name, span);
        self.check_generic_bounds(&info.type_param_bounds, &substitutions, span);
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.ensure_specialized_type_refs(payload, &substitutions, span);
            }
        }
        let variants: Vec<(String, Option<Type>)> = info
            .variants
            .iter()
            .map(|(variant_name, payload)| {
                (
                    variant_name.clone(),
                    payload
                        .as_ref()
                        .map(|payload| self.substitute_type(payload, &substitutions)),
                )
            })
            .collect();
        for (_, payload) in &variants {
            if let Some(payload) = payload {
                self.ensure_specialized_type_refs_for_type(payload, span);
            }
        }
        let variant_map = variants.iter().cloned().collect();
        let mangled = self.mangle_generic_type_name(name, type_args);
        if self.specialized_types_seen.insert(mangled.clone()) {
            let typed_variants = variants
                .into_iter()
                .enumerate()
                .map(
                    |(tag, (variant_name, payload))| crate::ast::typed::TypedVariant {
                        name: variant_name,
                        tag: tag as u32,
                        payload: payload.map(|ty| vec![("payload".to_string(), ty)]),
                    },
                )
                .collect();
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: TypeDefKind::Enum {
                    variants: typed_variants,
                },
                methods: Vec::new(),
                span,
            });
        }
        variant_map
    }

    fn ensure_specialized_type_refs(
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

    fn ensure_specialized_type_refs_for_type(&mut self, ty: &Type, span: Span) {
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
