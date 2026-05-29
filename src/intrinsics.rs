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

mod definitions;

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
    INTRINSICS.get_or_init(definitions::build_intrinsics)
}

/// Get full intrinsic definition (params and return type).
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
        Some(Err(CompileError::Resolution(
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
