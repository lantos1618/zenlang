use std::collections::BTreeMap;

use crate::ast::typed::Type;

use super::metrics::{scalar_layout, POINTER_ALIGN, POINTER_SIZE, USIZE_SIZE};
use super::schema::LayoutJsonType;

pub(super) fn seed_builtin_layouts(layouts: &mut BTreeMap<String, LayoutJsonType>) {
    for builtin in BuiltinLayout::ALL {
        layouts.insert(builtin.name().into(), builtin.layout());
    }
}

pub(super) fn builtin_layout_name(ty: &Type) -> Option<&'static str> {
    BuiltinLayout::from_type(ty).map(BuiltinLayout::name)
}

#[derive(Clone, Copy)]
enum BuiltinLayout {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    I64,
    U64,
    Usize,
    F64,
    Void,
    StaticString,
    DynamicString,
}

impl BuiltinLayout {
    const ALL: &[Self] = &[
        Self::Bool,
        Self::I8,
        Self::U8,
        Self::I16,
        Self::U16,
        Self::I32,
        Self::U32,
        Self::F32,
        Self::I64,
        Self::U64,
        Self::Usize,
        Self::F64,
        Self::Void,
        Self::StaticString,
        Self::DynamicString,
    ];

    fn from_type(ty: &Type) -> Option<Self> {
        match ty {
            Type::I8 => Some(Self::I8),
            Type::I16 => Some(Self::I16),
            Type::I32 => Some(Self::I32),
            Type::I64 => Some(Self::I64),
            Type::U8 => Some(Self::U8),
            Type::U16 => Some(Self::U16),
            Type::U32 => Some(Self::U32),
            Type::U64 => Some(Self::U64),
            Type::Usize => Some(Self::Usize),
            Type::F32 => Some(Self::F32),
            Type::F64 => Some(Self::F64),
            Type::Bool => Some(Self::Bool),
            Type::Void | Type::Never | Type::Unknown => Some(Self::Void),
            Type::Str => Some(Self::StaticString),
            Type::String => Some(Self::DynamicString),
            Type::Named(_)
            | Type::Struct { .. }
            | Type::Enum { .. }
            | Type::Array { .. }
            | Type::Slice(_)
            | Type::Ptr(_)
            | Type::MutPtr(_)
            | Type::RawPtr(_)
            | Type::Function { .. } => None,
        }
    }

    fn name(self) -> &'static str {
        self.source_type()
            .builtin_source_name()
            .expect("builtin layout source types have source names")
    }

    fn source_type(self) -> Type {
        match self {
            Self::Bool => Type::Bool,
            Self::I8 => Type::I8,
            Self::U8 => Type::U8,
            Self::I16 => Type::I16,
            Self::U16 => Type::U16,
            Self::I32 => Type::I32,
            Self::U32 => Type::U32,
            Self::F32 => Type::F32,
            Self::I64 => Type::I64,
            Self::U64 => Type::U64,
            Self::Usize => Type::Usize,
            Self::F64 => Type::F64,
            Self::Void => Type::Void,
            Self::StaticString => Type::Str,
            Self::DynamicString => Type::String,
        }
    }

    fn layout(self) -> LayoutJsonType {
        scalar_layout(self.kind(), self.size(), self.alignment())
    }

    fn kind(self) -> &'static str {
        match self {
            Self::StaticString => "static_string",
            Self::DynamicString => "dynamic_string",
            _ => "primitive",
        }
    }

    fn size(self) -> u32 {
        match self {
            Self::Bool | Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
            Self::Usize => USIZE_SIZE,
            Self::Void => 0,
            Self::StaticString => POINTER_SIZE + USIZE_SIZE,
            Self::DynamicString => POINTER_SIZE + USIZE_SIZE * 2 + POINTER_SIZE,
        }
    }

    fn alignment(self) -> u32 {
        match self {
            Self::Bool | Self::I8 | Self::U8 | Self::Void => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
            Self::Usize | Self::StaticString | Self::DynamicString => POINTER_ALIGN,
        }
    }
}
