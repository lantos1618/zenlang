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
    intrinsic!(m, "strlen", ("str" => AstType::StaticString) -> AstType::Usize, "Get length of a static string", "String");
    intrinsic!(m, "static_string_ptr", ("str" => AstType::StaticString) -> ptr.clone(), "Get pointer to static string data", "String");

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

/// Well-known types that have special compiler semantics.
///
/// IMPORTANT: Only types that REQUIRE compiler support belong here:
/// - Option/Result: Pattern exhaustiveness, ? operator, .raise()
/// - Ptr types: Pointer codegen, dereference, null checks
///
/// Regular stdlib types (Vec, HashMap, String, Range, etc.) do NOT belong here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WellKnownType {
    /// Option<T> - nullable type (pattern matching, ? operator)
    Option,
    /// Result<T, E> - error handling type (pattern matching, .raise())
    Result,
    /// Ptr<T> - immutable pointer (dereference codegen)
    Ptr,
    /// MutPtr<T> - mutable pointer (dereference codegen)
    MutPtr,
    /// RawPtr<T> - raw/unsafe pointer (FFI, unsafe codegen)
    RawPtr,
}

/// Well-known enum variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WellKnownVariant {
    /// Option::Some(T)
    Some,
    /// Option::None
    None,
    /// Result::Ok(T)
    Ok,
    /// Result::Err(E)
    Err,
}

/// Registry of well-known types and their variants
#[derive(Debug, Clone)]
pub struct WellKnownTypes {
    /// Map from type name to well-known type
    types: HashMap<String, WellKnownType>,
    /// Map from variant name to (parent type, variant)
    variants: HashMap<String, (WellKnownType, WellKnownVariant)>,
}

impl WellKnownTypes {
    /// Create a new registry with all well-known types registered.
    pub fn new() -> Self {
        let mut wkt = Self {
            types: HashMap::with_capacity(5),
            variants: HashMap::with_capacity(4),
        };

        wkt.types.insert("Option".into(), WellKnownType::Option);
        wkt.types.insert("Result".into(), WellKnownType::Result);
        wkt.types.insert("Ptr".into(), WellKnownType::Ptr);
        wkt.types.insert("MutPtr".into(), WellKnownType::MutPtr);
        wkt.types.insert("RawPtr".into(), WellKnownType::RawPtr);

        wkt.variants.insert(
            "Some".into(),
            (WellKnownType::Option, WellKnownVariant::Some),
        );
        wkt.variants.insert(
            "None".into(),
            (WellKnownType::Option, WellKnownVariant::None),
        );
        wkt.variants
            .insert("Ok".into(), (WellKnownType::Result, WellKnownVariant::Ok));
        wkt.variants
            .insert("Err".into(), (WellKnownType::Result, WellKnownVariant::Err));

        wkt
    }

    // ========================================================================
    // Type checks
    // ========================================================================

    /// Get the well-known type for a name, if any
    #[inline]
    pub fn get_type(&self, name: &str) -> Option<WellKnownType> {
        self.types.get(name).copied()
    }

