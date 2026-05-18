use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CIntrinsic {
    AddOverflow,
    Alignof,
    AtomicAdd,
    AtomicCas,
    AtomicLoad,
    AtomicStore,
    AtomicSub,
    AtomicXchg,
    Bswap16,
    Bswap32,
    Bswap64,
    CallExternal,
    Ctpop,
    Ctlz,
    Cttz,
    Debugtrap,
    Discriminant,
    Dlerror,
    Fence,
    Gep,
    GepStruct,
    GetPayload,
    GetSymbol,
    InlineC,
    IntToPtr,
    IsNull,
    LibcRead,
    LibcWrite,
    Load,
    LoadLibrary,
    Memcmp,
    Memcpy,
    Memmove,
    Memset,
    MulOverflow,
    NullPtr,
    Nullptr,
    Panic,
    PtrToInt,
    RawAllocate,
    RawDeallocate,
    RawPtrCast,
    RawPtrOffset,
    RawReallocate,
    SetDiscriminant,
    SetPayload,
    Sizeof,
    SitofpI64F64,
    StaticStringPtr,
    Store,
    Strlen,
    SubOverflow,
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,
    Trap,
    TruncF32I32,
    TruncF64I64,
    UitofpU64F64,
    UnloadLibrary,
    Unreachable,
}

impl CIntrinsic {
    pub(super) const ADD_OVERFLOW: &'static str = "add_overflow";
    pub(super) const ALIGNOF: &'static str = "alignof";
    pub(super) const ATOMIC_ADD: &'static str = "atomic_add";
    pub(super) const ATOMIC_CAS: &'static str = "atomic_cas";
    pub(super) const ATOMIC_LOAD: &'static str = "atomic_load";
    pub(super) const ATOMIC_STORE: &'static str = "atomic_store";
    pub(super) const ATOMIC_SUB: &'static str = "atomic_sub";
    pub(super) const ATOMIC_XCHG: &'static str = "atomic_xchg";
    pub(super) const BSWAP16: &'static str = "bswap16";
    pub(super) const BSWAP32: &'static str = "bswap32";
    pub(super) const BSWAP64: &'static str = "bswap64";
    pub(super) const CALL_EXTERNAL: &'static str = "call_external";
    pub(super) const CTPOP: &'static str = "ctpop";
    pub(super) const CTLZ: &'static str = "ctlz";
    pub(super) const CTTZ: &'static str = "cttz";
    pub(super) const DEBUGTRAP: &'static str = "debugtrap";
    pub(super) const DISCRIMINANT: &'static str = "discriminant";
    pub(super) const DLERROR: &'static str = "dlerror";
    pub(super) const FENCE: &'static str = "fence";
    pub(super) const GEP: &'static str = "gep";
    pub(super) const GEP_STRUCT: &'static str = "gep_struct";
    pub(super) const GET_PAYLOAD: &'static str = "get_payload";
    pub(super) const GET_SYMBOL: &'static str = "get_symbol";
    pub(super) const INLINE_C: &'static str = "inline_c";
    pub(super) const INT_TO_PTR: &'static str = "int_to_ptr";
    pub(super) const IS_NULL: &'static str = "is_null";
    pub(super) const LIBC_READ: &'static str = "libc_read";
    pub(super) const LIBC_WRITE: &'static str = "libc_write";
    pub(super) const LOAD: &'static str = "load";
    pub(super) const LOAD_LIBRARY: &'static str = "load_library";
    pub(super) const MEMCMP: &'static str = "memcmp";
    pub(super) const MEMCPY: &'static str = "memcpy";
    pub(super) const MEMMOVE: &'static str = "memmove";
    pub(super) const MEMSET: &'static str = "memset";
    pub(super) const MUL_OVERFLOW: &'static str = "mul_overflow";
    pub(super) const NULL_PTR: &'static str = "null_ptr";
    pub(super) const NULLPTR: &'static str = "nullptr";
    pub(super) const PANIC: &'static str = "panic";
    pub(super) const PTR_TO_INT: &'static str = "ptr_to_int";
    pub(super) const RAW_ALLOCATE: &'static str = "raw_allocate";
    pub(super) const RAW_DEALLOCATE: &'static str = "raw_deallocate";
    pub(super) const RAW_PTR_CAST: &'static str = "raw_ptr_cast";
    pub(super) const RAW_PTR_OFFSET: &'static str = "raw_ptr_offset";
    pub(super) const RAW_REALLOCATE: &'static str = "raw_reallocate";
    pub(super) const SET_DISCRIMINANT: &'static str = "set_discriminant";
    pub(super) const SET_PAYLOAD: &'static str = "set_payload";
    pub(super) const SIZEOF: &'static str = "sizeof";
    pub(super) const SITOFP_I64_F64: &'static str = "sitofp_i64_f64";
    pub(super) const STATIC_STRING_PTR: &'static str = "static_string_ptr";
    pub(super) const STORE: &'static str = "store";
    pub(super) const STRLEN: &'static str = "strlen";
    pub(super) const SUB_OVERFLOW: &'static str = "sub_overflow";
    pub(super) const SYSCALL0: &'static str = "syscall0";
    pub(super) const SYSCALL1: &'static str = "syscall1";
    pub(super) const SYSCALL2: &'static str = "syscall2";
    pub(super) const SYSCALL3: &'static str = "syscall3";
    pub(super) const SYSCALL4: &'static str = "syscall4";
    pub(super) const SYSCALL5: &'static str = "syscall5";
    pub(super) const SYSCALL6: &'static str = "syscall6";
    pub(super) const TRAP: &'static str = "trap";
    pub(super) const TRUNC_F32_I32: &'static str = "trunc_f32_i32";
    pub(super) const TRUNC_F64_I64: &'static str = "trunc_f64_i64";
    pub(super) const UITOFP_U64_F64: &'static str = "uitofp_u64_f64";
    pub(super) const UNLOAD_LIBRARY: &'static str = "unload_library";
    pub(super) const UNREACHABLE: &'static str = "unreachable";

