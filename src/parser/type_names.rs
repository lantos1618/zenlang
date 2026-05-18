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
        match value {
            Self::I8_NAME => Ok(Self::I8),
            Self::I16_NAME => Ok(Self::I16),
            Self::I32_NAME => Ok(Self::I32),
            Self::I64_NAME => Ok(Self::I64),
            Self::U8_NAME => Ok(Self::U8),
            Self::U16_NAME => Ok(Self::U16),
            Self::U32_NAME => Ok(Self::U32),
            Self::U64_NAME => Ok(Self::U64),
            Self::USIZE_NAME => Ok(Self::Usize),
            Self::F32_NAME => Ok(Self::F32),
            Self::F64_NAME => Ok(Self::F64),
            Self::BOOL_NAME => Ok(Self::Bool),
            Self::VOID_NAME => Ok(Self::Void),
            Self::STR_NAME => Ok(Self::Str),
            STATIC_STRING_TYPE_NAME => Ok(Self::StaticString),
            Self::SELF_NAME => Ok(Self::SelfType),
            _ => Err(()),
        }
    }
}

impl ParserBuiltinGenericTypeName {
    const PTR: &'static str = "Ptr";
    const MUT_PTR: &'static str = "MutPtr";
    const RAW_PTR: &'static str = "RawPtr";
    const SLICE: &'static str = "Slice";

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
        match value {
            Self::PTR => Ok(Self::Ptr),
            Self::MUT_PTR => Ok(Self::MutPtr),
            Self::RAW_PTR => Ok(Self::RawPtr),
            Self::SLICE => Ok(Self::Slice),
            _ => Err(()),
        }
    }
}
