macro_rules! gated_intrinsic_spellings {
    ($define:ident) => {
        $define! {
            AtomicAdd => "atomic_add",
            AtomicCas => "atomic_cas",
            AtomicLoad => "atomic_load",
            AtomicStore => "atomic_store",
            AtomicSub => "atomic_sub",
            AtomicXchg => "atomic_xchg",
            AsyncEnqueue => "async_enqueue",
            AsyncYield => "async_yield",
            Fence => "fence",
            Gep => "gep",
            GepStruct => "gep_struct",
            IntToPtr => "int_to_ptr",
            Load => "load",
            Memcmp => "memcmp",
            Memcpy => "memcpy",
            Memmove => "memmove",
            Memset => "memset",
            PtrToInt => "ptr_to_int",
            RawAllocate => "raw_allocate",
            RawDeallocate => "raw_deallocate",
            RawPtrCast => "raw_ptr_cast",
            RawReallocate => "raw_reallocate",
            Store => "store",
            Syscall0 => "syscall0",
            Syscall1 => "syscall1",
            Syscall2 => "syscall2",
            Syscall3 => "syscall3",
            Syscall4 => "syscall4",
            Syscall5 => "syscall5",
            Syscall6 => "syscall6",
            TypeMatch => "type_match",
        }
    };
}

macro_rules! define_gated_intrinsics {
    ($($variant:ident => $spelling:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(in crate::typechecker) enum GatedIntrinsic {
            $($variant,)*
        }

        pub(super) const SPELLINGS: &[(GatedIntrinsic, &str)] = &[
            $((GatedIntrinsic::$variant, $spelling),)*
        ];

        impl GatedIntrinsic {
            pub(super) const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $spelling,)*
                }
            }
        }
    };
}

gated_intrinsic_spellings!(define_gated_intrinsics);
