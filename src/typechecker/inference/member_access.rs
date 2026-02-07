//! Member access and struct field inference

use crate::ast::AstType;
use crate::error::{CompileError, Result};
use crate::typechecker::{EnumInfo, StructInfo};
use std::collections::HashMap;

/// Infer the type of a member access expression
pub fn infer_member_type(
    object_type: &AstType,
    member: &str,
    structs: &HashMap<String, StructInfo>,
    enums: &HashMap<String, EnumInfo>,
    span: Option<crate::error::Span>,
) -> Result<AstType> {
    match object_type {
        AstType::Struct { name, .. } => {
            if let Some(struct_info) = structs.get(name) {
                if let Some(field_type) = struct_info.get_field_type(member) {
                    return Ok(field_type.clone());
                }
                let available_fields: Vec<String> = struct_info.fields.iter().map(|(n, _)| n.clone()).collect();
                let suggestion = if available_fields.is_empty() {
                    String::new()
                } else {
                    format!(". Available fields: {}", available_fields.join(", "))
                };
                Err(CompileError::TypeError(
                    format!("Struct '{}' has no field '{}'{}", name, member, suggestion),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Unknown struct type: {}", name),
                    span,
                ))
            }
        }
        // Handle pointer to struct types
        t if t.is_ptr_type() => {
            // Dereference the pointer and check the inner type
            if let Some(inner) = t.ptr_inner() {
                infer_member_type(inner, member, structs, enums, span)
            } else {
                Err(CompileError::TypeError(
                    format!("Cannot access member '{}' on type {}: pointer does not point to a struct", member, t),
                    span,
                ))
            }
        }
        // Handle Generic types that represent structs
        AstType::Generic { name, .. } => {
            // Try to look up the struct info by name
            if let Some(struct_info) = structs.get(name) {
                if let Some(field_type) = struct_info.get_field_type(member) {
                    return Ok(field_type.clone());
                }
                let available_fields: Vec<String> = struct_info.fields.iter().map(|(n, _)| n.clone()).collect();
                let suggestion = if available_fields.is_empty() {
                    String::new()
                } else {
                    format!(". Available fields: {}", available_fields.join(", "))
                };
                Err(CompileError::TypeError(
                    format!("Struct '{}' has no field '{}'{}", name, member, suggestion),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Type '{}' is not a struct or is not defined", name),
                    span,
                ))
            }
        }
        // Handle enum type constructors
        AstType::EnumType { name } => {
            if let Some(enum_info) = enums.get(name) {
                for (variant_name, _variant_type) in &enum_info.variants {
                    if variant_name == member {
                        // Return the enum type itself - the variant constructor creates an instance of the enum
                        let enum_variants = enum_info
                            .variants
                            .iter()
                            .map(|(name, payload)| crate::ast::EnumVariant {
                                name: name.clone(),
                                payload: payload.clone(),
                            })
                            .collect();
                        return Ok(AstType::Enum {
                            name: name.clone(),
                            variants: enum_variants,
                        });
                    }
                }
                let available_variants: Vec<String> = enum_info.variants.iter().map(|(n, _)| n.clone()).collect();
                let suggestion = if available_variants.is_empty() {
                    String::new()
                } else {
                    format!(". Available variants: {}", available_variants.join(", "))
                };
                Err(CompileError::TypeError(
                    format!("Enum '{}' has no variant '{}'{}", name, member, suggestion),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Unknown enum type: {}", name),
                    span,
                ))
            }
        }
        AstType::StdModule => {
            use crate::stdlib_types::stdlib_types;
            let registry = stdlib_types();

            if let Some(return_type) = registry.get_method_return_type(member, "init") {
                return Ok(return_type.clone());
            }

            if let Some(struct_type) = registry.get_struct_type(member) {
                return Ok(struct_type);
            }

            // Fallback: stdlib module member returns a generic marker type
            // that will be resolved at later compilation stages
            Ok(AstType::Generic {
                name: format!("StdModule::{}", member),
                type_args: vec![],
            })
        }
        _ => Err(CompileError::TypeError(
            format!(
                "Cannot access member '{}' on type {}: member access requires a struct, enum, or pointer type",
                member, object_type
            ),
            span,
        )),
    }
}

/// Infer the type of a struct field access
pub fn infer_struct_field_type(
    struct_type: &AstType,
    field: &str,
    structs: &HashMap<String, StructInfo>,
    enums: &HashMap<String, EnumInfo>,
    span: Option<crate::error::Span>,
) -> Result<AstType> {
    match struct_type {
        t if t.is_ptr_type() => {
            if let Some(inner) = t.ptr_inner() {
                match inner {
                    AstType::Struct { name, .. } => infer_member_type(
                        &AstType::Struct {
                            name: name.clone(),
                            fields: vec![],
                        },
                        field,
                        structs,
                        enums,
                        span,
                    ),
                    AstType::Generic { name, .. } => infer_member_type(
                        &AstType::Generic {
                            name: name.clone(),
                            type_args: vec![],
                        },
                        field,
                        structs,
                        enums,
                        span,
                    ),
                    _ => Err(CompileError::TypeError(
                        format!("Cannot access field '{}' on pointer to {}: field access requires a pointer to a struct", field, inner),
                        span,
                    )),
                }
            } else {
                Err(CompileError::TypeError(
                    format!("Cannot access field '{}' on type {}: invalid pointer type", field, struct_type),
                    span,
                ))
            }
        }
        AstType::Struct { .. } | AstType::Generic { .. } => {
            infer_member_type(struct_type, field, structs, enums, span)
        }
        _ => Err(CompileError::TypeError(
            format!("Cannot access field '{}' on type {}: field access requires a struct or pointer to a struct", field, struct_type),
            span,
        )),
    }
}
