//! Built-in comptime functions and meta intrinsics.
//!
//! These enums define the operations available during compile-time evaluation.
//! They live in `ast/` because they're part of the language definition — just
//! like `BinaryOperator` and `Expression`, not interpreter implementation details.

use std::fmt;

/// Compile-time built-in functions (sizeof, typeof, emit, comptime_assert).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFn {
    Sizeof,
    Typeof,
    Emit,
    ComptimeAssert,
}

impl BuiltinFn {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "sizeof" => Some(Self::Sizeof),
            "typeof" => Some(Self::Typeof),
            "emit" => Some(Self::Emit),
            "comptime_assert" => Some(Self::ComptimeAssert),
            _ => None,
        }
    }

    pub const ALL: &[BuiltinFn] = &[Self::Sizeof, Self::Typeof, Self::Emit, Self::ComptimeAssert];
}

impl fmt::Display for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sizeof => write!(f, "sizeof"),
            Self::Typeof => write!(f, "typeof"),
            Self::Emit => write!(f, "emit"),
            Self::ComptimeAssert => write!(f, "comptime_assert"),
        }
    }
}

/// Meta-programming intrinsics (meta.type_info, meta.fields, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaIntrinsic {
    TypeInfo,
    Fields,
    VariantName,
    Children,
    Parse,
}

impl MetaIntrinsic {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "type_info" => Some(Self::TypeInfo),
            "fields" => Some(Self::Fields),
            "variant_name" => Some(Self::VariantName),
            "children" => Some(Self::Children),
            "parse" => Some(Self::Parse),
            _ => None,
        }
    }

    pub const ALL: &[MetaIntrinsic] = &[
        Self::TypeInfo,
        Self::Fields,
        Self::VariantName,
        Self::Children,
        Self::Parse,
    ];
}

impl fmt::Display for MetaIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeInfo => write!(f, "type_info"),
            Self::Fields => write!(f, "fields"),
            Self::VariantName => write!(f, "variant_name"),
            Self::Children => write!(f, "children"),
            Self::Parse => write!(f, "parse"),
        }
    }
}
