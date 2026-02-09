use crate::ast::{AstType, TraitImplementation};
use crate::error::Result;

/// Transform Self types to concrete types in trait implementations.
/// Used by the compiler pipeline to resolve Self before codegen.
pub fn transform_trait_impl_self_types(
    trait_impl: &TraitImplementation,
) -> Result<TraitImplementation> {
    let mut transformed = trait_impl.clone();
    let concrete_type = &trait_impl.type_name;

    for method in &mut transformed.methods {
        // Transform parameter types
        for (_, param_type) in &mut method.args {
            *param_type = replace_self_in_ast_type(param_type, concrete_type);
        }
        // Transform return type
        method.return_type = replace_self_in_ast_type(&method.return_type, concrete_type);
    }

    Ok(transformed)
}

/// Replace Self in an AST type with a given replacement type.
/// This is the core recursive walk; callers control what Self becomes.
pub fn replace_self_with(ast_type: &AstType, replacement: &AstType) -> AstType {
    match ast_type {
        AstType::Generic { name, type_args: _ } if name == "Self" => replacement.clone(),
        t if t.is_immutable_ptr() => {
            if let Some(inner) = t.ptr_inner() {
                AstType::ptr(replace_self_with(inner, replacement))
            } else {
                ast_type.clone()
            }
        }
        t if t.is_mutable_ptr() => {
            if let Some(inner) = t.ptr_inner() {
                AstType::mut_ptr(replace_self_with(inner, replacement))
            } else {
                ast_type.clone()
            }
        }
        t if t.is_raw_ptr() => {
            if let Some(inner) = t.ptr_inner() {
                AstType::raw_ptr(replace_self_with(inner, replacement))
            } else {
                ast_type.clone()
            }
        }
        AstType::Slice(element) => {
            AstType::Slice(Box::new(replace_self_with(element, replacement)))
        }
        AstType::FixedArray { element_type, size } => AstType::FixedArray {
            element_type: Box::new(replace_self_with(element_type, replacement)),
            size: *size,
        },
        AstType::Function { args, return_type } => AstType::Function {
            args: args
                .iter()
                .map(|t| replace_self_with(t, replacement))
                .collect(),
            return_type: Box::new(replace_self_with(return_type, replacement)),
        },
        AstType::FunctionPointer {
            param_types,
            return_type,
        } => AstType::FunctionPointer {
            param_types: param_types
                .iter()
                .map(|t| replace_self_with(t, replacement))
                .collect(),
            return_type: Box::new(replace_self_with(return_type, replacement)),
        },
        // For other types, return as-is
        _ => ast_type.clone(),
    }
}

/// Replace Self in an AST type with a tagged Generic for codegen resolution
pub fn replace_self_in_ast_type(ast_type: &AstType, concrete_type: &str) -> AstType {
    let replacement = AstType::Generic {
        name: format!("Self_{}", concrete_type),
        type_args: vec![],
    };
    replace_self_with(ast_type, &replacement)
}
