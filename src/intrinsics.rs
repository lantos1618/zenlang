//! Compiler Intrinsics
//!
//! This is the SINGLE SOURCE OF TRUTH for all compiler intrinsic type information.
//! These are low-level primitives that map directly to LLVM IR or syscalls.
//!
//! Everything else (io, math, collections, etc.) should be written in Zen
//! using these intrinsics.

use crate::ast::AstType;
use crate::error::CompileError;
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// Built-in Modules (always available without explicit import)
// ============================================================================

/// Modules that are always available without explicit import
pub const BUILTIN_MODULES: &[(&str, u64)] = &[("io", 1), ("core", 3), ("compiler", 6)];

/// Check if a name is a built-in module
pub fn is_builtin_module(name: &str) -> bool {
    BUILTIN_MODULES.iter().any(|(n, _)| *n == name)
}

/// Get module ID for codegen
#[allow(dead_code)]
pub fn module_id(name: &str) -> Option<u64> {
    BUILTIN_MODULES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, id)| *id)
}

// ============================================================================
// Intrinsic Function Registry
// ============================================================================

/// Intrinsic function signature
#[derive(Clone)]
pub struct Intrinsic {
    #[allow(dead_code)] // Used for future debugging/error messages
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
}

/// Global singleton for compiler intrinsics
static INTRINSICS: OnceLock<HashMap<String, Intrinsic>> = OnceLock::new();

/// Get the global intrinsics registry
fn get_intrinsics() -> &'static HashMap<String, Intrinsic> {
    INTRINSICS.get_or_init(build_intrinsics)
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

macro_rules! intrinsic {
    ($map:expr, $name:expr => () -> $ret:expr) => {
        $map.insert(
            $name.to_string(),
            Intrinsic {
                name: $name.to_string(),
                params: vec![],
                return_type: $ret,
            },
        );
    };
    ($map:expr, $name:expr => ($p1:expr, $t1:expr) -> $ret:expr) => {
        $map.insert(
            $name.to_string(),
            Intrinsic {
                name: $name.to_string(),
                params: vec![($p1.to_string(), $t1)],
                return_type: $ret,
            },
        );
    };
    ($map:expr, $name:expr => ($p1:expr, $t1:expr, $p2:expr, $t2:expr) -> $ret:expr) => {
        $map.insert(
            $name.to_string(),
            Intrinsic {
                name: $name.to_string(),
                params: vec![($p1.to_string(), $t1), ($p2.to_string(), $t2)],
                return_type: $ret,
            },
        );
    };
    ($map:expr, $name:expr => ($p1:expr, $t1:expr, $p2:expr, $t2:expr, $p3:expr, $t3:expr) -> $ret:expr) => {
        $map.insert(
            $name.to_string(),
            Intrinsic {
                name: $name.to_string(),
                params: vec![
                    ($p1.to_string(), $t1),
                    ($p2.to_string(), $t2),
                    ($p3.to_string(), $t3),
                ],
                return_type: $ret,
            },
        );
    };
}