    pub(super) const ALL: &[CIntrinsic] = &[
        Self::AddOverflow,
        Self::Alignof,
        Self::AtomicAdd,
        Self::AtomicCas,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSub,
        Self::AtomicXchg,
        Self::Bswap16,
        Self::Bswap32,
        Self::Bswap64,
        Self::CallExternal,
        Self::Ctpop,
        Self::Ctlz,
        Self::Cttz,
        Self::Debugtrap,
        Self::Discriminant,
        Self::Dlerror,
        Self::Fence,
        Self::Gep,
        Self::GepStruct,
        Self::GetPayload,
        Self::GetSymbol,
        Self::InlineC,
        Self::IntToPtr,
        Self::IsNull,
        Self::LibcRead,
        Self::LibcWrite,
        Self::Load,
        Self::LoadLibrary,
        Self::Memcmp,
        Self::Memcpy,
        Self::Memmove,
        Self::Memset,
        Self::MulOverflow,
        Self::NullPtr,
        Self::Nullptr,
        Self::Panic,
        Self::PtrToInt,
        Self::RawAllocate,
        Self::RawDeallocate,
        Self::RawPtrCast,
        Self::RawPtrOffset,
        Self::RawReallocate,
        Self::SetDiscriminant,
        Self::SetPayload,
        Self::Sizeof,
        Self::SitofpI64F64,
        Self::StaticStringPtr,
        Self::Store,
        Self::Strlen,
        Self::SubOverflow,
        Self::Syscall0,
        Self::Syscall1,
        Self::Syscall2,
        Self::Syscall3,
        Self::Syscall4,
        Self::Syscall5,
        Self::Syscall6,
        Self::Trap,
        Self::TruncF32I32,
        Self::TruncF64I64,
        Self::UitofpU64F64,
        Self::UnloadLibrary,
        Self::Unreachable,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::AddOverflow => Self::ADD_OVERFLOW,
            Self::Alignof => Self::ALIGNOF,
            Self::AtomicAdd => Self::ATOMIC_ADD,
            Self::AtomicCas => Self::ATOMIC_CAS,
            Self::AtomicLoad => Self::ATOMIC_LOAD,
            Self::AtomicStore => Self::ATOMIC_STORE,
            Self::AtomicSub => Self::ATOMIC_SUB,
            Self::AtomicXchg => Self::ATOMIC_XCHG,
            Self::Bswap16 => Self::BSWAP16,
            Self::Bswap32 => Self::BSWAP32,
            Self::Bswap64 => Self::BSWAP64,
            Self::CallExternal => Self::CALL_EXTERNAL,
            Self::Ctpop => Self::CTPOP,
            Self::Ctlz => Self::CTLZ,
            Self::Cttz => Self::CTTZ,
            Self::Debugtrap => Self::DEBUGTRAP,
            Self::Discriminant => Self::DISCRIMINANT,
            Self::Dlerror => Self::DLERROR,
            Self::Fence => Self::FENCE,
            Self::Gep => Self::GEP,
            Self::GepStruct => Self::GEP_STRUCT,
            Self::GetPayload => Self::GET_PAYLOAD,
            Self::GetSymbol => Self::GET_SYMBOL,
            Self::InlineC => Self::INLINE_C,
            Self::IntToPtr => Self::INT_TO_PTR,
            Self::IsNull => Self::IS_NULL,
            Self::LibcRead => Self::LIBC_READ,
            Self::LibcWrite => Self::LIBC_WRITE,
            Self::Load => Self::LOAD,
            Self::LoadLibrary => Self::LOAD_LIBRARY,
            Self::Memcmp => Self::MEMCMP,
            Self::Memcpy => Self::MEMCPY,
            Self::Memmove => Self::MEMMOVE,
            Self::Memset => Self::MEMSET,
            Self::MulOverflow => Self::MUL_OVERFLOW,
            Self::NullPtr => Self::NULL_PTR,
            Self::Nullptr => Self::NULLPTR,
            Self::Panic => Self::PANIC,
            Self::PtrToInt => Self::PTR_TO_INT,
            Self::RawAllocate => Self::RAW_ALLOCATE,
            Self::RawDeallocate => Self::RAW_DEALLOCATE,
            Self::RawPtrCast => Self::RAW_PTR_CAST,
            Self::RawPtrOffset => Self::RAW_PTR_OFFSET,
            Self::RawReallocate => Self::RAW_REALLOCATE,
            Self::SetDiscriminant => Self::SET_DISCRIMINANT,
            Self::SetPayload => Self::SET_PAYLOAD,
            Self::Sizeof => Self::SIZEOF,
            Self::SitofpI64F64 => Self::SITOFP_I64_F64,
            Self::StaticStringPtr => Self::STATIC_STRING_PTR,
            Self::Store => Self::STORE,
            Self::Strlen => Self::STRLEN,
            Self::SubOverflow => Self::SUB_OVERFLOW,
            Self::Syscall0 => Self::SYSCALL0,
            Self::Syscall1 => Self::SYSCALL1,
            Self::Syscall2 => Self::SYSCALL2,
            Self::Syscall3 => Self::SYSCALL3,
            Self::Syscall4 => Self::SYSCALL4,
            Self::Syscall5 => Self::SYSCALL5,
            Self::Syscall6 => Self::SYSCALL6,
            Self::Trap => Self::TRAP,
            Self::TruncF32I32 => Self::TRUNC_F32_I32,
            Self::TruncF64I64 => Self::TRUNC_F64_I64,
            Self::UitofpU64F64 => Self::UITOFP_U64_F64,
            Self::UnloadLibrary => Self::UNLOAD_LIBRARY,
            Self::Unreachable => Self::UNREACHABLE,
        }
    }
}

impl fmt::Display for CIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CIntrinsic {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|intrinsic| intrinsic.as_str() == name)
            .ok_or(())
    }
}
