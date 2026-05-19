use std::collections::{HashMap, HashSet};

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

type EnumVariantPayloads = Vec<(String, Option<Type>)>;

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

        match scrutinee_type {
            Type::Named(name) if self.enums.contains_key(name) => MatchKind::EnumMatch,
            Type::Enum { .. } => MatchKind::EnumMatch,
            _ => MatchKind::ValueMatch,
        }
    }

    pub(crate) fn check_match_exhaustiveness(
        &mut self,
        scrutinee_type: &Type,
        arms: &[MatchArm],
        span: Span,
    ) {
        let Some((enum_name, variants)) = self.enum_variants_for_match(scrutinee_type) else {
            return;
        };

        if arms
            .iter()
            .any(|arm| matches!(arm.pattern, Pattern::Wildcard { .. }))
        {
            return;
        }

        let covered: HashSet<&str> = arms
            .iter()
            .filter_map(|arm| self.enum_variant_name_from_pattern(scrutinee_type, &arm.pattern))
            .collect();
        let missing: Vec<&str> = variants
            .iter()
            .map(String::as_str)
            .filter(|variant| !covered.contains(variant))
            .collect();

        if !missing.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E4000",
                format!(
                    "non-exhaustive match on `{}`: missing {}",
                    enum_name,
                    missing
                        .iter()
                        .map(|variant| format!("`{variant}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                span,
            ));
        }
    }

    pub(crate) fn check_enum_match_patterns(&mut self, scrutinee_type: &Type, arms: &[MatchArm]) {
        let Some((enum_name, variants)) = self.enum_variant_payloads_for_match(scrutinee_type)
        else {
            return;
        };
        let variant_payloads: HashMap<&str, Option<&Type>> = variants
            .iter()
            .map(|(variant, payload)| (variant.as_str(), payload.as_ref()))
            .collect();
        let mut seen = HashSet::new();
        let mut wildcard_seen = false;

        for arm in arms {
            if let Pattern::Wildcard { span } = &arm.pattern {
                if wildcard_seen || seen.len() == variant_payloads.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "E4002",
                        "redundant wildcard match arm",
                        *span,
                    ));
                }
                wildcard_seen = true;
                continue;
            }

            let Some((variant, has_payload)) =
                self.explicit_enum_variant_pattern(&arm.pattern, &variant_payloads)
            else {
                continue;
            };
            let span = arm.pattern.span();
            let Some(expected_payload) = variant_payloads.get(variant) else {
                self.diagnostics.push(Diagnostic::error(
                    "E4001",
                    format!("enum `{enum_name}` has no variant `{variant}`"),
                    span,
                ));
                continue;
            };

            if wildcard_seen {
                self.diagnostics.push(Diagnostic::error(
                    "E4002",
                    format!("redundant match arm for `{enum_name}.{variant}`"),
                    span,
                ));
            } else if !seen.insert(variant.to_string()) {
                self.diagnostics.push(Diagnostic::error(
                    "E4002",
                    format!("duplicate match arm for `{enum_name}.{variant}`"),
                    span,
                ));
            }

            match (expected_payload.is_some(), has_payload) {
                (true, false) => self.diagnostics.push(Diagnostic::error(
                    "E4003",
                    format!("match arm `{enum_name}.{variant}` requires a payload"),
                    span,
                )),
                (false, true) => self.diagnostics.push(Diagnostic::error(
                    "E4004",
                    format!("match arm `{enum_name}.{variant}` does not accept a payload"),
                    span,
                )),
                _ => {}
            }
        }
    }

    fn enum_variants_for_match(&self, ty: &Type) -> Option<(String, Vec<String>)> {
        match ty {
            Type::Enum { name, variants } => Some((
                name.clone(),
                variants.iter().map(|(name, _)| name.clone()).collect(),
            )),
            Type::Named(name) => self.enums.get(name).map(|info| {
                (
                    name.clone(),
                    info.variants
                        .iter()
                        .map(|(variant, _)| variant.clone())
                        .collect(),
                )
            }),
            _ => None,
        }
    }

    fn enum_variant_payloads_for_match(&self, ty: &Type) -> Option<(String, EnumVariantPayloads)> {
        match ty {
            Type::Enum { name, variants } => Some((name.clone(), variants.clone())),
            Type::Named(name) => self.enums.get(name).map(|info| {
                (
                    name.clone(),
                    info.variants
                        .iter()
                        .map(|(variant, payload)| {
                            (
                                variant.clone(),
                                payload.as_ref().map(|ty| self.resolve_type(ty)),
                            )
                        })
                        .collect(),
                )
            }),
            _ => None,
        }
    }

    fn enum_variant_name_from_pattern<'a>(
        &self,
        scrutinee_type: &Type,
        pattern: &'a Pattern,
    ) -> Option<&'a str> {
        match pattern {
            Pattern::Identifier { name, .. } => {
                let (_, variants) = self.enum_variants_for_match(scrutinee_type)?;
                variants
                    .iter()
                    .any(|variant| variant == name)
                    .then_some(name)
            }
            Pattern::Enum { variant, .. } => Some(variant),
            _ => None,
        }
    }

    fn explicit_enum_variant_pattern<'a>(
        &self,
        pattern: &'a Pattern,
        variants: &HashMap<&str, Option<&Type>>,
    ) -> Option<(&'a str, bool)> {
        match pattern {
            Pattern::Identifier { name, .. } => (variants.contains_key(name.as_str())
                || name
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_uppercase()))
            .then_some((name.as_str(), false)),
            Pattern::Enum {
                variant, payload, ..
            } => Some((variant, payload.is_some())),
            _ => None,
        }
    }
}
