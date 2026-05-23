use std::fmt;
use std::str::FromStr;

mod spelling;

pub(super) use spelling::GatedIntrinsic;

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";

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
            Self::RawPtrOffset => {
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
        spelling::SPELLINGS
            .iter()
            .find(|(_, spelling)| *spelling == name)
            .map(|(intrinsic, _)| *intrinsic)
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn gated_intrinsic_spellings_round_trip_through_single_table() {
        let mut seen = HashSet::new();

        for (intrinsic, spelling) in spelling::SPELLINGS {
            assert!(
                seen.insert(*spelling),
                "duplicate gated intrinsic spelling: {spelling}"
            );
            assert_eq!(intrinsic.as_str(), *spelling);
            assert_eq!(intrinsic.to_string(), *spelling);
            assert_eq!(spelling.parse::<GatedIntrinsic>(), Ok(*intrinsic));
        }
    }
}
