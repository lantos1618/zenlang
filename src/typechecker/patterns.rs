//! Pattern helpers — binding, lowering, match kind determination.

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

type EnumVariantPayloads = Vec<(String, Option<Type>)>;

impl TypeChecker {
    /// Look up the payload type for a specific enum variant.
    pub(crate) fn lookup_variant_payload(
        &self,
        scrutinee_type: &Type,
        variant: &str,
    ) -> Option<Type> {
        let enum_name = match scrutinee_type {
            Type::Named(n) => n.as_str(),
            Type::Enum { name, .. } => name.as_str(),
            _ => return None,
        };
        // Try direct lookup from Type::Enum variants
        if let Type::Enum { variants, .. } = scrutinee_type {
            for (vname, payload) in variants {
                if vname == variant {
                    return payload.clone();
                }
            }
        }
        // Fall back to self.enums registry
        if let Some(info) = self.enums.get(enum_name) {
            for (vname, payload) in &info.variants {
                if vname == variant {
                    return payload.as_ref().map(|t| self.resolve_type(t));
                }
            }
        }
        None
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
                        // Shorthand: `{ name }` binds `name` to the field value
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
            Pattern::Enum { payload: None, .. } => {}
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
            Pattern::Wildcard { .. } => TypedPattern::Wildcard,
            Pattern::Identifier { name, .. } => {
                // Check if it's an enum variant
                let enum_name = match scrutinee_type {
                    Type::Named(n) => Some(n.clone()),
                    Type::Enum { name: n, .. } => Some(n.clone()),
                    _ => None,
                };
                if let Some(ref ename) = enum_name {
                    let is_variant = if let Type::Enum { variants, .. } = scrutinee_type {
                        variants.iter().any(|(n, _)| n == name)
                    } else if let Some(info) = self.enums.get(ename.as_str()) {
                        info.variants.iter().any(|(n, _)| n == name)
                    } else {
                        false
                    };
                    if is_variant {
                        return TypedPattern::EnumVariant {
                            type_name: ename.clone(),
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
                // Resolve enum name from scrutinee type if parser left it empty
                let resolved_name = if enum_name.is_empty() {
                    match scrutinee_type {
                        Type::Named(n) => n.clone(),
                        Type::Enum { name, .. } => name.clone(),
                        _ => enum_name.clone(),
                    }
                } else {
                    enum_name.clone()
                };
                // Extract payload bindings
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
            Pattern::Literal { value, .. } => {
                // For now, use Value pattern
                match self.check_expr(value) {
                    Ok(typed) => TypedPattern::Value(Box::new(typed)),
                    Err(_) => TypedPattern::Wildcard,
                }
            }
            _ => TypedPattern::Wildcard,
        }
    }

    pub(crate) fn determine_match_kind(
        &self,
        scrutinee_type: &Type,
        arms: &[MatchArm],
    ) -> MatchKind {
        // Check if all arms are boolean patterns
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

        // Check if scrutinee is an enum type
        match scrutinee_type {
            Type::Named(name) if self.enums.contains_key(name) => {
                return MatchKind::EnumMatch;
            }
            Type::Enum { .. } => {
                return MatchKind::EnumMatch;
            }
            _ => {}
        }

        MatchKind::ValueMatch
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

        let covered: std::collections::HashSet<&str> = arms
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
        let variant_payloads: std::collections::HashMap<&str, Option<&Type>> = variants
            .iter()
            .map(|(variant, payload)| (variant.as_str(), payload.as_ref()))
            .collect();
        let mut seen = std::collections::HashSet::new();
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
            match &arm.pattern {
                Pattern::BoolTrue { span } => {
                    if true_seen || wildcard_seen {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "duplicate match arm for `true`",
                            *span,
                        ));
                    }
                    true_seen = true;
                }
                Pattern::BoolFalse { span } => {
                    if false_seen || wildcard_seen {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "duplicate match arm for `false`",
                            *span,
                        ));
                    }
                    false_seen = true;
                }
                Pattern::Wildcard { span } => {
                    if wildcard_seen || (true_seen && false_seen) {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "redundant wildcard match arm",
                            *span,
                        ));
                    }
                    wildcard_seen = true;
                }
                _ => {}
            }
        }

        if require_exhaustive && !wildcard_seen && !(true_seen && false_seen) {
            let missing = match (true_seen, false_seen) {
                (true, false) => "`false`",
                (false, true) => "`true`",
                _ => "`true`, `false`",
            };
            self.diagnostics.push(Diagnostic::error(
                "E4006",
                format!("non-exhaustive bool match: missing {missing}"),
                span,
            ));
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
        variants: &std::collections::HashMap<&str, Option<&Type>>,
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
