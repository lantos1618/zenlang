//! Method call type inference
//! Handles type checking for method calls on various types

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
            // TODO: should come from TypeContext String::split return type
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

/// Infer return type for Vec<T, N> methods
pub fn infer_vec_method_type(method: &str, element_type: &AstType) -> Option<AstType> {
    let wk = well_known();
    match method {
        "get" => Some(element_type.clone()), // Returns element directly
        "pop" => Some(AstType::Generic {
            // Returns Option<element>
            name: wk.option_name().to_string(),
            type_args: vec![element_type.clone()],
        }),
        "len" | "capacity" => Some(AstType::Usize),
        "push" | "set" | "clear" => Some(AstType::Void),
        _ => None,
    }
}

/// Infer return type for pointer methods
pub fn infer_pointer_method_type(method: &str, inner_type: &AstType) -> Option<AstType> {
    match method {
        "val" => Some(inner_type.clone()), // Dereference pointer
        "addr" => Some(AstType::Usize),    // Get address as usize
        _ => None,
    }
}

/// Infer return type for Result methods
pub fn infer_result_method_type(method: &str, type_args: &[AstType]) -> Option<AstType> {
    match method {
        "raise" => {
            // The raise() method returns the Ok type (T) from Result<T,E>
            if type_args.len() >= 1 {
                Some(type_args[0].clone())
            } else {
                Some(AstType::Void)
            }
        }
        _ => None,
    }
}
