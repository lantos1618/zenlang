use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

pub(super) fn type_mangle_key(ty: &Type) -> String {
    match ty {
        Type::I8 => "i8".into(),
        Type::I16 => "i16".into(),
        Type::I32 => "i32".into(),
        Type::I64 => "i64".into(),
        Type::U8 => "u8".into(),
        Type::U16 => "u16".into(),
        Type::U32 => "u32".into(),
        Type::U64 => "u64".into(),
        Type::Usize => "usize".into(),
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Void => "void".into(),
        Type::Str => "StaticString".into(),
        Type::String => "String".into(),
        Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
            symbol_mangle_key(name)
        }
        Type::Array { elem, size } => match size {
            Some(size) => format!("array_{}_{}", type_mangle_key(elem), size),
            None => format!("array_{}", type_mangle_key(elem)),
        },
        Type::Slice(elem) => format!("slice_{}", type_mangle_key(elem)),
        Type::Ptr(inner) => format!("ptr_{}", type_mangle_key(inner)),
        Type::MutPtr(inner) => format!("mutptr_{}", type_mangle_key(inner)),
        Type::RawPtr(inner) => format!("rawptr_{}", type_mangle_key(inner)),
        Type::Function { params, ret } => {
            let params = params
                .iter()
                .map(type_mangle_key)
                .collect::<Vec<_>>()
                .join("_");
            format!("fn_{}_ret_{}", params, type_mangle_key(ret))
        }
        Type::Never => "never".into(),
        Type::Unknown => "unknown".into(),
    }
}

fn symbol_mangle_key(symbol: &str) -> String {
    let mut out = String::with_capacity(symbol.len());
    for ch in symbol.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

pub(super) fn concrete_name_matches_generic(concrete_name: &str, generic_name: &str) -> bool {
    concrete_name == generic_name || concrete_name.starts_with(&format!("{generic_name}_"))
}

#[cfg(test)]
mod tests {
    use super::{type_mangle_key, Type};

    #[test]
    fn static_string_mangle_uses_public_type_name() {
        assert_eq!(type_mangle_key(&Type::Str), "StaticString");
    }
}

/// Substitute type parameters in an AstType, returning a new AstType.
pub(super) fn substitute_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => {
            if let Some(concrete) = substitutions.get(name) {
                type_to_ast(concrete)
            } else {
                ast_type.clone()
            }
        }
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_ast_type(inner, substitutions))),
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|a| substitute_ast_type(a, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

/// Convert a resolved Type back to an AstType (best-effort, for substitution).
pub(crate) fn type_to_ast(ty: &Type) -> AstType {
    match ty {
        Type::I8 => AstType::I8,
        Type::I16 => AstType::I16,
        Type::I32 => AstType::I32,
        Type::I64 => AstType::I64,
        Type::U8 => AstType::U8,
        Type::U16 => AstType::U16,
        Type::U32 => AstType::U32,
        Type::U64 => AstType::U64,
        Type::Usize => AstType::Usize,
        Type::F32 => AstType::F32,
        Type::F64 => AstType::F64,
        Type::Bool => AstType::Bool,
        Type::Void => AstType::Void,
        Type::Str => AstType::Str,
        Type::String => AstType::Named("String".into()),
        Type::Named(n) => AstType::Named(n.clone()),
        Type::Struct { name, .. } => AstType::Named(name.clone()),
        Type::Enum { name, .. } => AstType::Named(name.clone()),
        Type::Ptr(inner) => AstType::Ptr(Box::new(type_to_ast(inner))),
        Type::MutPtr(inner) => AstType::MutPtr(Box::new(type_to_ast(inner))),
        Type::RawPtr(inner) => AstType::RawPtr(Box::new(type_to_ast(inner))),
        Type::Slice(inner) => AstType::Slice(Box::new(type_to_ast(inner))),
        Type::Array { elem, size } => AstType::Array {
            elem: Box::new(type_to_ast(elem)),
            size: *size,
        },
        Type::Function { params, ret } => AstType::Function {
            params: params.iter().map(type_to_ast).collect(),
            ret: Box::new(type_to_ast(ret)),
        },
        Type::Never | Type::Unknown => AstType::Void,
    }
}
