use super::super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

pub fn compile_binary_operation<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::BinaryOp { op, left, right } => {
            compiler.compile_binary_operation(op, left, right)
        }
        _ => Err(CompileError::InternalError(
            format!("Expected BinaryOp, got {:?}", expr),
            None,
        )),
    }
}
