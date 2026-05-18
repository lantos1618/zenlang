#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedIntrinsic {
    AsyncEnqueue,
    AsyncYield,
    RawAllocate,
    RawDeallocate,
    RawReallocate,
    TypeMatch,
}

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";
    pub(super) const ASYNC_ENQUEUE: &'static str = "async_enqueue";
    pub(super) const ASYNC_YIELD: &'static str = "async_yield";
    pub(super) const RAW_ALLOCATE: &'static str = "raw_allocate";
    pub(super) const RAW_DEALLOCATE: &'static str = "raw_deallocate";
    pub(super) const RAW_REALLOCATE: &'static str = "raw_reallocate";
    pub(super) const TYPE_MATCH: &'static str = "type_match";
    const ALL: &[GatedIntrinsic] = &[
        Self::AsyncEnqueue,
        Self::AsyncYield,
        Self::RawAllocate,
        Self::RawDeallocate,
        Self::RawReallocate,
        Self::TypeMatch,
    ];

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|intrinsic| intrinsic.as_str() == name)
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::AsyncEnqueue => Self::ASYNC_ENQUEUE,
            Self::AsyncYield => Self::ASYNC_YIELD,
            Self::RawAllocate => Self::RAW_ALLOCATE,
            Self::RawDeallocate => Self::RAW_DEALLOCATE,
            Self::RawReallocate => Self::RAW_REALLOCATE,
            Self::TypeMatch => Self::TYPE_MATCH,
        }
    }

    pub(super) const fn gate_message(self) -> &'static str {
        match self {
            Self::AsyncEnqueue => {
                "async task enqueue is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::AsyncYield => {
                "async yield is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::RawAllocate => {
                "raw allocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::RawDeallocate => {
                "raw deallocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::RawReallocate => {
                "raw reallocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::TypeMatch => {
                "comptime type matching is gated until typed metadata and derive lowering are implemented"
            }
        }
    }
}
