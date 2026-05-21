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

mod registry;

use self::registry::build_intrinsics;

// ============================================================================
// Intrinsic Module Recognition
// ============================================================================

/// The prefix used for raw intrinsic calls (e.g., `@builtin.raw_allocate`).
pub const INTRINSIC_PREFIX: &str = "@builtin";

/// The stdlib module that wraps raw intrinsics (stdlib/compiler.zen).
pub const COMPILER_MODULE: &str = "compiler";

/// Check if a module name routes directly to raw compiler intrinsics.
/// Only `@builtin` - the raw intrinsic prefix used in stdlib/compiler.zen.
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
