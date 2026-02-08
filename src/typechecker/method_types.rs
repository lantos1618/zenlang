//! Method call type inference
//! Handles type checking for method calls on various types
//!
//! ARCHITECTURE VIOLATION WARNING:
//! This module contains hardcoded type inference for Layer 3 stdlib types
//! (HashMap, HashSet, Vec, DynVec) which should NOT require compiler support.
//! See calls.rs for detailed explanation and Phase 5 fix plan.

use crate::ast::AstType;
use crate::well_known::well_known;

pub fn infer_string_method_type(method: &str, is_string_struct: bool) -> Option<AstType> {
    let wk = well_known();
    match method {
        "len" => Some(AstType::Usize),
        "to_i32" => Some(AstType::Generic {
            name: wk.option_name().to_string(),
            type_args: vec![AstType::I32],
        }),
        "to_i64" => Some(AstType::Generic {
            name: wk.option_name().to_string(),
            type_args: vec![AstType::I64],
        }),
        "to_f32" => Some(AstType::Generic {
            name: wk.option_name().to_string(),
            type_args: vec![AstType::F32],
        }),
        "to_f64" => Some(AstType::Generic {
            name: wk.option_name().to_string(),
            type_args: vec![AstType::F64],
        }),
        "substr" => {
            // substr returns same type as input (static stays static, dynamic stays dynamic)
            Some(if is_string_struct {
                crate::ast::resolve_string_struct_type()
            } else {
                AstType::StaticString
            })
        }
        "char_at" => Some(AstType::I32),
        "split" => {
            // split returns array of same string type as input
            let string_type = if is_string_struct {
                crate::ast::resolve_string_struct_type()
            } else {
                AstType::StaticString
            };
            Some(AstType::Generic {
                name: "Array".to_string(),
                type_args: vec![string_type],
            })
        }
        "trim" | "to_upper" | "to_lower" => {
            // returns same type as input
            Some(if is_string_struct {
                crate::ast::resolve_string_struct_type()
            } else {
                AstType::StaticString
            })
        }
        "contains" | "starts_with" | "ends_with" => Some(AstType::Bool),
        "index_of" => Some(AstType::Usize),
        _ => None,
    }
}

/// ARCHITECTURE VIOLATION: Hardcoded type inference for HashMap (Layer 3 type)
///
/// HashMap is a Layer 3 stdlib type that should NOT need compiler support.
/// This function exists because the trait system (Phase 5) is not yet implemented.
/// Once traits are available, this should be replaced with trait method resolution.
///
/// Methods hardcoded here: size, len, is_empty, clear, contains, remove, get, insert
pub fn infer_hashmap_method_type(method: &str, type_args: &[AstType]) -> Option<AstType> {
    let wk = well_known();
    match method {
        "size" | "len" => Some(AstType::Usize),
        "is_empty" => Some(AstType::Bool),
        "clear" => Some(AstType::Void),
        "contains" => Some(AstType::Bool),
        "remove" => {
            // HashMap.remove() returns Option<V>
            if type_args.len() >= 2 {
                Some(AstType::Generic {
                    name: wk.option_name().to_string(),
                    type_args: vec![type_args[1].clone()],
                })
            } else {
                None
            }
        }
        "get" => {
            // HashMap.get() returns Option<V>
            if type_args.len() >= 2 {
                Some(AstType::Generic {
                    name: wk.option_name().to_string(),
                    type_args: vec![type_args[1].clone()],
                })
            } else {
                None
            }
        }
        "insert" => Some(AstType::Void),
        _ => None,
    }
}

/// ARCHITECTURE VIOLATION: Hardcoded type inference for HashSet (Layer 3 type)
///
/// HashSet is a Layer 3 stdlib type that should NOT need compiler support.
/// This function exists because the trait system (Phase 5) is not yet implemented.
/// Once traits are available, this should be replaced with trait method resolution.
///
/// Methods hardcoded here: size, len, is_empty, clear, contains, insert, remove
pub fn infer_hashset_method_type(method: &str) -> Option<AstType> {
    match method {
        "size" | "len" => Some(AstType::Usize),
        "is_empty" => Some(AstType::Bool),
        "clear" => Some(AstType::Void),
        "contains" => Some(AstType::Bool),
        "insert" => Some(AstType::Bool),
        "remove" => Some(AstType::Bool),
        _ => None,
    }
}

/// ARCHITECTURE VIOLATION: Hardcoded type inference for Vec/DynVec (Layer 3 types)
///
/// Vec and DynVec are Layer 3 stdlib types that should NOT need compiler support.
/// This function exists because the trait system (Phase 5) is not yet implemented.
/// Once traits are available, this should be replaced with trait method resolution.
///
/// Methods hardcoded here: get, pop, len, capacity, push, set, clear
pub fn infer_vec_method_type(method: &str, element_type: &AstType) -> Option<AstType> {
    let wk = well_known();
    match method {
        "get" | "pop" => Some(AstType::Generic {
            name: wk.option_name().to_string(),
            type_args: vec![element_type.clone()],
        }),
        "len" | "capacity" => Some(AstType::Usize),
        "push" | "set" | "clear" => Some(AstType::Void),
        _ => None,
    }
}

/// Infer return type for pointer methods (Ptr<T>, MutPtr<T>, RawPtr<T>)
pub fn infer_pointer_method_type(method: &str, inner_type: &AstType) -> Option<AstType> {
    match method {
        "val" => Some(inner_type.clone()), // Dereference pointer
        "addr" => Some(AstType::raw_ptr(AstType::U8)), // Get address as RawPtr<u8>
        "is_some" | "is_none" => Some(AstType::Bool), // Check if pointer is valid/null
        "at" => Some(AstType::ptr(inner_type.clone())), // Get pointer at index offset
        "eq" => Some(AstType::Bool),       // Compare pointer addresses
        "unwrap" => Some(AstType::raw_ptr(AstType::U8)), // Unsafe unwrap to RawPtr<u8>
        _ => None,
    }
}

/// Infer return type for Result methods
pub fn infer_result_method_type(method: &str, type_args: &[AstType]) -> Option<AstType> {
    match method {
        "raise" => {
            // The raise() method returns the Ok type (T) from Result<T,E>
            if !type_args.is_empty() {
                Some(type_args[0].clone())
            } else {
                Some(AstType::Void)
            }
        }
        _ => None,
    }
}
