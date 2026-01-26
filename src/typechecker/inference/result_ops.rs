//! Result type operations inference

use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};
use crate::typechecker::TypeChecker;

/// Infer the type of a .raise() call on a Result<T, E>
pub fn infer_raise_type(checker: &mut TypeChecker, expr: &Expression) -> Result<AstType> {
    let result_type = checker.infer_expression_type(expr)?;
    match result_type {
        AstType::Generic {
            ref name,
            ref type_args,
        } if checker.well_known.is_result(name) && !type_args.is_empty() => {
            Ok(type_args[0].clone())
        }
        _ => Err(CompileError::TypeError(
            format!(
                ".raise() can only be used on Result<T, E> types, found: {:?}",
                result_type
            ),
            checker.get_current_span(),
        )),
    }
}
