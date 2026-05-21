//! Pattern helpers — binding, lowering, match kind determination.

use crate::ast::typed::*;
use crate::ast::Pattern;

use super::TypeChecker;

mod match_validation;
mod match_validation_bool;
mod match_validation_enum;

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
}
