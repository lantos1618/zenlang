use super::AstType;

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

const BUILTIN_TYPE_SPELLINGS: &[(BuiltinTypeName, &str)] = &[
    (BuiltinTypeName::I8, "i8"),
    (BuiltinTypeName::I16, "i16"),
    (BuiltinTypeName::I32, "i32"),
    (BuiltinTypeName::I64, "i64"),
    (BuiltinTypeName::U8, "u8"),
    (BuiltinTypeName::U16, "u16"),
    (BuiltinTypeName::U32, "u32"),
    (BuiltinTypeName::U64, "u64"),
    (BuiltinTypeName::Usize, "usize"),
    (BuiltinTypeName::F32, "f32"),
    (BuiltinTypeName::F64, "f64"),
    (BuiltinTypeName::Bool, "bool"),
    (BuiltinTypeName::Void, "void"),
    (BuiltinTypeName::Str, "str"),
    (BuiltinTypeName::StaticString, "StaticString"),
    (BuiltinTypeName::SelfType, "Self"),
];

const BUILTIN_GENERIC_TYPE_SPELLINGS: &[(BuiltinGenericTypeName, &str)] = &[
    (BuiltinGenericTypeName::Ptr, "Ptr"),
    (BuiltinGenericTypeName::MutPtr, "MutPtr"),
    (BuiltinGenericTypeName::RawPtr, "RawPtr"),
    (BuiltinGenericTypeName::Slice, "Slice"),
];

impl BuiltinTypeName {
    pub fn as_str(self) -> &'static str {
        crate::static_spelling::static_spelling(BUILTIN_TYPE_SPELLINGS, self)
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

crate::static_spelling::impl_static_spelling_display!(
    BuiltinTypeName,
    table = BUILTIN_TYPE_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuiltinTypeName,
    table = BUILTIN_TYPE_SPELLINGS
);

impl BuiltinGenericTypeName {
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

crate::static_spelling::impl_static_spelling_display!(
    BuiltinGenericTypeName,
    table = BUILTIN_GENERIC_TYPE_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuiltinGenericTypeName,
    table = BUILTIN_GENERIC_TYPE_SPELLINGS
);
