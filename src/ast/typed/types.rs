use crate::ast::types::BuiltinTypeName;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
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

    Str, // static string view over baked program storage: { ptr, len }

    Named(String),

    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },

    Enum {
        name: String,
        variants: Vec<(String, Option<Type>)>,
    },

    Array {
        elem: Box<Type>,
        size: Option<usize>,
    },
    Slice(Box<Type>),

    Ptr(Box<Type>),
    MutPtr(Box<Type>),
    RawPtr(Box<Type>),

    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    /// The result of calling an `@async` function returning `T`: a suspendable
    /// computation that, when driven to completion, yields a `T`. Produced by an
    /// async call, consumed (unwrapped to `T`) by `@await`. See ASYNC_PLAN.md
    /// milestone 1.
    Future(Box<Type>),

    Never,
    Unknown,
}

impl Type {
    pub fn nominal_name(&self) -> Option<&str> {
        match self {
            Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => Some(name),
            _ => None,
        }
    }

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
            Type::Named(_)
            | Type::Struct { .. }
            | Type::Enum { .. }
            | Type::Array { .. }
            | Type::Slice(_)
            | Type::Ptr(_)
            | Type::MutPtr(_)
            | Type::RawPtr(_)
            | Type::Function { .. }
            | Type::Future(_)
            | Type::Never
            | Type::Unknown => return None,
        })
    }

    pub fn display_name(&self) -> String {
        if let Some(name) = self.builtin_source_name() {
            return name.into();
        }

        match self {
            Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => name.clone(),
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
            Type::Future(inner) => format!("Future<{}>", inner.display_name()),
            Type::Never => "!".into(),
            Type::Unknown => "?".into(),
            _ => unreachable!("handled by builtin_source_name"),
        }
    }

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

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }
}
