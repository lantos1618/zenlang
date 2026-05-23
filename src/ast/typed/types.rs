use crate::ast::types::{BuiltinTypeName, DYNAMIC_STRING_TYPE_NAME};
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
    Str,    // static string view over baked program storage: { ptr, len }
    String, // allocator-backed dynamic string: { ptr, len, cap, allocator }

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
    pub fn builtin_source_name(&self) -> Option<&'static str> {
        Some(match self {
            Type::I8 => BuiltinTypeName::I8.as_str(),
            Type::I16 => BuiltinTypeName::I16.as_str(),
            Type::I32 => BuiltinTypeName::I32.as_str(),
            Type::I64 => BuiltinTypeName::I64.as_str(),
            Type::U8 => BuiltinTypeName::U8.as_str(),
            Type::U16 => BuiltinTypeName::U16.as_str(),
            Type::U32 => BuiltinTypeName::U32.as_str(),
            Type::U64 => BuiltinTypeName::U64.as_str(),
            Type::Usize => BuiltinTypeName::Usize.as_str(),
            Type::F32 => BuiltinTypeName::F32.as_str(),
            Type::F64 => BuiltinTypeName::F64.as_str(),
            Type::Bool => BuiltinTypeName::Bool.as_str(),
            Type::Void => BuiltinTypeName::Void.as_str(),
            Type::Str => BuiltinTypeName::StaticString.as_str(),
            Type::String => DYNAMIC_STRING_TYPE_NAME,
            Type::Named(_)
            | Type::Struct { .. }
            | Type::Enum { .. }
            | Type::Array { .. }
            | Type::Slice(_)
            | Type::Ptr(_)
            | Type::MutPtr(_)
            | Type::RawPtr(_)
            | Type::Function { .. }
            | Type::Never
            | Type::Unknown => return None,
        })
    }

    pub fn display_name(&self) -> std::string::String {
        if let Some(name) = self.builtin_source_name() {
            return name.into();
        }

        match self {
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
            _ => unreachable!("handled by builtin_source_name"),
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

    #[test]
    fn builtin_source_names_cover_primitives_and_strings() {
        for (ty, name) in [
            (Type::I8, "i8"),
            (Type::I16, "i16"),
            (Type::I32, "i32"),
            (Type::I64, "i64"),
            (Type::U8, "u8"),
            (Type::U16, "u16"),
            (Type::U32, "u32"),
            (Type::U64, "u64"),
            (Type::Usize, "usize"),
            (Type::F32, "f32"),
            (Type::F64, "f64"),
            (Type::Bool, "bool"),
            (Type::Void, "void"),
            (Type::Str, "StaticString"),
            (Type::String, "String"),
        ] {
            assert_eq!(ty.builtin_source_name(), Some(name));
            assert_eq!(ty.display_name(), name);
        }
    }

    #[test]
    fn non_builtin_source_names_are_not_reported() {
        assert_eq!(Type::Named("Point".into()).builtin_source_name(), None);
        assert_eq!(Type::Never.builtin_source_name(), None);
        assert_eq!(Type::Unknown.builtin_source_name(), None);
    }
}
