use crate::error::Span;
use serde::Serialize;

pub const STATIC_STRING_TYPE_NAME: &str = "StaticString";
pub const DYNAMIC_STRING_TYPE_NAME: &str = "String";
pub const ALLOCATOR_TYPE_NAME: &str = "Allocator";
pub const SYNC_EFFECT_TYPE_NAME: &str = "Sync";
pub const ASYNC_EFFECT_TYPE_NAME: &str = "Async";
pub const ACTOR_TYPE_NAME: &str = "Actor";
pub const ACTOR_REF_TYPE_NAME: &str = "ActorRef";
pub const MAILBOX_TYPE_NAME: &str = "Mailbox";
pub const SUPERVISOR_TYPE_NAME: &str = "Supervisor";

pub fn is_builtin_type_name(name: &str) -> bool {
    name == DYNAMIC_STRING_TYPE_NAME
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatedBuiltinType {
    Allocator,
    SyncEffect,
    AsyncEffect,
    Actor,
    ActorRef,
    Mailbox,
    Supervisor,
}

impl GatedBuiltinType {
    pub const ALL: &[GatedBuiltinType] = &[
        Self::Allocator,
        Self::SyncEffect,
        Self::AsyncEffect,
        Self::Actor,
        Self::ActorRef,
        Self::Mailbox,
        Self::Supervisor,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        GatedBuiltinType::ALL
            .iter()
            .copied()
            .find(|ty| ty.as_str() == name)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allocator => ALLOCATOR_TYPE_NAME,
            Self::SyncEffect => SYNC_EFFECT_TYPE_NAME,
            Self::AsyncEffect => ASYNC_EFFECT_TYPE_NAME,
            Self::Actor => ACTOR_TYPE_NAME,
            Self::ActorRef => ACTOR_REF_TYPE_NAME,
            Self::Mailbox => MAILBOX_TYPE_NAME,
            Self::Supervisor => SUPERVISOR_TYPE_NAME,
        }
    }

    pub fn gate_message(self) -> &'static str {
        match self {
            Self::Allocator => {
                "typed allocators are gated until allocator ownership and effect semantics are implemented"
            }
            Self::SyncEffect => {
                "`Sync` effect mode is gated until Sync/Async effect checking is implemented"
            }
            Self::AsyncEffect => {
                "`Async` effect mode is gated until Sync/Async effect checking is implemented"
            }
            Self::Actor => {
                "`Actor` framework type is gated until std actor scheduling and mailbox semantics are implemented"
            }
            Self::ActorRef => {
                "`ActorRef` framework type is gated until std actor scheduling and mailbox semantics are implemented"
            }
            Self::Mailbox => {
                "`Mailbox` framework type is gated until std actor scheduling and mailbox semantics are implemented"
            }
            Self::Supervisor => {
                "`Supervisor` framework type is gated until std actor scheduling and mailbox semantics are implemented"
            }
        }
    }
}

pub fn gated_builtin_type_name(name: &str) -> Option<GatedBuiltinType> {
    GatedBuiltinType::from_name(name)
}

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
        match self {
            AstType::I8 => "i8".into(),
            AstType::I16 => "i16".into(),
            AstType::I32 => "i32".into(),
            AstType::I64 => "i64".into(),
            AstType::U8 => "u8".into(),
            AstType::U16 => "u16".into(),
            AstType::U32 => "u32".into(),
            AstType::U64 => "u64".into(),
            AstType::Usize => "usize".into(),
            AstType::F32 => "f32".into(),
            AstType::F64 => "f64".into(),
            AstType::Bool => "bool".into(),
            AstType::Void => "void".into(),
            AstType::Str => STATIC_STRING_TYPE_NAME.into(),
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
            AstType::SelfType => "Self".into(),
            AstType::Inferred => "_".into(),
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
