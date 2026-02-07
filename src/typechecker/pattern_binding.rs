use crate::ast::{primitive_from_str, AstType, Pattern};
use crate::error::Result;
use crate::typechecker::TypeChecker;

/// Pattern binding helpers for adding pattern-matched variables to scope
impl TypeChecker {
    pub fn add_pattern_bindings_to_scope(&mut self, pattern: &Pattern) -> Result<()> {
        // Default to I32 when no type context is available (legacy behavior)
        self.add_pattern_bindings_to_scope_with_type(pattern, &AstType::I32)
    }

    /// Helper to resolve the payload type for an enum variant pattern match
    fn resolve_enum_payload_type(&self, variant: &str, scrutinee_type: &AstType) -> AstType {
        match scrutinee_type {
            AstType::Generic {
                name: enum_name,
                type_args,
            } => {
                if self.well_known.is_result(enum_name) && type_args.len() >= 2 {
                    if self.well_known.is_ok(variant) {
                        type_args[0].clone()
                    } else if self.well_known.is_err(variant) {
                        type_args[1].clone()
                    } else {
                        AstType::I32
                    }
                } else if self.well_known.is_option(enum_name) && !type_args.is_empty() {
                    if self.well_known.is_some(variant) {
                        type_args[0].clone()
                    } else {
                        AstType::Void
                    }
                } else {
                    scrutinee_type.clone()
                }
            }
            AstType::Enum {
                name: enum_name,
                variants,
            } => {
                if self.well_known.is_option(enum_name) || self.well_known.is_result(enum_name) {
                    if let Some(enum_variant) = variants.iter().find(|v| v.name == variant) {
                        if let Some(payload_ty) = &enum_variant.payload {
                            return payload_ty.clone();
                        }
                        return AstType::Void;
                    }
                }
                scrutinee_type.clone()
            }
            _ => scrutinee_type.clone(),
        }
    }

    /// Helper to unwrap primitive types from Generic wrapper
    fn unwrap_primitive_generic(&self, scrutinee_type: &AstType) -> AstType {
        if let AstType::Generic {
            name: type_name,
            type_args,
        } = scrutinee_type
        {
            if type_args.is_empty() {
                // Try the canonical lowercase name first, then try lowercasing the input
                if let Some(prim) = primitive_from_str(type_name)
                    .or_else(|| primitive_from_str(&type_name.to_lowercase()))
                {
                    return prim;
                }
                return match type_name.as_str() {
                    "string" => AstType::StaticString,
                    "String" => crate::ast::resolve_string_struct_type(),
                    _ => scrutinee_type.clone(),
                };
            }
        }
        scrutinee_type.clone()
    }

    pub fn add_pattern_bindings_to_scope_with_type(
        &mut self,
        pattern: &Pattern,
        scrutinee_type: &AstType,
    ) -> Result<()> {
        match pattern {
            Pattern::Identifier(name) => {
                let binding_type = self.unwrap_primitive_generic(scrutinee_type);
                self.declare_variable(name, binding_type, false)?;
            }
            Pattern::EnumLiteral { variant, payload } => {
                if let Some(payload_pattern) = payload {
                    let payload_type = self.resolve_enum_payload_type(variant, scrutinee_type);
                    self.add_pattern_bindings_to_scope_with_type(payload_pattern, &payload_type)?;
                }
            }
            Pattern::EnumVariant {
                variant, payload, ..
            } => {
                if let Some(payload_pattern) = payload {
                    let payload_type = self.resolve_enum_payload_type(variant, scrutinee_type);
                    self.add_pattern_bindings_to_scope_with_type(payload_pattern, &payload_type)?;
                }
            }
            Pattern::Binding { name, pattern } => {
                // Binding pattern: name @ pattern
                // Add the name as a variable with the scrutinee type
                self.declare_variable(name, scrutinee_type.clone(), false)?;
                // And recursively process the pattern
                self.add_pattern_bindings_to_scope_with_type(pattern, scrutinee_type)?;
            }
            Pattern::Or(patterns) => {
                // For or patterns, we need to ensure all alternatives bind the same names
                // For now, just process the first one
                if let Some(first) = patterns.first() {
                    self.add_pattern_bindings_to_scope_with_type(first, scrutinee_type)?;
                }
            }
            Pattern::Struct { fields, .. } => {
                // For struct patterns, add bindings for all fields
                // Uses scrutinee_type for all bindings; field-specific types need struct lookup
                for field in fields {
                    self.add_pattern_bindings_to_scope_with_type(&field.1, scrutinee_type)?;
                }
            }
            Pattern::Type { binding, .. } => {
                // Type pattern with optional binding
                if let Some(name) = binding {
                    self.declare_variable(name, scrutinee_type.clone(), false)?;
                }
            }
            // Tuple patterns - recursively bind patterns in the tuple
            Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    self.add_pattern_bindings_to_scope_with_type(pattern, scrutinee_type)?;
                }
            }
            // Other patterns don't create bindings
            Pattern::Wildcard
            | Pattern::Literal(_)
            | Pattern::Range { .. }
            | Pattern::Guard { .. } => {}
        }
        Ok(())
    }
}
