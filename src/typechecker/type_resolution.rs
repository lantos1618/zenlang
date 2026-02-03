//! Type resolution - converting Generic types to Struct types, handling forward references

use crate::ast::AstType;
use crate::typechecker::TypeChecker;
use std::collections::HashSet;

/// Resolve Generic types to Struct types if they're known structs
/// This handles the case where the parser represents struct types as Generic
/// Recursively resolves nested Generic types in fields
/// Uses a visited set to prevent infinite recursion on circular references
pub fn resolve_generic_to_struct(checker: &TypeChecker, ast_type: &AstType) -> AstType {
    resolve_generic_to_struct_impl(checker, ast_type, &mut HashSet::new())
}

fn resolve_generic_to_struct_impl(
    checker: &TypeChecker,
    ast_type: &AstType,
    visited: &mut HashSet<String>,
) -> AstType {
    match ast_type {
        AstType::Generic { name, type_args } if type_args.is_empty() => {
            // Check if this Generic is actually a known struct
            if checker.structs.contains_key(name) {
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
                if let Some(struct_info) = checker.structs.get(name) {
                    let resolved_fields: Vec<(String, AstType)> = struct_info
                        .fields
                        .iter()
                        .map(|(field_name, field_type)| {
                            (
                                field_name.clone(),
                                resolve_generic_to_struct_impl(checker, field_type, visited),
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
                .map(|arg| resolve_generic_to_struct_impl(checker, arg, visited))
                .collect();
            AstType::Generic {
                name: name.clone(),
                type_args: resolved_args,
            }
        }
        t if t.is_immutable_ptr() => {
            let inner = t.ptr_inner().expect("immutable ptr should have inner type");
            AstType::ptr(resolve_generic_to_struct_impl(checker, inner, visited))
        }
        t if t.is_mutable_ptr() => {
            let inner = t.ptr_inner().expect("mutable ptr should have inner type");
            AstType::mut_ptr(resolve_generic_to_struct_impl(checker, inner, visited))
        }
        t if t.is_raw_ptr() => {
            let inner = t.ptr_inner().expect("raw ptr should have inner type");
            AstType::raw_ptr(resolve_generic_to_struct_impl(checker, inner, visited))
        }
        AstType::Struct { name, fields } => {
            // Recursively resolve fields
            let resolved_fields: Vec<(String, AstType)> = fields
                .iter()
                .map(|(field_name, field_type)| {
                    (
                        field_name.clone(),
                        resolve_generic_to_struct_impl(checker, field_type, visited),
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
                .map(|t| resolve_generic_to_struct_impl(checker, t, visited))
                .collect();
            AstType::FunctionPointer {
                param_types: resolved_params,
                return_type: Box::new(resolve_generic_to_struct_impl(
                    checker,
                    return_type,
                    visited,
                )),
            }
        }
        _ => ast_type.clone(),
    }
}
