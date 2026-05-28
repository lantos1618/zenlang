use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;

use super::TypeChecker;

mod match_validation_bool;
mod match_validation_enum;

impl TypeChecker {
    pub(crate) fn determine_match_kind(
        &self,
        scrutinee_type: &Type,
        arms: &[MatchArm],
    ) -> MatchKind {
        let all_bool = arms.iter().all(|arm| {
            matches!(
                &arm.pattern,
                Pattern::BoolTrue { .. } | Pattern::BoolFalse { .. }
            )
        });
        if all_bool {
            if arms.len() >= 2 {
                return MatchKind::ConditionalElse;
            }
            return MatchKind::Conditional;
        }

        if self.enum_type_name(scrutinee_type).is_some() {
            MatchKind::EnumMatch
        } else {
            MatchKind::ValueMatch
        }
    }

    pub(crate) fn lookup_variant_payload(
        &self,
        scrutinee_type: &Type,
        variant: &str,
    ) -> Option<Type> {
        self.enum_variants_for_type(scrutinee_type)
            .into_iter()
            .find(|(name, _)| name == variant)
            .and_then(|(_, payload)| payload)
    }

    pub(crate) fn bind_pattern(&mut self, pattern: &Pattern, scrutinee_type: &Type) {
        match pattern {
            Pattern::Identifier { name, .. } => {
                self.define_var(name, scrutinee_type.clone());
            }
            Pattern::Struct { fields, .. } => {
                for (field_name, sub_pattern) in fields {
                    let field_ty = self.lookup_field_type(scrutinee_type, field_name);
                    if let Some(p) = sub_pattern {
                        self.bind_pattern(p, &field_ty);
                    } else {
                        self.define_var(field_name, field_ty);
                    }
                }
            }
            Pattern::Enum {
                variant,
                payload: Some(p),
                ..
            } => {
                let payload_ty = self
                    .lookup_variant_payload(scrutinee_type, variant)
                    .unwrap_or(Type::Unknown);
                self.bind_pattern(p, &payload_ty);
            }
            _ => {}
        }
    }

    pub(crate) fn lower_pattern(
        &mut self,
        pattern: &Pattern,
        scrutinee_type: &Type,
    ) -> TypedPattern {
        match pattern {
            Pattern::BoolTrue { .. } => TypedPattern::Bool(true),
            Pattern::BoolFalse { .. } => TypedPattern::Bool(false),
            Pattern::Identifier { name, .. } => {
                if let Some(enum_name) = self.enum_type_name(scrutinee_type) {
                    if self
                        .enum_variants_for_type(scrutinee_type)
                        .iter()
                        .any(|(variant, _)| variant == name)
                    {
                        return TypedPattern::EnumVariant {
                            type_name: enum_name.to_string(),
                            variant: name.clone(),
                            bindings: Vec::new(),
                        };
                    }
                }
                TypedPattern::Wildcard // bind to variable
            }
            Pattern::Enum {
                enum_name,
                variant,
                payload,
                ..
            } => {
                let resolved_name = if enum_name.is_empty() {
                    self.enum_type_name(scrutinee_type)
                        .unwrap_or(enum_name)
                        .to_string()
                } else {
                    enum_name.clone()
                };
                let bindings = if let Some(p) = payload {
                    let payload_ty = self
                        .lookup_variant_payload(scrutinee_type, variant)
                        .unwrap_or(Type::Unknown);
                    match p.as_ref() {
                        Pattern::Identifier { name, .. } => {
                            vec![(name.clone(), payload_ty)]
                        }
                        _ => Vec::new(),
                    }
                } else {
                    Vec::new()
                };
                TypedPattern::EnumVariant {
                    type_name: resolved_name,
                    variant: variant.clone(),
                    bindings,
                }
            }
            Pattern::Literal { value, .. } => match self.check_expr(value) {
                Ok(typed) => TypedPattern::Value(Box::new(typed)),
                Err(_) => TypedPattern::Wildcard,
            },
            _ => TypedPattern::Wildcard,
        }
    }

    fn enum_type_name<'a>(&'a self, ty: &'a Type) -> Option<&'a str> {
        match ty {
            Type::Enum { name, .. } => Some(name),
            Type::Named(name) if self.enums.contains_key(name) => Some(name),
            _ => None,
        }
    }

    fn enum_variants_for_type(&self, ty: &Type) -> Vec<(String, Option<Type>)> {
        match ty {
            Type::Enum { variants, .. } => variants.clone(),
            _ => self
                .enum_type_name(ty)
                .and_then(|enum_name| self.enums.get(enum_name))
                .map(|info| {
                    info.variants
                        .iter()
                        .map(|(variant, payload)| {
                            (
                                variant.clone(),
                                payload.as_ref().map(|ty| self.resolve_type(ty)),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}
