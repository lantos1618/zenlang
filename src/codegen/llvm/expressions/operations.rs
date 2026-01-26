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

pub fn compile_type_cast<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::TypeCast {
            expr: inner_expr,
            target_type,
        } => {
            // Compile the inner expression to get the source value
            let source_value = compiler.compile_expression(inner_expr)?;

            // Convert target AstType to LLVM type
            let target_llvm_type = compiler.to_llvm_type(target_type)?;
            let target_basic_type = compiler.expect_basic_type(target_llvm_type)?;

            // Perform the type cast based on source and target types
            perform_type_cast(compiler, source_value, target_basic_type)
        }
        _ => Err(CompileError::InternalError(
            format!("Expected TypeCast, got {:?}", expr),
            None,
        )),
    }
}

fn perform_type_cast<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    source_value: BasicValueEnum<'ctx>,
    target_type: inkwell::types::BasicTypeEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    use inkwell::types::BasicTypeEnum;

    // Determine source and target type categories
    let (source_is_int, source_is_float, source_is_ptr) = (
        source_value.is_int_value(),
        source_value.is_float_value(),
        source_value.is_pointer_value(),
    );

    let (target_is_int, target_is_float, target_is_ptr) = match target_type {
        BasicTypeEnum::IntType(_) => (true, false, false),
        BasicTypeEnum::FloatType(_) => (false, true, false),
        BasicTypeEnum::PointerType(_) => (false, false, true),
        _ => (false, false, false),
    };

    // Handle different conversion cases
    match (
        source_is_int,
        source_is_float,
        source_is_ptr,
        target_is_int,
        target_is_float,
        target_is_ptr,
    ) {
        // Int to Int: resize with sign extension
        (true, false, false, true, false, false) => {
            let source_int = source_value.into_int_value();
            let target_int_type = target_type.into_int_type();

            let source_width = source_int.get_type().get_bit_width();
            let target_width = target_int_type.get_bit_width();

            if source_width == target_width {
                // Same size, no conversion needed
                Ok(source_value)
            } else if source_width < target_width {
                // Extend to larger type (sign extend for signed types, zero extend for bool)
                let result = if source_width == 1 {
                    // Bool to int: zero extend
                    compiler.builder.build_int_z_extend(
                        source_int,
                        target_int_type,
                        "bool_to_int",
                    )?
                } else {
                    // Regular int extension (sign extend)
                    compiler.builder.build_int_s_extend(
                        source_int,
                        target_int_type,
                        "int_extend",
                    )?
                };
                Ok(result.into())
            } else {
                // Truncate to smaller type
                let result = compiler.builder.build_int_truncate(
                    source_int,
                    target_int_type,
                    "int_trunc",
                )?;
                Ok(result.into())
            }
        }

        // Float to Float: convert between f32 and f64
        (false, true, false, false, true, false) => {
            let source_float = source_value.into_float_value();
            let target_float_type = target_type.into_float_type();

            if source_float.get_type() == target_float_type {
                // Same type, no conversion needed
                Ok(source_value)
            } else {
                // Check if we need to extend or truncate based on type comparison
                // f32 is smaller than f64
                let is_f32_source = source_float.get_type() == compiler.context.f32_type();
                let is_f64_target = target_float_type == compiler.context.f64_type();

                if is_f32_source && is_f64_target {
                    // f32 to f64: extend
                    let result = compiler.builder.build_float_ext(
                        source_float,
                        target_float_type,
                        "float_ext",
                    )?;
                    Ok(result.into())
                } else {
                    // f64 to f32: truncate (or other float conversion)
                    let result = compiler.builder.build_float_trunc(
                        source_float,
                        target_float_type,
                        "float_trunc",
                    )?;
                    Ok(result.into())
                }
            }
        }

        // Int to Float: signed int to float conversion
        (true, false, false, false, true, false) => {
            let source_int = source_value.into_int_value();
            let target_float_type = target_type.into_float_type();
            let result = compiler.builder.build_signed_int_to_float(
                source_int,
                target_float_type,
                "int_to_float",
            )?;
            Ok(result.into())
        }

        // Float to Int: float to signed int conversion
        (false, true, false, true, false, false) => {
            let source_float = source_value.into_float_value();
            let target_int_type = target_type.into_int_type();
            let result = compiler.builder.build_float_to_signed_int(
                source_float,
                target_int_type,
                "float_to_int",
            )?;
            Ok(result.into())
        }

        // Int to Pointer: int to ptr conversion
        (true, false, false, false, false, true) => {
            let source_int = source_value.into_int_value();
            let target_ptr_type = target_type.into_pointer_type();
            let result =
                compiler
                    .builder
                    .build_int_to_ptr(source_int, target_ptr_type, "int_to_ptr")?;
            Ok(result.into())
        }

        // Pointer to Int: ptr to int conversion
        (false, false, true, true, false, false) => {
            let source_ptr = source_value.into_pointer_value();
            let target_int_type = target_type.into_int_type();
            let result =
                compiler
                    .builder
                    .build_ptr_to_int(source_ptr, target_int_type, "ptr_to_int")?;
            Ok(result.into())
        }

        // Pointer to Pointer: pointer cast for pointer type changes
        (false, false, true, false, false, true) => {
            let source_ptr = source_value.into_pointer_value();
            let target_ptr_type = target_type.into_pointer_type();
            let result =
                compiler
                    .builder
                    .build_pointer_cast(source_ptr, target_ptr_type, "ptr_cast")?;
            Ok(result.into())
        }

        // Default case: unsupported conversion
        _ => Err(CompileError::TypeMismatch {
            expected: format!(
                "Cannot cast from {:?} to {:?}",
                source_value.get_type(),
                target_type
            ),
            found: "unsupported type conversion".to_string(),
            span: compiler.get_current_span(),
        }),
    }
}