fn build_intrinsics() -> HashMap<String, Intrinsic> {
    let mut m = HashMap::new();

    // Type aliases
    let ptr = AstType::raw_ptr(AstType::U8);
    let ptr64 = AstType::raw_ptr(AstType::U64);
    let overflow_result = AstType::Struct {
        name: "OverflowResult".to_string(),
        fields: vec![
            ("result".to_string(), AstType::I64),
            ("overflow".to_string(), AstType::Bool),
        ],
    };

    // Memory allocation
    intrinsic!(m, "raw_allocate" => ("size", AstType::Usize) -> ptr.clone());
    intrinsic!(m, "raw_deallocate" => ("ptr", ptr.clone(), "size", AstType::Usize) -> AstType::Void);
    intrinsic!(m, "raw_reallocate" => ("ptr", ptr.clone(), "old_size", AstType::Usize, "new_size", AstType::Usize) -> ptr.clone());

    // Pointer operations
    intrinsic!(m, "raw_ptr_offset" => ("ptr", ptr.clone(), "offset", AstType::I64) -> ptr.clone());
    intrinsic!(m, "raw_ptr_cast" => ("ptr", ptr.clone()) -> ptr.clone());
    intrinsic!(m, "ptr_to_int" => ("ptr", ptr.clone()) -> AstType::I64);
    intrinsic!(m, "int_to_ptr" => ("addr", AstType::I64) -> ptr.clone());
    intrinsic!(m, "null_ptr" => () -> ptr.clone());
    intrinsic!(m, "nullptr" => () -> ptr.clone());
    intrinsic!(m, "gep" => ("base_ptr", ptr.clone(), "offset", AstType::I64) -> ptr.clone());
    intrinsic!(m, "gep_struct" => ("struct_ptr", ptr.clone(), "field_index", AstType::I32) -> ptr.clone());

    // Memory operations
    intrinsic!(m, "memcpy" => ("dest", ptr.clone(), "src", ptr.clone(), "size", AstType::Usize) -> AstType::Void);
    intrinsic!(m, "memmove" => ("dest", ptr.clone(), "src", ptr.clone(), "size", AstType::Usize) -> AstType::Void);
    intrinsic!(m, "memset" => ("dest", ptr.clone(), "value", AstType::U8, "size", AstType::Usize) -> AstType::Void);
    intrinsic!(m, "memcmp" => ("ptr1", ptr.clone(), "ptr2", ptr.clone(), "size", AstType::Usize) -> AstType::I32);

    // Type introspection
    intrinsic!(m, "sizeof" => () -> AstType::Usize);
    intrinsic!(m, "alignof" => () -> AstType::Usize);

    // Inline C
    intrinsic!(m, "inline_c" => ("code", AstType::StaticString) -> AstType::Void);

    // Atomic operations
    intrinsic!(m, "atomic_load" => ("ptr", ptr64.clone()) -> AstType::U64);
    intrinsic!(m, "atomic_store" => ("ptr", ptr64.clone(), "value", AstType::U64) -> AstType::Void);
    intrinsic!(m, "atomic_add" => ("ptr", ptr64.clone(), "value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "atomic_sub" => ("ptr", ptr64.clone(), "value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "atomic_cas" => ("ptr", ptr64.clone(), "expected", AstType::U64, "new_value", AstType::U64) -> AstType::Bool);
    intrinsic!(m, "atomic_xchg" => ("ptr", ptr64.clone(), "value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "fence" => () -> AstType::Void);

    // Bitwise operations
    intrinsic!(m, "bswap16" => ("value", AstType::U16) -> AstType::U16);
    intrinsic!(m, "bswap32" => ("value", AstType::U32) -> AstType::U32);
    intrinsic!(m, "bswap64" => ("value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "ctlz" => ("value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "cttz" => ("value", AstType::U64) -> AstType::U64);
    intrinsic!(m, "ctpop" => ("value", AstType::U64) -> AstType::U64);

    // Overflow-checked arithmetic
    intrinsic!(m, "add_overflow" => ("a", AstType::I64, "b", AstType::I64) -> overflow_result.clone());
    intrinsic!(m, "sub_overflow" => ("a", AstType::I64, "b", AstType::I64) -> overflow_result.clone());
    intrinsic!(m, "mul_overflow" => ("a", AstType::I64, "b", AstType::I64) -> overflow_result.clone());

    // Type conversions
    intrinsic!(m, "trunc_f64_i64" => ("value", AstType::F64) -> AstType::I64);
    intrinsic!(m, "trunc_f32_i32" => ("value", AstType::F32) -> AstType::I32);
    intrinsic!(m, "sitofp_i64_f64" => ("value", AstType::I64) -> AstType::F64);
    intrinsic!(m, "uitofp_u64_f64" => ("value", AstType::U64) -> AstType::F64);

    // Debug/trap/panic
    intrinsic!(m, "unreachable" => () -> AstType::Void);
    intrinsic!(m, "trap" => () -> AstType::Void);
    intrinsic!(m, "debugtrap" => () -> AstType::Void);
    intrinsic!(m, "panic" => ("message", AstType::StaticString) -> AstType::Void);

    // Syscalls (Linux x86-64)
    intrinsic!(m, "syscall0" => ("number", AstType::I64) -> AstType::I64);
    intrinsic!(m, "syscall1" => ("number", AstType::I64, "arg0", AstType::I64) -> AstType::I64);
    intrinsic!(m, "syscall2" => ("number", AstType::I64, "arg0", AstType::I64, "arg1", AstType::I64) -> AstType::I64);

    // syscall3-6 need more params - add manually
    m.insert(
        "syscall3".to_string(),
        Intrinsic {
            name: "syscall3".to_string(),
            params: vec![
                ("number".to_string(), AstType::I64),
                ("arg0".to_string(), AstType::I64),
                ("arg1".to_string(), AstType::I64),
                ("arg2".to_string(), AstType::I64),
            ],
            return_type: AstType::I64,
        },
    );
    m.insert(
        "syscall4".to_string(),
        Intrinsic {
            name: "syscall4".to_string(),
            params: vec![
                ("number".to_string(), AstType::I64),
                ("arg0".to_string(), AstType::I64),
                ("arg1".to_string(), AstType::I64),
                ("arg2".to_string(), AstType::I64),
                ("arg3".to_string(), AstType::I64),
            ],
            return_type: AstType::I64,
        },
    );
    m.insert(
        "syscall5".to_string(),
        Intrinsic {
            name: "syscall5".to_string(),
            params: vec![
                ("number".to_string(), AstType::I64),
                ("arg0".to_string(), AstType::I64),
                ("arg1".to_string(), AstType::I64),
                ("arg2".to_string(), AstType::I64),
                ("arg3".to_string(), AstType::I64),
                ("arg4".to_string(), AstType::I64),
            ],
            return_type: AstType::I64,
        },
    );
    m.insert(
        "syscall6".to_string(),
        Intrinsic {
            name: "syscall6".to_string(),
            params: vec![
                ("number".to_string(), AstType::I64),
                ("arg0".to_string(), AstType::I64),
                ("arg1".to_string(), AstType::I64),
                ("arg2".to_string(), AstType::I64),
                ("arg3".to_string(), AstType::I64),
                ("arg4".to_string(), AstType::I64),
                ("arg5".to_string(), AstType::I64),
            ],
            return_type: AstType::I64,
        },
    );

    // FFI/dynamic loading
    intrinsic!(m, "load_library" => ("path", AstType::StaticString) -> ptr.clone());
    intrinsic!(m, "get_symbol" => ("lib_handle", ptr.clone(), "symbol_name", AstType::StaticString) -> ptr.clone());
    intrinsic!(m, "unload_library" => ("lib_handle", ptr.clone()) -> AstType::Void);
    intrinsic!(m, "dlerror" => () -> ptr.clone());

    // IO operations (libc wrappers)
    intrinsic!(m, "libc_write" => ("fd", AstType::I32, "buf", ptr.clone(), "len", AstType::Usize) -> AstType::I64);
    intrinsic!(m, "libc_read" => ("fd", AstType::I32, "buf", ptr.clone(), "len", AstType::Usize) -> AstType::I64);

    // Generic load/store (type determined by context)
    let generic_t = AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    };
    intrinsic!(m, "load" => ("ptr", ptr.clone()) -> generic_t.clone());
    intrinsic!(m, "store" => ("ptr", ptr.clone(), "value", generic_t.clone()) -> AstType::Void);

    // Enum intrinsics
    intrinsic!(m, "discriminant" => ("enum_value", ptr.clone()) -> AstType::I32);
    intrinsic!(m, "set_discriminant" => ("enum_ptr", ptr.clone(), "discriminant", AstType::I32) -> AstType::Void);
    intrinsic!(m, "get_payload" => ("enum_value", ptr.clone()) -> ptr.clone());
    intrinsic!(m, "set_payload" => ("enum_ptr", ptr.clone(), "payload", ptr.clone()) -> AstType::Void);

    m
}
