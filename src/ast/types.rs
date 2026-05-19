use crate::error::Span;
use serde::Serialize;

mod gated;
mod names;

pub use gated::{
    gated_builtin_type_name, is_builtin_type_name, GatedBuiltinType, ACTOR_REF_TYPE_NAME,
    ACTOR_TYPE_NAME, ALLOCATOR_TYPE_NAME, ASYNC_EFFECT_TYPE_NAME, DYNAMIC_STRING_TYPE_NAME,
    MAILBOX_TYPE_NAME, SUPERVISOR_TYPE_NAME, SYNC_EFFECT_TYPE_NAME,
};
pub use names::{BuiltinGenericTypeName, BuiltinTypeName, STATIC_STRING_TYPE_NAME};

/// Parser-level type representation.
///
/// These types may be unresolved — `Named("Point")` hasn't been looked up yet,
/// `Inferred` means the typechecker must figure it out. The typechecker resolves
/// these into fully concrete `Type` values (see `typed.rs`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AstType {
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

    // Other primitives
    Bool,
    Void,

    // Strings
    Str,    // static string view over baked program storage: { ptr, len }
    String, // allocator-backed dynamic string: { ptr, len, cap, allocator }

    // User-defined / unresolved named type
    Named(std::string::String),

    // Generic type application: `Channel<SensorReading>`, `Result<T, E>`
    Generic {
        name: std::string::String,
        type_args: Vec<AstType>,
    },

    // Collections
    Array {
        elem: Box<AstType>,
        size: Option<usize>,
    },
    Slice(Box<AstType>),

    // Pointers
    Ptr(Box<AstType>),
    MutPtr(Box<AstType>),
    RawPtr(Box<AstType>),

    // Function type: `(i32, i32) i32`
    Function {
        params: Vec<AstType>,
        ret: Box<AstType>,
    },

    // Self type — used in method signatures and behavior definitions
    SelfType,

    // Type to be inferred by the typechecker
    Inferred,
}

impl AstType {
    /// Returns the span-free name for display/error messages.
    pub fn display_name(&self) -> std::string::String {
        if let Some(builtin) = BuiltinTypeName::from_ast_type(self) {
            return builtin.to_string();
        }

        match self {
            AstType::String => DYNAMIC_STRING_TYPE_NAME.into(),
            AstType::Named(n) => n.clone(),
            AstType::Generic { name, type_args } => {
                let args: Vec<_> = type_args.iter().map(|a| a.display_name()).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            AstType::Array { elem, size } => match size {
                Some(n) => format!("[{}; {}]", elem.display_name(), n),
                None => format!("[{}]", elem.display_name()),
            },
            AstType::Slice(elem) => format!("[{}]", elem.display_name()),
            AstType::Ptr(inner) => format!("Ptr<{}>", inner.display_name()),
            AstType::MutPtr(inner) => format!("MutPtr<{}>", inner.display_name()),
            AstType::RawPtr(inner) => format!("RawPtr<{}>", inner.display_name()),
            AstType::Function { params, ret } => {
                let ps: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) {}", ps.join(", "), ret.display_name())
            }
            AstType::Inferred => "_".into(),
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::SelfType => unreachable!("handled by BuiltinTypeName"),
        }
    }
}

/// A typed parameter in a function/method/closure signature.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Param {
    pub name: std::string::String,
    pub ty: AstType,
    pub mutable: bool,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::AstType;

    #[test]
    fn static_string_display_uses_public_type_name() {
        assert_eq!(AstType::Str.display_name(), "StaticString");
    }
}
