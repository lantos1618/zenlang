use super::AstType;
use std::fmt;
use std::str::FromStr;

pub const STATIC_STRING_TYPE_NAME: &str = "StaticString";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTypeName {
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
pub enum BuiltinGenericTypeName {
    Ptr,
    MutPtr,
    RawPtr,
    Slice,
}

impl BuiltinTypeName {
    pub const ALL: &[BuiltinTypeName] = &[
        BuiltinTypeName::I8,
        BuiltinTypeName::I16,
        BuiltinTypeName::I32,
        BuiltinTypeName::I64,
        BuiltinTypeName::U8,
        BuiltinTypeName::U16,
        BuiltinTypeName::U32,
        BuiltinTypeName::U64,
        BuiltinTypeName::Usize,
        BuiltinTypeName::F32,
        BuiltinTypeName::F64,
        BuiltinTypeName::Bool,
        BuiltinTypeName::Void,
        BuiltinTypeName::Str,
        BuiltinTypeName::StaticString,
        BuiltinTypeName::SelfType,
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

    pub fn as_str(self) -> &'static str {
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

    pub fn ast_type(self) -> AstType {
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

    pub fn from_ast_type(ast_type: &AstType) -> Option<Self> {
        match ast_type {
            AstType::I8 => Some(Self::I8),
            AstType::I16 => Some(Self::I16),
            AstType::I32 => Some(Self::I32),
            AstType::I64 => Some(Self::I64),
            AstType::U8 => Some(Self::U8),
            AstType::U16 => Some(Self::U16),
            AstType::U32 => Some(Self::U32),
            AstType::U64 => Some(Self::U64),
            AstType::Usize => Some(Self::Usize),
            AstType::F32 => Some(Self::F32),
            AstType::F64 => Some(Self::F64),
            AstType::Bool => Some(Self::Bool),
            AstType::Void => Some(Self::Void),
            AstType::Str => Some(Self::StaticString),
            AstType::SelfType => Some(Self::SelfType),
            _ => None,
        }
    }
}

impl fmt::Display for BuiltinTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuiltinTypeName {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|name| name.as_str() == value)
            .ok_or(())
    }
}

impl BuiltinGenericTypeName {
    pub const ALL: &[BuiltinGenericTypeName] = &[
        BuiltinGenericTypeName::Ptr,
        BuiltinGenericTypeName::MutPtr,
        BuiltinGenericTypeName::RawPtr,
        BuiltinGenericTypeName::Slice,
    ];
    const PTR: &'static str = "Ptr";
    const MUT_PTR: &'static str = "MutPtr";
    const RAW_PTR: &'static str = "RawPtr";
    const SLICE: &'static str = "Slice";

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ptr => Self::PTR,
            Self::MutPtr => Self::MUT_PTR,
            Self::RawPtr => Self::RAW_PTR,
            Self::Slice => Self::SLICE,
        }
    }

    pub fn ast_type(self, mut type_args: Vec<AstType>) -> Result<AstType, Vec<AstType>> {
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

impl fmt::Display for BuiltinGenericTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuiltinGenericTypeName {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|name| name.as_str() == value)
            .ok_or(())
    }
}
