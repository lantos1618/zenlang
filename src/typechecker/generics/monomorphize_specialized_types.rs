use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::Span;

use super::super::{BehaviorBound, TypeChecker};
use super::monomorphize_names::reserve_specialization_name;

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
        let substitutions = self.bound_checked_substitutions(
            &info.type_params,
            &info.type_param_bounds,
            "struct",
            name,
            type_args,
            span,
        );
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
        self.emit_specialized_type(
            "struct",
            name,
            type_args,
            info.specialization_scope.as_deref(),
            span,
            || TypeDefKind::Struct { fields },
        );
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
        let substitutions = self.bound_checked_substitutions(
            &info.type_params,
            &info.type_param_bounds,
            "enum",
            name,
            type_args,
            span,
        );
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
        for payload in variants.iter().filter_map(|(_, payload)| payload.as_ref()) {
            self.ensure_specialized_type_refs_for_type(payload, span);
        }
        let variant_map = variants.iter().cloned().collect();
        self.emit_specialized_type(
            "enum",
            name,
            type_args,
            info.specialization_scope.as_deref(),
            span,
            || {
                let variants = variants
                    .into_iter()
                    .enumerate()
                    .map(
                        |(tag, (variant_name, payload))| crate::ast::typed::TypedVariant {
                            name: variant_name,
                            tag: tag as u32,
                            payload,
                        },
                    )
                    .collect();
                TypeDefKind::Enum { variants }
            },
        );
        variant_map
    }

    /// Shared prefix of struct/enum specialization: map type params to the
    /// concrete args and verify their behavior bounds, returning the
    /// substitution map both then apply to fields/variants.
    fn bound_checked_substitutions(
        &mut self,
        type_params: &[String],
        type_param_bounds: &HashMap<String, BehaviorBound>,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Type> {
        let substitutions = self.type_param_substitutions(type_params, type_args, kind, name, span);
        self.check_generic_bounds(type_param_bounds, &substitutions, span);
        substitutions
    }

    /// Shared tail of struct/enum specialization: reserve the mangled name and,
    /// if this concrete instantiation hasn't been emitted yet, push its
    /// definition. `make_kind` builds the (struct- or enum-specific) body and is
    /// only invoked on first emission.
    fn emit_specialized_type(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        specialization_scope: Option<&str>,
        span: Span,
        make_kind: impl FnOnce() -> TypeDefKind,
    ) {
        let (mangled, already_emitted) =
            self.reserve_specialized_type_definition(kind, name, type_args, specialization_scope);
        if !already_emitted {
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: make_kind(),
                span,
            });
        }
    }

    fn reserve_specialized_type_definition(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        specialization_scope: Option<&str>,
    ) -> (String, bool) {
        let requested = self.mangle_generic_type_name(name, type_args);
        let specialization_key =
            self.generic_specialization_key(kind, specialization_scope, &requested);
        let already_emitted = self
            .specialized_types_seen
            .contains_key(&specialization_key);
        let mangled = reserve_specialization_name(
            &mut self.specialized_types_seen,
            &mut self.specialized_type_name_owners,
            &specialization_key,
            &requested,
            specialization_scope,
        );
        let concrete_type_args: Vec<AstType> = type_args
            .iter()
            .map(|arg| self.type_to_ast_ref(&self.resolve_type(arg)))
            .collect();
        self.specialized_type_generic_names
            .insert(mangled.clone(), name.to_string());
        self.specialized_type_args
            .insert(mangled.clone(), concrete_type_args);

        (mangled, already_emitted)
    }
}
