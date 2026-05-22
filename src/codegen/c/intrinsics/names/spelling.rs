macro_rules! c_intrinsic_spellings {
    ($define:ident) => {
        $define! {
            AddOverflow => "add_overflow",
            Alignof => "alignof",
            AtomicAdd => "atomic_add",
            AtomicCas => "atomic_cas",
            AtomicLoad => "atomic_load",
            AtomicStore => "atomic_store",
            AtomicSub => "atomic_sub",
            AtomicXchg => "atomic_xchg",
            Bswap16 => "bswap16",
            Bswap32 => "bswap32",
            Bswap64 => "bswap64",
            CallExternal => "call_external",
            Ctpop => "ctpop",
            Ctlz => "ctlz",
            Cttz => "cttz",
            Debugtrap => "debugtrap",
            Discriminant => "discriminant",
            Dlerror => "dlerror",
            Fence => "fence",
            Gep => "gep",
            GepStruct => "gep_struct",
            GetPayload => "get_payload",
            GetSymbol => "get_symbol",
            InlineC => "inline_c",
            IntToPtr => "int_to_ptr",
            IsNull => "is_null",
            LibcRead => "libc_read",
            LibcWrite => "libc_write",
            Load => "load",
            LoadLibrary => "load_library",
            Memcmp => "memcmp",
            Memcpy => "memcpy",
            Memmove => "memmove",
            Memset => "memset",
            MulOverflow => "mul_overflow",
            NullPtr => "null_ptr",
            Nullptr => "nullptr",
            Panic => "panic",
            PtrToInt => "ptr_to_int",
            RawAllocate => "raw_allocate",
            RawDeallocate => "raw_deallocate",
            RawPtrCast => "raw_ptr_cast",
            RawPtrOffset => "raw_ptr_offset",
            RawReallocate => "raw_reallocate",
            SetDiscriminant => "set_discriminant",
            SetPayload => "set_payload",
            Sizeof => "sizeof",
            SitofpI64F64 => "sitofp_i64_f64",
            StaticStringPtr => "static_string_ptr",
            Store => "store",
            Strlen => "strlen",
            SubOverflow => "sub_overflow",
            Syscall0 => "syscall0",
            Syscall1 => "syscall1",
            Syscall2 => "syscall2",
            Syscall3 => "syscall3",
            Syscall4 => "syscall4",
            Syscall5 => "syscall5",
            Syscall6 => "syscall6",
            Trap => "trap",
            TruncF32I32 => "trunc_f32_i32",
            TruncF64I64 => "trunc_f64_i64",
            UitofpU64F64 => "uitofp_u64_f64",
            UnloadLibrary => "unload_library",
            Unreachable => "unreachable",
        }
    };
}

macro_rules! define_c_intrinsics {
    ($($variant:ident => $spelling:literal,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(in crate::codegen::c::intrinsics) enum CIntrinsic {
            $($variant,)*
        }

        pub(super) const SPELLINGS: &[(CIntrinsic, &str)] = &[
            $((CIntrinsic::$variant, $spelling),)*
        ];

        impl CIntrinsic {
            pub(super) const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $spelling,)*
                }
            }
        }
    };
}

c_intrinsic_spellings!(define_c_intrinsics);
