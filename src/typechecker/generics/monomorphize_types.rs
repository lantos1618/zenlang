use std::collections::HashMap;

use super::super::ast_type_substitution::substitute_ast_type_names;
use crate::ast::typed::Type;
use crate::ast::{symbol_key_part, AstType};

pub(super) fn type_mangle_key(ty: &Type) -> String {
    if let Some(name) = ty.builtin_source_name() {
        return name.into();
    }
    if let Some(name) = ty.nominal_name() {
        return symbol_key_part(name);
    }

    match ty {
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
        _ => unreachable!("handled by builtin_source_name"),
    }
}

pub(crate) fn substitute_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    substitute_ast_type_names(ast_type, &|name| substitutions.get(name).map(type_to_ast))
}

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
        Type::Void | Type::Never | Type::Unknown => AstType::Void,
        Type::Str => AstType::Str,
        Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
            AstType::Named(name.clone())
        }
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
    }
}
