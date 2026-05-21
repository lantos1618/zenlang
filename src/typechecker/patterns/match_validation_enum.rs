use std::collections::{HashMap, HashSet};

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

mod metadata;

impl TypeChecker {
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
}
