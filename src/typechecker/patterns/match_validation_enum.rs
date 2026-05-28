use std::collections::HashSet;

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;
use crate::error::{CompilerDiagnosticCode::*, Span};

use super::super::{quoted_list, TypeChecker};

impl TypeChecker {
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
