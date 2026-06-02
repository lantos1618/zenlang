use std::collections::HashSet;

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::{quoted_list, TypeChecker};

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

    pub(crate) fn check_bool_match_patterns(
        &mut self,
        arms: &[MatchArm],
        require_exhaustive: bool,
        span: Span,
    ) {
        let mut true_seen = false;
        let mut false_seen = false;
        let mut wildcard_seen = false;

        for arm in arms {
            let (value, seen, span) = match &arm.pattern {
                Pattern::BoolTrue { span } => ("true", &mut true_seen, *span),
                Pattern::BoolFalse { span } => ("false", &mut false_seen, *span),
                Pattern::Wildcard { span } => {
                    if wildcard_seen || (true_seen && false_seen) {
                        self.push_error(E4005, "redundant wildcard match arm", *span);
                    }
                    wildcard_seen = true;
                    continue;
                }
                _ => continue,
            };
            if *seen || wildcard_seen {
                self.push_error(E4005, format!("duplicate match arm for `{value}`"), span);
            }
            *seen = true;
        }

        if require_exhaustive && !wildcard_seen && !(true_seen && false_seen) {
            let missing_values: &[&str] = match (true_seen, false_seen) {
                (true, false) => &["false"],
                (false, true) => &["true"],
                _ => &["true", "false"],
            };
            let missing = quoted_list(missing_values);
            let replacement = missing_values
                .iter()
                .map(|value| format!("        | {value} {{ <expression> }}"))
                .collect::<Vec<_>>()
                .join("\n");
            let insertion = Span::new(span.file_id, span.end, span.end);
            self.diagnostics.push(
                Diagnostic::error_code(
                    E4006,
                    format!("non-exhaustive bool match: missing {missing}"),
                    span,
                )
                .with_fix(
                    "add_missing_bool_match_arm",
                    "Add missing bool match arm",
                    insertion,
                    format!("\n{replacement}"),
                ),
            );
        }
    }

    pub(crate) fn check_enum_match_patterns(
        &mut self,
        scrutinee_type: &Type,
        arms: &[MatchArm],
        span: Span,
    ) {
        let Some(enum_name) = self.enum_type_name(scrutinee_type) else {
            return;
        };
        let enum_name = enum_name.to_string();
        let variants = self.enum_variants_for_type(scrutinee_type);
        let mut seen = HashSet::new();
        let mut wildcard_seen = false;

        for arm in arms {
            if let Pattern::Wildcard { span } = &arm.pattern {
                if wildcard_seen || seen.len() == variants.len() {
                    self.push_error(E4002, "redundant wildcard match arm", *span);
                }
                wildcard_seen = true;
                continue;
            }

            let (variant, has_payload) = match &arm.pattern {
                Pattern::Identifier { name, .. }
                    if variants.iter().any(|(variant, _)| variant == name)
                        || name
                            .chars()
                            .next()
                            .is_some_and(|first| first.is_ascii_uppercase()) =>
                {
                    (name.as_str(), false)
                }
                Pattern::Enum {
                    variant, payload, ..
                } => (variant.as_str(), payload.is_some()),
                _ => continue,
            };
            let span = arm.pattern.span();
            let Some((_, expected_payload)) = variants.iter().find(|(name, _)| name == variant)
            else {
                self.push_error(
                    E4001,
                    format!("enum `{enum_name}` has no variant `{variant}`"),
                    span,
                );
                continue;
            };

            if wildcard_seen {
                self.push_error(
                    E4002,
                    format!("redundant match arm for `{enum_name}.{variant}`"),
                    span,
                );
            } else if !seen.insert(variant) {
                self.push_error(
                    E4002,
                    format!("duplicate match arm for `{enum_name}.{variant}`"),
                    span,
                );
            }

            match (expected_payload.is_some(), has_payload) {
                (true, false) => self.push_error(
                    E4003,
                    format!("match arm `{enum_name}.{variant}` requires a payload"),
                    span,
                ),
                (false, true) => self.push_error(
                    E4004,
                    format!("match arm `{enum_name}.{variant}` does not accept a payload"),
                    span,
                ),
                _ => {}
            }
        }

        if !wildcard_seen {
            let missing: Vec<&str> = variants
                .iter()
                .map(|(variant, _)| variant.as_str())
                .filter(|variant| !seen.contains(variant))
                .collect();
            if !missing.is_empty() {
                let missing = quoted_list(&missing);
                self.push_error(
                    E4000,
                    format!("non-exhaustive match on `{enum_name}`: missing {missing}"),
                    span,
                );
            }
        }
    }
}
