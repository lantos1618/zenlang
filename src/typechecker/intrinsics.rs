//! Compiler intrinsics type checking
//! Uses crate::intrinsics as the single source of truth for intrinsic types

use crate::ast::AstType;
use crate::error::Result;
use crate::intrinsics as compiler_intrinsics;

/// Check compiler intrinsic function calls and return their type
/// Returns None if not a compiler intrinsic, otherwise returns Ok(type) or error
/// Only "@builtin" module routes to raw intrinsics
pub fn check_compiler_intrinsic(
    module: &str,
    func: &str,
    args_len: usize,
) -> Option<Result<AstType>> {
    if !compiler_intrinsics::is_intrinsic_module(module) {
        return None;
    }

    compiler_intrinsics::check_intrinsic_call(func, args_len)
}
