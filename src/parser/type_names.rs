use crate::ast::types::{AstType, STATIC_STRING_TYPE_NAME};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserBuiltinTypeName {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
    Bool,
    Void,
    Str,
    StaticString,
    SelfType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserBuiltinGenericTypeName {
    Ptr,
    MutPtr,
    RawPtr,
    Slice,
}

impl ParserBuiltinTypeName {
    const ALL: &[ParserBuiltinTypeName] = &[
        ParserBuiltinTypeName::I8,
        ParserBuiltinTypeName::I16,
        ParserBuiltinTypeName::I32,
        ParserBuiltinTypeName::I64,
        ParserBuiltinTypeName::U8,
        ParserBuiltinTypeName::U16,
        ParserBuiltinTypeName::U32,
        ParserBuiltinTypeName::U64,
        ParserBuiltinTypeName::Usize,
        ParserBuiltinTypeName::F32,
        ParserBuiltinTypeName::F64,
        ParserBuiltinTypeName::Bool,
        ParserBuiltinTypeName::Void,
        ParserBuiltinTypeName::Str,
        ParserBuiltinTypeName::StaticString,
        ParserBuiltinTypeName::SelfType,
    ];
    const I8_NAME: &'static str = "i8";
    const I16_NAME: &'static str = "i16";
    const I32_NAME: &'static str = "i32";
    const I64_NAME: &'static str = "i64";
    const U8_NAME: &'static str = "u8";
    const U16_NAME: &'static str = "u16";
    const U32_NAME: &'static str = "u32";
    const U64_NAME: &'static str = "u64";
    const USIZE_NAME: &'static str = "usize";
    const F32_NAME: &'static str = "f32";
    const F64_NAME: &'static str = "f64";
    const BOOL_NAME: &'static str = "bool";
    const VOID_NAME: &'static str = "void";
    const STR_NAME: &'static str = "str";
    const SELF_NAME: &'static str = "Self";

    fn as_str(self) -> &'static str {
        match self {
            Self::I8 => Self::I8_NAME,
            Self::I16 => Self::I16_NAME,
            Self::I32 => Self::I32_NAME,
            Self::I64 => Self::I64_NAME,
            Self::U8 => Self::U8_NAME,
            Self::U16 => Self::U16_NAME,
            Self::U32 => Self::U32_NAME,
            Self::U64 => Self::U64_NAME,
            Self::Usize => Self::USIZE_NAME,
            Self::F32 => Self::F32_NAME,
            Self::F64 => Self::F64_NAME,
            Self::Bool => Self::BOOL_NAME,
            Self::Void => Self::VOID_NAME,
            Self::Str => Self::STR_NAME,
            Self::StaticString => STATIC_STRING_TYPE_NAME,
            Self::SelfType => Self::SELF_NAME,
        }
    }

    pub(super) fn ast_type(self) -> AstType {
        match self {
            Self::I8 => AstType::I8,
            Self::I16 => AstType::I16,
            Self::I32 => AstType::I32,
            Self::I64 => AstType::I64,
            Self::U8 => AstType::U8,
            Self::U16 => AstType::U16,
            Self::U32 => AstType::U32,
            Self::U64 => AstType::U64,
            Self::Usize => AstType::Usize,
            Self::F32 => AstType::F32,
            Self::F64 => AstType::F64,
            Self::Bool => AstType::Bool,
            Self::Void => AstType::Void,
            Self::Str | Self::StaticString => AstType::Str,
            Self::SelfType => AstType::SelfType,
        }
    }
}

impl FromStr for ParserBuiltinTypeName {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|name| name.as_str() == value)
            .ok_or(())
    }
}

impl ParserBuiltinGenericTypeName {
    const ALL: &[ParserBuiltinGenericTypeName] = &[
        ParserBuiltinGenericTypeName::Ptr,
        ParserBuiltinGenericTypeName::MutPtr,
        ParserBuiltinGenericTypeName::RawPtr,
        ParserBuiltinGenericTypeName::Slice,
    ];
    const PTR: &'static str = "Ptr";
    const MUT_PTR: &'static str = "MutPtr";
    const RAW_PTR: &'static str = "RawPtr";
    const SLICE: &'static str = "Slice";

    fn as_str(self) -> &'static str {
        match self {
            Self::Ptr => Self::PTR,
            Self::MutPtr => Self::MUT_PTR,
            Self::RawPtr => Self::RAW_PTR,
            Self::Slice => Self::SLICE,
        }
    }

    pub(super) fn ast_type(self, mut type_args: Vec<AstType>) -> Result<AstType, Vec<AstType>> {
        if type_args.len() != 1 {
            return Err(type_args);
        }
        let ty = Box::new(type_args.remove(0));
        Ok(match self {
            Self::Ptr => AstType::Ptr(ty),
            Self::MutPtr => AstType::MutPtr(ty),
            Self::RawPtr => AstType::RawPtr(ty),
            Self::Slice => AstType::Slice(ty),
        })
    }
}

impl FromStr for ParserBuiltinGenericTypeName {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|name| name.as_str() == value)
            .ok_or(())
    }
}
