//! Standard library codegen - only compiler intrinsics
//! High-level stdlib (io, fs, core, math) should be implemented in Zen
//! Collections (HashMap, HashSet, Vec) are implemented in stdlib Zen using intrinsics

pub mod compiler;
pub mod helpers;

// Re-export compiler intrinsics
pub use compiler::{
    // Bitwise intrinsics
    compile_bswap16,
    compile_bswap32,
    compile_bswap64,
    // External calls
    compile_call_external,
    compile_ctlz,
    compile_ctpop,
    compile_cttz,
    // Enum intrinsics
    compile_discriminant,
    // Library loading
    compile_dlerror,
    // GEP intrinsics
    compile_gep,
    compile_gep_struct,
    compile_get_payload,
    compile_get_symbol,
    // Inline C
    compile_inline_c,
    // Pointer conversion
    compile_int_to_ptr,
    // Pointer utilities
    compile_is_null,
    compile_libc_read,
    // IO intrinsics (libc wrappers)
    compile_libc_write,
    // Load/store intrinsics
    compile_load,
    compile_load_library,
    // Memory operations
    compile_memcmp,
    compile_memcpy,
    compile_memmove,
    compile_memset,
    compile_null_ptr,
    // Panic
    compile_panic,
    compile_ptr_to_int,
    // Memory allocation
    compile_raw_allocate,
    compile_raw_deallocate,
    // Pointer operations
    compile_raw_ptr_cast,
    compile_raw_ptr_offset,
    compile_raw_reallocate,
    compile_set_discriminant,
    compile_set_payload,
    // Sizeof
    compile_sizeof,
    compile_store,
    // Syscall intrinsics
    compile_syscall0,
    compile_syscall1,
    compile_syscall2,
    compile_syscall3,
    compile_syscall4,
    compile_syscall5,
    compile_syscall6,
    compile_unload_library,
};
