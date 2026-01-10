//! Standard library codegen - only compiler intrinsics
//! High-level stdlib (io, fs, core, math) should be implemented in Zen
//! Collections (HashMap, HashSet, Vec) are implemented in stdlib Zen using intrinsics

pub mod compiler;

// Re-export compiler intrinsics
pub use compiler::{
    // Inline C
    compile_inline_c,
    // Memory allocation
    compile_raw_allocate,
    compile_raw_deallocate,
    compile_raw_reallocate,
    // Pointer operations
    compile_raw_ptr_cast,
    compile_raw_ptr_offset,
    // External calls
    compile_call_external,
    // Library loading
    compile_dlerror,
    compile_get_symbol,
    compile_load_library,
    compile_unload_library,
    // Pointer utilities
    compile_is_null,
    compile_null_ptr,
    // Enum intrinsics
    compile_discriminant,
    compile_get_payload,
    compile_set_discriminant,
    compile_set_payload,
    // GEP intrinsics
    compile_gep,
    compile_gep_struct,
    // Load/store intrinsics
    compile_load,
    compile_store,
    // Pointer conversion
    compile_int_to_ptr,
    compile_ptr_to_int,
    // Sizeof
    compile_sizeof,
    // Memory operations
    compile_memcmp,
    compile_memcpy,
    compile_memmove,
    compile_memset,
    // Bitwise intrinsics
    compile_bswap16,
    compile_bswap32,
    compile_bswap64,
    compile_ctlz,
    compile_ctpop,
    compile_cttz,
    // Syscall intrinsics
    compile_syscall0,
    compile_syscall1,
    compile_syscall2,
    compile_syscall3,
    compile_syscall4,
    compile_syscall5,
    compile_syscall6,
};