    /// Check if a type name is Option
    #[inline]
    pub fn is_option(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Option)
    }

    /// Check if a type name is Result
    #[inline]
    pub fn is_result(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Result)
    }

    /// Check if a type name is any pointer type (Ptr, MutPtr, RawPtr)
    #[inline]
    pub fn is_ptr(&self, name: &str) -> bool {
        matches!(
            self.get_type(name),
            Some(WellKnownType::Ptr | WellKnownType::MutPtr | WellKnownType::RawPtr)
        )
    }

    /// Check if a type name is an immutable pointer (Ptr)
    #[inline]
    pub fn is_immutable_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Ptr)
    }

    /// Check if a type name is a mutable pointer (MutPtr)
    #[inline]
    pub fn is_mutable_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::MutPtr)
    }

    /// Check if a type name is a raw pointer (RawPtr)
    #[inline]
    pub fn is_raw_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::RawPtr)
    }

    /// Check if a type name is Option or Result (types with success/failure variants)
    #[inline]
    #[allow(dead_code)]
    pub fn is_option_or_result(&self, name: &str) -> bool {
        matches!(
            self.get_type(name),
            Some(WellKnownType::Option | WellKnownType::Result)
        )
    }

    // ========================================================================
    // Variant checks
    // ========================================================================

    /// Get the well-known variant info for a name, if any
    #[inline]
    pub fn get_variant(&self, name: &str) -> Option<(WellKnownType, WellKnownVariant)> {
        self.variants.get(name).copied()
    }

    /// Check if a variant name belongs to Option (Some or None)
    #[inline]
    pub fn is_option_variant(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((WellKnownType::Option, _)))
    }

    /// Check if a variant name belongs to Result (Ok or Err)
    #[inline]
    pub fn is_result_variant(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((WellKnownType::Result, _)))
    }

    /// Check if a variant name is Some
    #[inline]
    pub fn is_some(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Some)))
    }

    /// Check if a variant name is None
    #[inline]
    pub fn is_none(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::None)))
    }

    /// Check if a variant name is Ok
    #[inline]
    pub fn is_ok(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Ok)))
    }

    /// Check if a variant name is Err
    #[inline]
    pub fn is_err(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Err)))
    }

    /// Get the parent type for a variant
    #[inline]
    pub fn get_variant_parent(&self, variant_name: &str) -> Option<WellKnownType> {
        self.get_variant(variant_name).map(|(parent, _)| parent)
    }

    /// Get the canonical type name for a variant's parent
    #[inline]
    pub fn get_variant_parent_name(&self, variant_name: &str) -> Option<&'static str> {
        self.get_variant_parent(variant_name).map(|t| match t {
            WellKnownType::Option => "Option",
            WellKnownType::Result => "Result",
            WellKnownType::Ptr => "Ptr",
            WellKnownType::MutPtr => "MutPtr",
            WellKnownType::RawPtr => "RawPtr",
        })
    }

    // ========================================================================
    // Canonical name getters
    // ========================================================================

    #[inline]
    pub fn option_name(&self) -> &'static str {
        "Option"
    }

    #[inline]
    pub fn result_name(&self) -> &'static str {
        "Result"
    }

    #[inline]
    pub fn ptr_name(&self) -> &'static str {
        "Ptr"
    }

    #[inline]
    pub fn mut_ptr_name(&self) -> &'static str {
        "MutPtr"
    }

    #[inline]
    pub fn raw_ptr_name(&self) -> &'static str {
        "RawPtr"
    }

    // ========================================================================
    // Variant name getters
    // ========================================================================

    #[inline]
    pub fn some_name(&self) -> &'static str {
        "Some"
    }

    #[inline]
    pub fn none_name(&self) -> &'static str {
        "None"
    }

    #[inline]
    pub fn ok_name(&self) -> &'static str {
        "Ok"
    }

    #[inline]
    pub fn err_name(&self) -> &'static str {
        "Err"
    }

    /// Get discriminant tag for a variant (for codegen)
    #[inline]
    #[allow(dead_code)]
    pub fn get_variant_tag(&self, variant_name: &str) -> Option<u64> {
        match self.get_variant(variant_name) {
            Some((_, WellKnownVariant::Some)) => Some(0),
            Some((_, WellKnownVariant::Ok)) => Some(0),
            Some((_, WellKnownVariant::None)) => Some(1),
            Some((_, WellKnownVariant::Err)) => Some(1),
            None => None,
        }
    }
}

impl Default for WellKnownTypes {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static instance for use in parser and other contexts
pub fn well_known() -> &'static WellKnownTypes {
    static INSTANCE: OnceLock<WellKnownTypes> = OnceLock::new();
    INSTANCE.get_or_init(WellKnownTypes::new)
}
