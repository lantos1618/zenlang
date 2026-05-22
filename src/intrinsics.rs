//! Compiler Intrinsics
//!
//! This is the SINGLE SOURCE OF TRUTH for all compiler intrinsic type information.
//! These are low-level primitives that map directly to backend code or host calls.
//!
//! Everything else (io, math, collections, etc.) should be written in Zen
//! using these intrinsics.

use crate::ast::AstType;
use crate::error::CompileError;
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// Intrinsic Module Recognition
// ============================================================================

/// The prefix used for raw intrinsic calls (e.g., `@builtin.raw_allocate`).
pub const INTRINSIC_PREFIX: &str = "@builtin";

/// The stdlib module that wraps raw intrinsics (stdlib/compiler.zen).
pub const COMPILER_MODULE: &str = "compiler";

/// Check if a module name routes directly to raw compiler intrinsics.
/// Only `@builtin` — the raw intrinsic prefix used in stdlib/compiler.zen.
pub fn is_intrinsic_module(name: &str) -> bool {
    name == INTRINSIC_PREFIX
}

/// Check if a module dispatches to compiler intrinsics (either raw `@builtin`
/// or the stdlib `compiler` bridge that wraps them).
pub fn is_compiler_intrinsic_module(name: &str) -> bool {
    name == INTRINSIC_PREFIX || name == COMPILER_MODULE
}

// ============================================================================
// Intrinsic Function Registry
// ============================================================================

/// Intrinsic function signature
#[derive(Clone)]
pub struct Intrinsic {
    pub params: Vec<(&'static str, AstType)>,
    pub return_type: AstType,
    pub doc: &'static str,
    pub category: &'static str,
}

/// Global singleton for compiler intrinsics
static INTRINSICS: OnceLock<HashMap<&'static str, Intrinsic>> = OnceLock::new();

/// Get the global intrinsics registry
fn get_intrinsics() -> &'static HashMap<&'static str, Intrinsic> {
    INTRINSICS.get_or_init(build_intrinsics)
}

/// Get all intrinsics (public API for LSP and other consumers)
pub fn get_all_intrinsics() -> &'static HashMap<&'static str, Intrinsic> {
    get_intrinsics()
}

/// Quick lookup for intrinsic return type
pub fn get_intrinsic_return_type(func_name: &str) -> Option<AstType> {
    get_intrinsics()
        .get(func_name)
        .map(|f| f.return_type.clone())
}

/// Get full intrinsic definition (params and return type)
#[allow(dead_code)] // Used by LSP
pub fn get_intrinsic(func_name: &str) -> Option<&'static Intrinsic> {
    get_intrinsics().get(func_name)
}

/// Validate intrinsic call and return its type
pub fn check_intrinsic_call(
    func_name: &str,
    args_len: usize,
) -> Option<Result<AstType, CompileError>> {
    let func = get_intrinsics().get(func_name)?;
    let expected = func.params.len();

    if args_len != expected {
        Some(Err(CompileError::TypeError(
            format!(
                "compiler.{}() expects {} argument(s), got {}",
                func_name, expected, args_len
            ),
            None,
        )))
    } else {
        Some(Ok(func.return_type.clone()))
    }
}

/// Check if a function name is a compiler intrinsic
#[allow(dead_code)] // Public API
pub fn is_intrinsic(name: &str) -> bool {
    get_intrinsics().contains_key(name)
}

// ============================================================================
// Intrinsic Definitions
// ============================================================================

/// Single variadic macro handles all arities (0..N params). No more copy-pasted arms.
macro_rules! intrinsic {
    ($map:expr, $name:expr, ($($pname:expr => $ptype:expr),* $(,)?) -> $ret:expr, $doc:expr, $category:expr) => {
        $map.insert($name, Intrinsic {
            params: vec![$(($pname, $ptype)),*],
            return_type: $ret,
            doc: $doc,
            category: $category,
        });
    };
}

