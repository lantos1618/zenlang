//! Type resolution - converting Generic types to Struct types, handling forward references

use crate::ast::AstType;
use crate::typechecker::TypeChecker;
use std::collections::HashSet;

/// Maximum recursion depth for type resolution to prevent stack overflow
const MAX_TYPE_RESOLUTION_DEPTH: usize = 256;

/// Resolve Generic types to Struct types if they're known structs
/// This handles the case where the parser represents struct types as Generic
/// Recursively resolves nested Generic types in fields
/// Uses a visited set to prevent infinite recursion on circular references
pub fn resolve_generic_to_struct(checker: &TypeChecker, ast_type: &AstType) -> AstType {
    resolve_generic_to_struct_impl(checker, ast_type, &mut HashSet::new(), 0)
}

fn resolve_generic_to_struct_impl(
    checker: &TypeChecker,
    ast_type: &AstType,
    visited: &mut HashSet<String>,
    depth: usize,
) -> AstType {
    if depth > MAX_TYPE_RESOLUTION_DEPTH {
        // Return type as-is to prevent stack overflow on pathologically deep types
        return ast_type.clone();
    }
    match ast_type {
        AstType::Generic { name, type_args } if type_args.is_empty() => {
            // Check if this Generic is actually a known struct
            if checker.type_store.borrow().has_struct(name) {
                // Prevent infinite recursion on circular references
                if visited.contains(name) {
                    // Return a Struct type without resolving fields to break the cycle
                    // This handles self-referential structs like Node { child: Ptr<Node> }
                    return AstType::Struct {
                        name: name.clone(),
                        fields: vec![], // Empty fields to break cycle
                    };
                }
                visited.insert(name.clone());

                // Convert to Struct type with recursively resolved fields
                let struct_fields: Option<Vec<(String, AstType)>> = checker
                    .type_store
                    .borrow()
                    .get_struct(name)
                    .map(|s| s.fields.clone());
                if let Some(fields) = struct_fields {
                    let resolved_fields: Vec<(String, AstType)> = fields
                        .iter()
                        .map(|(field_name, field_type)| {
                            (
                                field_name.clone(),
                                resolve_generic_to_struct_impl(
                                    checker,
                                    field_type,
                                    visited,
                                    depth + 1,
                                ),
                            )
                        })
                        .collect();
                    visited.remove(name);
                    AstType::Struct {
                        name: name.clone(),
                        fields: resolved_fields,
                    }
                } else {
                    visited.remove(name);
                    AstType::Struct {
                        name: name.clone(),
                        fields: vec![],
                    }
                }
            } else {
                // Not a struct, return as-is
                ast_type.clone()
            }
        }
        AstType::Generic { name, type_args } => {
            // Generic with type arguments - resolve each type arg
            let resolved_args: Vec<AstType> = type_args
                .iter()
                .map(|arg| resolve_generic_to_struct_impl(checker, arg, visited, depth + 1))
                .collect();
            AstType::Generic {
                name: name.clone(),
                type_args: resolved_args,
            }
        }
        t if t.is_immutable_ptr() => match t.ptr_inner() {
            Some(inner) => AstType::ptr(resolve_generic_to_struct_impl(
                checker,
                inner,
                visited,
                depth + 1,
            )),
            None => t.clone(),
        },
        t if t.is_mutable_ptr() => match t.ptr_inner() {
            Some(inner) => AstType::mut_ptr(resolve_generic_to_struct_impl(
                checker,
                inner,
                visited,
                depth + 1,
            )),
            None => t.clone(),
        },
        t if t.is_raw_ptr() => match t.ptr_inner() {
            Some(inner) => AstType::raw_ptr(resolve_generic_to_struct_impl(
                checker,
                inner,
                visited,
                depth + 1,
            )),
            None => t.clone(),
        },
        AstType::Struct { name, fields } => {
            // Recursively resolve fields
            let resolved_fields: Vec<(String, AstType)> = fields
                .iter()
                .map(|(field_name, field_type)| {
                    (
                        field_name.clone(),
                        resolve_generic_to_struct_impl(checker, field_type, visited, depth + 1),
                    )
                })
                .collect();
            AstType::Struct {
                name: name.clone(),
                fields: resolved_fields,
            }
        }
        AstType::FunctionPointer {
            param_types,
            return_type,
        } => {
            let resolved_params: Vec<AstType> = param_types
                .iter()
                .map(|t| resolve_generic_to_struct_impl(checker, t, visited, depth + 1))
                .collect();
            AstType::FunctionPointer {
                param_types: resolved_params,
                return_type: Box::new(resolve_generic_to_struct_impl(
                    checker,
                    return_type,
                    visited,
                    depth + 1,
                )),
            }
        }
        _ => ast_type.clone(),
    }
}
