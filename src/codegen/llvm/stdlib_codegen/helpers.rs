//! Helper functions for stdlib codegen
//! - Result creation helpers

use crate::codegen::llvm::LLVMCompiler;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

/// Helper function to create Result.Ok with a value
pub fn create_result_ok<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    value: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let result_type = compiler.well_known_enum_type();
    let ok_tag = compiler.enum_tag_const(result_type, 0); // Ok = 0

    let mut result = result_type.get_undef();
    result = compiler
        .builder
        .build_insert_value(result, ok_tag, 0, "set_ok")?
        .into_struct_value();
    result = compiler
        .builder
        .build_insert_value(result, value, 1, "set_payload")?
        .into_struct_value();

    Ok(result.into())
}

/// Helper function to create Result.Ok(void)
pub fn create_result_ok_void<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let result_type = compiler.well_known_enum_type();
    let ok_tag = compiler.enum_tag_const(result_type, 0); // Ok = 0

    let mut result = result_type.get_undef();
    result = compiler
        .builder
        .build_insert_value(result, ok_tag, 0, "set_ok")?
        .into_struct_value();
    let null_ptr = compiler
        .context
        .ptr_type(inkwell::AddressSpace::default())
        .const_null();
    result = compiler
        .builder
        .build_insert_value(result, null_ptr, 1, "set_payload")?
        .into_struct_value();

    Ok(result.into())
}

/// Helper function to create Result.Err with an error message
pub fn create_result_err<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    error: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let result_type = compiler.well_known_enum_type();
    let err_tag = compiler.enum_tag_const(result_type, 1); // Err = 1

    let mut result = result_type.get_undef();
    result = compiler
        .builder
        .build_insert_value(result, err_tag, 0, "set_err")?
        .into_struct_value();
    result = compiler
        .builder
        .build_insert_value(result, error, 1, "set_error")?
        .into_struct_value();

    Ok(result.into())
}
