#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedStdlibModule {
    ActorFramework,
    AllocatorFramework,
    AsyncRuntime,
    CompilerFacade,
    SyncRuntime,
    IoUringRuntime,
}

impl GatedStdlibModule {
    const CONCURRENCY_SEGMENT: &'static str = "concurrency";
    const ACTOR_SEGMENT: &'static str = "actor";
    const ASYNC_SEGMENT: &'static str = "async";
    const SYNC_SEGMENT: &'static str = "sync";
    const MEMORY_SEGMENT: &'static str = "memory";
    const ALLOCATOR_SEGMENT: &'static str = "allocator";
    const COMPILER_SEGMENT: &'static str = "compiler";
    const IO_SEGMENT: &'static str = "io";
    const MUX_SEGMENT: &'static str = "mux";
    const URING_SEGMENT: &'static str = "uring";
    const URING_CONSTANTS_SEGMENT: &'static str = "uring_constants";

    pub(super) fn from_import(names: &[String], sub_path: &[String]) -> Option<Self> {
        if sub_path.is_empty() && names.iter().any(|name| name == Self::COMPILER_SEGMENT) {
            return Some(Self::CompilerFacade);
        }

        Self::from_sub_path(sub_path)
    }

    pub(super) fn from_sub_path(sub_path: &[String]) -> Option<Self> {
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::COMPILER_SEGMENT)
        {
            return Some(Self::CompilerFacade);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ACTOR_SEGMENT)
        {
            return Some(Self::ActorFramework);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ASYNC_SEGMENT)
        {
            return Some(Self::AsyncRuntime);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::SYNC_SEGMENT)
        {
            return Some(Self::SyncRuntime);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::MEMORY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ALLOCATOR_SEGMENT)
        {
            return Some(Self::AllocatorFramework);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::IO_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::MUX_SEGMENT)
            && sub_path.get(2).is_some_and(|segment| {
                segment == Self::URING_SEGMENT || segment == Self::URING_CONSTANTS_SEGMENT
            })
        {
            return Some(Self::IoUringRuntime);
        }
        None
    }

    pub(super) fn gate_message(self) -> &'static str {
        match self {
            Self::ActorFramework => {
                "std actor framework modules are gated until mailbox, scheduling, supervisor, and allocator semantics are implemented"
            }
            Self::AllocatorFramework => {
                "std allocator modules are gated until allocator ownership and effect semantics are implemented"
            }
            Self::AsyncRuntime => {
                "std async runtime modules are gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::CompilerFacade => {
                "std compiler facade is gated until raw intrinsic ownership, allocation, and host-effect boundaries are implemented"
            }
            Self::SyncRuntime => {
                "std sync runtime modules are gated until channel, mailbox, and blocking semantics are implemented"
            }
            Self::IoUringRuntime => {
                "std io_uring modules are gated until host-effect, allocator, and async runtime semantics are implemented"
            }
        }
    }
}
