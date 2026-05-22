//! Monomorphization helpers for generic struct and enum specializations.

use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::Span;

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
        let requested = self.mangle_generic_type_name(name, type_args);
        let specialization_key = self.generic_type_specialization_key(
            "struct",
            info.specialization_scope.as_deref(),
            &requested,
        );
        let already_emitted = self
            .specialized_types_seen
            .contains_key(&specialization_key);
        let mangled = self.reserve_generic_type_name(
            &specialization_key,
            &requested,
            info.specialization_scope.as_deref(),
        );
        if !already_emitted {
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
        let requested = self.mangle_generic_type_name(name, type_args);
        let specialization_key = self.generic_type_specialization_key(
            "enum",
            info.specialization_scope.as_deref(),
            &requested,
        );
        let already_emitted = self
            .specialized_types_seen
            .contains_key(&specialization_key);
        let mangled = self.reserve_generic_type_name(
            &specialization_key,
            &requested,
            info.specialization_scope.as_deref(),
        );
        if !already_emitted {
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
}
