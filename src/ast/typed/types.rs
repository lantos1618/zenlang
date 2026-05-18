use crate::ast::types::{DYNAMIC_STRING_TYPE_NAME, STATIC_STRING_TYPE_NAME};
use serde::Serialize;

// Fully resolved types — no generics, no inference variables.
// This is the typechecker's output; codegen only sees these.

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
    // Integers
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,

    // Floats
    F32,
    F64,

    // Primitives
    Bool,
    Void,

    // Strings
    Str,    // static string: { ptr, len }
    String, // heap string: { ptr, len, cap, alloc }

    // Named type — struct/enum by mangled name
    Named(std::string::String),

    // Struct with known fields (inline definition)
    Struct {
        name: std::string::String,
        fields: Vec<(std::string::String, Type)>,
    },

    // Enum with known variants
    Enum {
        name: std::string::String,
        variants: Vec<(std::string::String, Option<Type>)>,
    },

    // Collections
    Array {
        elem: Box<Type>,
        size: Option<usize>,
    },
    Slice(Box<Type>),

    // Pointers
    Ptr(Box<Type>),
    MutPtr(Box<Type>),
    RawPtr(Box<Type>),

    // Function pointer type
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    // Diverging expression (return, break, infinite loop)
    Never,

    // Unresolved / error — sema couldn't determine the type
    Unknown,
}

impl Type {
    pub fn display_name(&self) -> std::string::String {
        match self {
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
            Type::Str => STATIC_STRING_TYPE_NAME.into(),
            Type::String => DYNAMIC_STRING_TYPE_NAME.into(),
            Type::Named(n) => n.clone(),
            Type::Struct { name, .. } => name.clone(),
            Type::Enum { name, .. } => name.clone(),
            Type::Array { elem, size } => match size {
                Some(n) => format!("[{}; {}]", elem.display_name(), n),
                None => format!("[{}]", elem.display_name()),
            },
            Type::Slice(elem) => format!("[{}]", elem.display_name()),
            Type::Ptr(inner) => format!("Ptr<{}>", inner.display_name()),
            Type::MutPtr(inner) => format!("MutPtr<{}>", inner.display_name()),
            Type::RawPtr(inner) => format!("RawPtr<{}>", inner.display_name()),
            Type::Function { params, ret } => {
                let ps: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) {}", ps.join(", "), ret.display_name())
            }
            Type::Never => "!".into(),
            Type::Unknown => "?".into(),
        }
    }

    /// Returns true if this type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::F32
                | Type::F64
        )
    }

    /// Returns true if this type is an integer.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
        )
    }

    /// Returns true if this type is a float.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    fn static_string_display_uses_public_type_name() {
        assert_eq!(Type::Str.display_name(), "StaticString");
    }
}