fn build_intrinsics() -> HashMap<&'static str, Intrinsic> {
    let mut m = HashMap::new();

    let ptr = AstType::raw_ptr(AstType::U8);
    let ptr64 = AstType::raw_ptr(AstType::U64);
    let overflow_result = AstType::Struct {
        name: "OverflowResult".to_string(),
        fields: vec![
            ("result".to_string(), AstType::I64),
            ("overflow".to_string(), AstType::Bool),
        ],
    };

    // -- Memory allocation ------------------------------------------------
    intrinsic!(m, "raw_allocate",   ("size" => AstType::Usize) -> ptr.clone(), "Allocates raw memory using malloc", "Memory");
    intrinsic!(m, "raw_deallocate", ("ptr" => ptr.clone(), "size" => AstType::Usize) -> AstType::Void, "Deallocates memory", "Memory");
    intrinsic!(m, "raw_reallocate", ("ptr" => ptr.clone(), "old_size" => AstType::Usize, "new_size" => AstType::Usize) -> ptr.clone(), "Reallocates memory to a new size", "Memory");

    // -- Pointer operations -----------------------------------------------
    intrinsic!(m, "raw_ptr_offset", ("ptr" => ptr.clone(), "offset" => AstType::I64) -> ptr.clone(), "Offset a pointer by byte count", "Pointer");
    intrinsic!(m, "raw_ptr_cast",   ("ptr" => ptr.clone()) -> ptr.clone(), "Reinterprets a pointer type", "Pointer");
    intrinsic!(m, "ptr_to_int",     ("ptr" => ptr.clone()) -> AstType::I64, "Convert pointer to integer", "Convert");
    intrinsic!(m, "int_to_ptr",     ("addr" => AstType::I64) -> ptr.clone(), "Convert integer to pointer", "Convert");
    intrinsic!(m, "null_ptr",       () -> ptr.clone(), "Returns a null pointer", "Pointer");
    intrinsic!(m, "nullptr",        () -> ptr.clone(), "Alias for null_ptr", "Pointer");
    intrinsic!(m, "gep",            ("base_ptr" => ptr.clone(), "offset" => AstType::I64) -> ptr.clone(), "GetElementPointer - byte-level pointer arithmetic", "Pointer");
    intrinsic!(m, "gep_struct",     ("struct_ptr" => ptr.clone(), "field_index" => AstType::I32) -> ptr.clone(), "Struct field access using GEP", "Pointer");
    intrinsic!(m, "is_null",        ("ptr" => ptr.clone()) -> AstType::Bool, "Check if pointer is null", "Pointer");

    // -- Memory operations ------------------------------------------------
    intrinsic!(m, "memcpy",  ("dest" => ptr.clone(), "src" => ptr.clone(), "size" => AstType::Usize) -> AstType::Void, "Copy bytes (non-overlapping)", "Memory");
    intrinsic!(m, "memmove", ("dest" => ptr.clone(), "src" => ptr.clone(), "size" => AstType::Usize) -> AstType::Void, "Copy bytes (overlapping safe)", "Memory");
    intrinsic!(m, "memset",  ("dest" => ptr.clone(), "value" => AstType::U8, "size" => AstType::Usize) -> AstType::Void, "Set all bytes to a value", "Memory");
    intrinsic!(m, "memcmp",  ("ptr1" => ptr.clone(), "ptr2" => ptr.clone(), "size" => AstType::Usize) -> AstType::I32, "Compare bytes in memory", "Memory");

    // -- Type introspection -----------------------------------------------
    intrinsic!(m, "sizeof",  () -> AstType::Usize, "Returns the size of a type in bytes", "Type");
    intrinsic!(m, "alignof", () -> AstType::Usize, "Returns the alignment of a type", "Type");

    // -- Inline C ---------------------------------------------------------
    intrinsic!(m, "inline_c", ("code" => AstType::StaticString) -> AstType::Void, "Inline C code compilation", "FFI");

    // -- Atomic operations ------------------------------------------------
    intrinsic!(m, "atomic_load",  ("ptr" => ptr64.clone()) -> AstType::U64, "Atomically load a value", "Atomic");
    intrinsic!(m, "atomic_store", ("ptr" => ptr64.clone(), "value" => AstType::U64) -> AstType::Void, "Atomically store a value", "Atomic");
    intrinsic!(m, "atomic_add",   ("ptr" => ptr64.clone(), "value" => AstType::U64) -> AstType::U64, "Atomic add", "Atomic");
    intrinsic!(m, "atomic_sub",   ("ptr" => ptr64.clone(), "value" => AstType::U64) -> AstType::U64, "Atomic subtract", "Atomic");
    intrinsic!(m, "atomic_cas",   ("ptr" => ptr64.clone(), "expected" => AstType::U64, "new_value" => AstType::U64) -> AstType::Bool, "Compare-and-swap", "Atomic");
    intrinsic!(m, "atomic_xchg",  ("ptr" => ptr64.clone(), "value" => AstType::U64) -> AstType::U64, "Atomic exchange", "Atomic");
    intrinsic!(m, "fence",        () -> AstType::Void, "Memory fence", "Atomic");

    // -- Bitwise operations -----------------------------------------------
    intrinsic!(m, "bswap16", ("value" => AstType::U16) -> AstType::U16, "Byte-swap 16-bit value", "Bitwise");
    intrinsic!(m, "bswap32", ("value" => AstType::U32) -> AstType::U32, "Byte-swap 32-bit value", "Bitwise");
    intrinsic!(m, "bswap64", ("value" => AstType::U64) -> AstType::U64, "Byte-swap 64-bit value", "Bitwise");
    intrinsic!(m, "ctlz",   ("value" => AstType::U64) -> AstType::U64, "Count leading zeros", "Bitwise");
    intrinsic!(m, "cttz",   ("value" => AstType::U64) -> AstType::U64, "Count trailing zeros", "Bitwise");
    intrinsic!(m, "ctpop",  ("value" => AstType::U64) -> AstType::U64, "Population count", "Bitwise");

    // -- Overflow-checked arithmetic --------------------------------------
    intrinsic!(m, "add_overflow", ("a" => AstType::I64, "b" => AstType::I64) -> overflow_result.clone(), "Add with overflow detection", "Overflow");
    intrinsic!(m, "sub_overflow", ("a" => AstType::I64, "b" => AstType::I64) -> overflow_result.clone(), "Subtract with overflow detection", "Overflow");
    intrinsic!(m, "mul_overflow", ("a" => AstType::I64, "b" => AstType::I64) -> overflow_result, "Multiply with overflow detection", "Overflow");

    // -- Type conversions -------------------------------------------------
    intrinsic!(m, "cast",            ("value" => AstType::I64, "target_type" => AstType::I64) -> AstType::I64, "Cast a value to a numeric type: cast(value, i64)", "Convert");
    intrinsic!(m, "trunc_f64_i64",  ("value" => AstType::F64) -> AstType::I64, "Truncate f64 to i64", "Convert");
    intrinsic!(m, "trunc_f32_i32",  ("value" => AstType::F32) -> AstType::I32, "Truncate f32 to i32", "Convert");
    intrinsic!(m, "sitofp_i64_f64", ("value" => AstType::I64) -> AstType::F64, "Convert signed i64 to f64", "Convert");
    intrinsic!(m, "uitofp_u64_f64", ("value" => AstType::U64) -> AstType::F64, "Convert unsigned u64 to f64", "Convert");

    // -- Debug/trap/panic -------------------------------------------------
    intrinsic!(m, "unreachable", () -> AstType::Void, "Mark code as unreachable", "Debug");
    intrinsic!(m, "trap",        () -> AstType::Void, "Trigger a trap/abort", "Debug");
    intrinsic!(m, "debugtrap",   () -> AstType::Void, "Trigger a debug trap", "Debug");
    intrinsic!(m, "panic",       ("message" => AstType::StaticString) -> AstType::Void, "Trigger a panic with message", "Debug");

    // -- Syscalls (Linux x86-64) ------------------------------------------
    intrinsic!(m, "syscall0", ("number" => AstType::I64) -> AstType::I64, "System call with 0 arguments", "Syscall");
    intrinsic!(m, "syscall1", ("number" => AstType::I64, "arg0" => AstType::I64) -> AstType::I64, "System call with 1 argument", "Syscall");
    intrinsic!(m, "syscall2", ("number" => AstType::I64, "arg0" => AstType::I64, "arg1" => AstType::I64) -> AstType::I64, "System call with 2 arguments", "Syscall");
    intrinsic!(m, "syscall3", ("number" => AstType::I64, "arg0" => AstType::I64, "arg1" => AstType::I64, "arg2" => AstType::I64) -> AstType::I64, "System call with 3 arguments", "Syscall");
    intrinsic!(m, "syscall4", ("number" => AstType::I64, "arg0" => AstType::I64, "arg1" => AstType::I64, "arg2" => AstType::I64, "arg3" => AstType::I64) -> AstType::I64, "System call with 4 arguments", "Syscall");
    intrinsic!(m, "syscall5", ("number" => AstType::I64, "arg0" => AstType::I64, "arg1" => AstType::I64, "arg2" => AstType::I64, "arg3" => AstType::I64, "arg4" => AstType::I64) -> AstType::I64, "System call with 5 arguments", "Syscall");
    intrinsic!(m, "syscall6", ("number" => AstType::I64, "arg0" => AstType::I64, "arg1" => AstType::I64, "arg2" => AstType::I64, "arg3" => AstType::I64, "arg4" => AstType::I64, "arg5" => AstType::I64) -> AstType::I64, "System call with 6 arguments", "Syscall");

    // -- FFI/dynamic loading ----------------------------------------------
    intrinsic!(m, "load_library",   ("path" => AstType::StaticString) -> ptr.clone(), "Load a dynamic library", "FFI");
    intrinsic!(m, "get_symbol",     ("lib_handle" => ptr.clone(), "symbol_name" => AstType::StaticString) -> ptr.clone(), "Get symbol from library", "FFI");
    intrinsic!(m, "unload_library", ("lib_handle" => ptr.clone()) -> AstType::Void, "Unload a dynamic library", "FFI");
    intrinsic!(m, "dlerror",        () -> ptr.clone(), "Get dynamic linker error", "FFI");
    intrinsic!(m, "call_external",  ("func_ptr" => ptr.clone()) -> AstType::I64, "Call external function pointer", "FFI");

    // -- String operations ------------------------------------------------
    intrinsic!(m, "strlen", ("value" => AstType::StaticString) -> AstType::Usize, "Get length of a static string", "String");
    intrinsic!(m, "static_string_ptr", ("value" => AstType::StaticString) -> ptr.clone(), "Get pointer to static string data", "String");

    // -- IO operations (libc wrappers) ------------------------------------
    intrinsic!(m, "libc_write", ("fd" => AstType::I32, "buf" => ptr.clone(), "len" => AstType::Usize) -> AstType::I64, "Write via libc", "IO");
    intrinsic!(m, "libc_read",  ("fd" => AstType::I32, "buf" => ptr.clone(), "len" => AstType::Usize) -> AstType::I64, "Read via libc", "IO");

    // -- Generic load/store (type determined by context) -------------------
    let generic_t = AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    };
    intrinsic!(m, "load",  ("ptr" => ptr.clone()) -> generic_t.clone(), "Load a value from a pointer", "Memory");
    intrinsic!(m, "store", ("ptr" => ptr.clone(), "value" => generic_t) -> AstType::Void, "Store a value to a pointer", "Memory");

    // -- Enum intrinsics --------------------------------------------------
    intrinsic!(m, "discriminant",     ("enum_value" => ptr.clone()) -> AstType::I32, "Reads the discriminant from an enum", "Enum");
    intrinsic!(m, "set_discriminant", ("enum_ptr" => ptr.clone(), "discriminant" => AstType::I32) -> AstType::Void, "Sets the discriminant of an enum", "Enum");
    intrinsic!(m, "get_payload",      ("enum_value" => ptr.clone()) -> ptr.clone(), "Returns pointer to enum payload", "Enum");
    intrinsic!(m, "set_payload",      ("enum_ptr" => ptr.clone(), "payload" => ptr) -> AstType::Void, "Copies payload into enum", "Enum");

    m
}

// ============================================================================
// Well-Known Types (types with special compiler semantics)
// ============================================================================
