use super::GatedIntrinsic;

impl GatedIntrinsic {
    pub(in crate::typechecker) const fn gate_message(self) -> &'static str {
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
