#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedIntrinsic {
    AsyncEnqueue,
    AsyncYield,
    Gep,
    GepStruct,
    IntToPtr,
    Load,
    Memcmp,
    Memcpy,
    Memmove,
    Memset,
    PtrToInt,
    RawAllocate,
    RawDeallocate,
    RawPtrCast,
    RawReallocate,
    Store,
    TypeMatch,
}

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";
    pub(super) const ASYNC_ENQUEUE: &'static str = "async_enqueue";
    pub(super) const ASYNC_YIELD: &'static str = "async_yield";
    pub(super) const GEP: &'static str = "gep";
    pub(super) const GEP_STRUCT: &'static str = "gep_struct";
    pub(super) const INT_TO_PTR: &'static str = "int_to_ptr";
    pub(super) const LOAD: &'static str = "load";
    pub(super) const MEMCMP: &'static str = "memcmp";
    pub(super) const MEMCPY: &'static str = "memcpy";
    pub(super) const MEMMOVE: &'static str = "memmove";
    pub(super) const MEMSET: &'static str = "memset";
    pub(super) const PTR_TO_INT: &'static str = "ptr_to_int";
    pub(super) const RAW_ALLOCATE: &'static str = "raw_allocate";
    pub(super) const RAW_DEALLOCATE: &'static str = "raw_deallocate";
    pub(super) const RAW_PTR_CAST: &'static str = "raw_ptr_cast";
    pub(super) const RAW_REALLOCATE: &'static str = "raw_reallocate";
    pub(super) const STORE: &'static str = "store";
    pub(super) const TYPE_MATCH: &'static str = "type_match";
    const ALL: &[GatedIntrinsic] = &[
        Self::AsyncEnqueue,
        Self::AsyncYield,
        Self::Gep,
        Self::GepStruct,
        Self::IntToPtr,
        Self::Load,
        Self::Memcmp,
        Self::Memcpy,
        Self::Memmove,
        Self::Memset,
        Self::PtrToInt,
        Self::RawAllocate,
        Self::RawDeallocate,
        Self::RawPtrCast,
        Self::RawReallocate,
        Self::Store,
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
            Self::Gep => Self::GEP,
            Self::GepStruct => Self::GEP_STRUCT,
            Self::IntToPtr => Self::INT_TO_PTR,
            Self::Load => Self::LOAD,
            Self::Memcmp => Self::MEMCMP,
            Self::Memcpy => Self::MEMCPY,
            Self::Memmove => Self::MEMMOVE,
            Self::Memset => Self::MEMSET,
            Self::PtrToInt => Self::PTR_TO_INT,
            Self::RawAllocate => Self::RAW_ALLOCATE,
            Self::RawDeallocate => Self::RAW_DEALLOCATE,
            Self::RawPtrCast => Self::RAW_PTR_CAST,
            Self::RawReallocate => Self::RAW_REALLOCATE,
            Self::Store => Self::STORE,
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
            Self::Gep => {
                "raw pointer offset is gated until ownership and layout semantics are implemented"
            }
            Self::GepStruct => {
                "raw struct pointer offset is gated until ownership and layout semantics are implemented"
            }
            Self::IntToPtr => {
                "integer to raw pointer conversion is gated until ownership and pointer provenance semantics are implemented"
            }
            Self::Load => {
                "raw pointer load is gated until ownership and memory access semantics are implemented"
            }
            Self::Memcmp => {
                "raw memory compare is gated until allocator ownership and effect semantics are implemented"
            }
            Self::Memcpy => {
                "raw memory copy is gated until allocator ownership and effect semantics are implemented"
            }
            Self::Memmove => {
                "raw memory move is gated until allocator ownership and effect semantics are implemented"
            }
            Self::Memset => {
                "raw memory set is gated until allocator ownership and effect semantics are implemented"
            }
            Self::PtrToInt => {
                "raw pointer to integer conversion is gated until ownership and pointer provenance semantics are implemented"
            }
            Self::RawAllocate => {
                "raw allocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::RawDeallocate => {
                "raw deallocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::RawPtrCast => {
                "raw pointer cast is gated until ownership and pointer provenance semantics are implemented"
            }
            Self::RawReallocate => {
                "raw reallocation is gated until allocator ownership and effect semantics are implemented"
            }
            Self::Store => {
                "raw pointer store is gated until ownership and memory access semantics are implemented"
            }
            Self::TypeMatch => {
                "comptime type matching is gated until typed metadata and derive lowering are implemented"
            }
        }
    }
}
