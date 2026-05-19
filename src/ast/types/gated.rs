//! Gated builtin type names and diagnostics.

pub const DYNAMIC_STRING_TYPE_NAME: &str = "String";
pub const ALLOCATOR_TYPE_NAME: &str = "Allocator";
pub const SYNC_EFFECT_TYPE_NAME: &str = "Sync";
pub const ASYNC_EFFECT_TYPE_NAME: &str = "Async";
pub const ACTOR_TYPE_NAME: &str = "Actor";
pub const ACTOR_REF_TYPE_NAME: &str = "ActorRef";
pub const MAILBOX_TYPE_NAME: &str = "Mailbox";
pub const SUPERVISOR_TYPE_NAME: &str = "Supervisor";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatedBuiltinType {
    DynamicString,
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
        Self::DynamicString,
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
            Self::DynamicString => DYNAMIC_STRING_TYPE_NAME,
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
            Self::DynamicString => {
                "`String` is gated until allocator-backed dynamic text ownership is implemented; use `StaticString` for baked literal text"
            }
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

pub fn is_builtin_type_name(name: &str) -> bool {
    matches!(
        gated_builtin_type_name(name),
        Some(GatedBuiltinType::DynamicString)
    )
}
