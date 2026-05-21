use std::collections::HashMap;

use crate::ast::typed::*;
use crate::ast::Pattern;

use super::TypeChecker;

type EnumVariantPayloads = Vec<(String, Option<Type>)>;

impl TypeChecker {
    pub(super) fn enum_variants_for_match(&self, ty: &Type) -> Option<(String, Vec<String>)> {
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

    pub(super) fn enum_variant_payloads_for_match(
        &self,
        ty: &Type,
    ) -> Option<(String, EnumVariantPayloads)> {
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

    pub(super) fn enum_variant_name_from_pattern<'a>(
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

    pub(super) fn explicit_enum_variant_pattern<'a>(
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
