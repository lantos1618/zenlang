//! Pattern helpers — binding, lowering, match kind determination.

use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;

use super::TypeChecker;

impl TypeChecker {
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
                payload: Some(p), ..
            } => {
                self.bind_pattern(p, &Type::Unknown);
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
                if let Type::Named(enum_name) = scrutinee_type {
                    if let Some(info) = self.enums.get(enum_name) {
                        if info.variants.iter().any(|(n, _)| n == name) {
                            return TypedPattern::EnumVariant {
                                type_name: enum_name.clone(),
                                variant: name.clone(),
                                bindings: Vec::new(),
                            };
                        }
                    }
                }
                TypedPattern::Wildcard // bind to variable
            }
            Pattern::Enum {
                enum_name,
                variant,
                payload: _,
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
                TypedPattern::EnumVariant {
                    type_name: resolved_name,
                    variant: variant.clone(),
                    bindings: Vec::new(), // TODO: extract payload bindings
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
        if let Type::Named(name) = scrutinee_type {
            if self.enums.contains_key(name) {
                return MatchKind::EnumMatch;
            }
        }

        MatchKind::ValueMatch
    }
}
