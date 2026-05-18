use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedIntrinsic {
    AtomicAdd,
    AtomicCas,
    AtomicLoad,
    AtomicStore,
    AtomicSub,
    AtomicXchg,
    AsyncEnqueue,
    AsyncYield,
    Fence,
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
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,
    TypeMatch,
}

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";
    pub(super) const ATOMIC_ADD: &'static str = "atomic_add";
    pub(super) const ATOMIC_CAS: &'static str = "atomic_cas";
    pub(super) const ATOMIC_LOAD: &'static str = "atomic_load";
    pub(super) const ATOMIC_STORE: &'static str = "atomic_store";
    pub(super) const ATOMIC_SUB: &'static str = "atomic_sub";
    pub(super) const ATOMIC_XCHG: &'static str = "atomic_xchg";
    pub(super) const ASYNC_ENQUEUE: &'static str = "async_enqueue";
    pub(super) const ASYNC_YIELD: &'static str = "async_yield";
    pub(super) const FENCE: &'static str = "fence";
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
    pub(super) const SYSCALL0: &'static str = "syscall0";
    pub(super) const SYSCALL1: &'static str = "syscall1";
    pub(super) const SYSCALL2: &'static str = "syscall2";
    pub(super) const SYSCALL3: &'static str = "syscall3";
    pub(super) const SYSCALL4: &'static str = "syscall4";
    pub(super) const SYSCALL5: &'static str = "syscall5";
    pub(super) const SYSCALL6: &'static str = "syscall6";
    pub(super) const TYPE_MATCH: &'static str = "type_match";
    const ALL: &[GatedIntrinsic] = &[
        Self::AtomicAdd,
        Self::AtomicCas,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSub,
        Self::AtomicXchg,
        Self::AsyncEnqueue,
        Self::AsyncYield,
        Self::Fence,
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
        Self::Syscall0,
        Self::Syscall1,
        Self::Syscall2,
        Self::Syscall3,
        Self::Syscall4,
        Self::Syscall5,
        Self::Syscall6,
        Self::TypeMatch,
    ];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::AtomicAdd => Self::ATOMIC_ADD,
            Self::AtomicCas => Self::ATOMIC_CAS,
            Self::AtomicLoad => Self::ATOMIC_LOAD,
            Self::AtomicStore => Self::ATOMIC_STORE,
            Self::AtomicSub => Self::ATOMIC_SUB,
            Self::AtomicXchg => Self::ATOMIC_XCHG,
            Self::AsyncEnqueue => Self::ASYNC_ENQUEUE,
            Self::AsyncYield => Self::ASYNC_YIELD,
            Self::Fence => Self::FENCE,
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
            Self::Syscall0 => Self::SYSCALL0,
            Self::Syscall1 => Self::SYSCALL1,
            Self::Syscall2 => Self::SYSCALL2,
            Self::Syscall3 => Self::SYSCALL3,
            Self::Syscall4 => Self::SYSCALL4,
            Self::Syscall5 => Self::SYSCALL5,
            Self::Syscall6 => Self::SYSCALL6,
            Self::TypeMatch => Self::TYPE_MATCH,
        }
    }

    pub(super) const fn gate_message(self) -> &'static str {
        match self {
            Self::AtomicAdd => {
                "atomic add is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AtomicCas => {
                "atomic compare-and-swap is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AtomicLoad => {
                "atomic load is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AtomicStore => {
                "atomic store is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AtomicSub => {
                "atomic subtract is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AtomicXchg => {
                "atomic exchange is gated until memory-order and Sync/Async effect semantics are implemented"
            }
            Self::AsyncEnqueue => {
                "async task enqueue is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::AsyncYield => {
                "async yield is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::Fence => {
                "atomic fence is gated until memory-order and Sync/Async effect semantics are implemented"
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
            Self::Syscall0 => {
                "syscall0 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall1 => {
                "syscall1 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall2 => {
                "syscall2 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall3 => {
                "syscall3 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall4 => {
                "syscall4 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall5 => {
                "syscall5 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::Syscall6 => {
                "syscall6 is gated until host effect declarations and syscall ABI semantics are implemented"
            }
            Self::TypeMatch => {
                "comptime type matching is gated until typed metadata and derive lowering are implemented"
            }
        }
    }
}

impl fmt::Display for GatedIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for GatedIntrinsic {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        GatedIntrinsic::ALL
            .iter()
            .copied()
            .find(|intrinsic| intrinsic.as_str() == name)
            .ok_or(())
    }
}
