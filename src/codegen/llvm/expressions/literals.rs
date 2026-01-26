use super::super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

pub fn compile_literal<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::Integer8(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Integer16(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Integer32(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Integer64(v) => compiler.compile_integer_literal(*v),
        Expression::Unsigned8(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Unsigned16(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Unsigned32(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Unsigned64(v) => compiler.compile_integer_literal(*v as i64),
        Expression::Float32(v) => compiler.compile_float_literal(*v as f64),
        Expression::Float64(v) => compiler.compile_float_literal(*v),
        Expression::Boolean(v) => Ok(compiler
            .context
            .bool_type()
            .const_int(
                if *v {
                    1
                } else {
                    0
                },
                false,
            )
            .into()),
        Expression::Unit => Ok(compiler.context.i32_type().const_zero().into()),
        Expression::String(s) => compiler.compile_string_literal(s),
        _ => Err(CompileError::InternalError(
            format!("Expected literal, got {:?}", expr),
            None,
        )),
    }
}

pub fn compile_identifier<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::Identifier(name) => compiler.compile_identifier(name),
        _ => Err(CompileError::InternalError(
            format!("Expected Identifier, got {:?}", expr),
            None,
        )),
    }
}
