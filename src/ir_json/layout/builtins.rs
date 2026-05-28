use std::collections::BTreeMap;

use crate::ast::typed::Type;

use super::metrics::{scalar_layout, POINTER_ALIGN, POINTER_SIZE, USIZE_SIZE};
use super::schema::LayoutJsonType;

pub(super) fn seed_builtin_layouts(layouts: &mut BTreeMap<String, LayoutJsonType>) {
    for ty in BUILTIN_LAYOUT_TYPES {
        let name = ty
            .builtin_source_name()
            .expect("builtin layout source types have source names");
        layouts.insert(name.into(), layout_for_builtin_type(ty));
    }
}

pub(super) fn builtin_layout_name(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Never | Type::Unknown => Type::Void.builtin_source_name(),
        _ => ty.builtin_source_name(),
    }
}

const BUILTIN_LAYOUT_TYPES: &[Type] = &[
    Type::Bool,
    Type::I8,
    Type::U8,
    Type::I16,
    Type::U16,
    Type::I32,
    Type::U32,
    Type::F32,
    Type::I64,
    Type::U64,
    Type::Usize,
    Type::F64,
    Type::Void,
    Type::Str,
];

fn layout_for_builtin_type(ty: &Type) -> LayoutJsonType {
    let (kind, size, alignment) = match ty {
        Type::Bool | Type::I8 | Type::U8 => ("primitive", 1, 1),
        Type::I16 | Type::U16 => ("primitive", 2, 2),
        Type::I32 | Type::U32 | Type::F32 => ("primitive", 4, 4),
        Type::I64 | Type::U64 | Type::F64 => ("primitive", 8, 8),
        Type::Usize => ("primitive", USIZE_SIZE, POINTER_ALIGN),
        Type::Void => ("primitive", 0, 1),
        Type::Str => ("static_string", POINTER_SIZE + USIZE_SIZE, POINTER_ALIGN),
        _ => unreachable!("non-builtin types are not seeded as builtin layouts"),
    };
    scalar_layout(kind, size, alignment)
}
