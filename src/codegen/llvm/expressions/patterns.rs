use crate::ast::{Expression, Statement};
use crate::codegen::llvm::LLVMCompiler;
use crate::error::CompileError;
use inkwell::context::Context;
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;

/// Create a zero/null value matching the type of the given value
fn zero_value_like<'ctx>(
    _context: &'ctx Context,
    val: BasicValueEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    match val {
        BasicValueEnum::IntValue(v) => v.get_type().const_zero().into(),
        BasicValueEnum::FloatValue(v) => v.get_type().const_zero().into(),
        BasicValueEnum::PointerValue(v) => v.get_type().const_null().into(),
        BasicValueEnum::StructValue(v) => v.get_type().const_zero().into(),
        BasicValueEnum::ArrayValue(v) => v.get_type().const_zero().into(),
        BasicValueEnum::VectorValue(v) => v.get_type().const_zero().into(),
        BasicValueEnum::ScalableVectorValue(v) => v.get_type().const_zero().into(),
    }
}

/// Build a PHI node for the given value type
fn build_phi_for_value<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
    val: BasicValueEnum<'ctx>,
    name: &str,
) -> Result<Option<inkwell::values::PhiValue<'ctx>>, CompileError> {
    let phi = match val {
        BasicValueEnum::IntValue(v) => Some(compiler.builder.build_phi(v.get_type(), name)?),
        BasicValueEnum::FloatValue(v) => Some(compiler.builder.build_phi(v.get_type(), name)?),
        BasicValueEnum::PointerValue(v) => Some(compiler.builder.build_phi(v.get_type(), name)?),
        BasicValueEnum::StructValue(v) => Some(compiler.builder.build_phi(v.get_type(), name)?),
        BasicValueEnum::ArrayValue(v) => Some(
            compiler
                .builder
                .build_phi(v.get_type().as_basic_type_enum(), name)?,
        ),
        BasicValueEnum::VectorValue(v) => Some(compiler.builder.build_phi(v.get_type(), name)?),
        BasicValueEnum::ScalableVectorValue(v) => {
            Some(compiler.builder.build_phi(v.get_type(), name)?)
        }
    };
    Ok(phi)
}

pub fn compile_pattern_match<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::QuestionMatch { scrutinee, arms } => {
            let scrutinee_val = compiler.compile_expression(scrutinee)?;

            let current_fn = compiler.current_function.ok_or_else(|| {
                CompileError::InternalError(
                    "Pattern matching outside function".to_string(),
                    compiler.get_current_span(),
                )
            })?;

            let merge_block = compiler
                .context
                .append_basic_block(current_fn, "pattern_merge");
            let default_block = compiler
                .context
                .append_basic_block(current_fn, "pattern_default");

            let current_block = compiler.current_block()?;
            let scrutinee_type = compiler.infer_expression_type(scrutinee).ok();

            if arms.is_empty() {
                return Err(CompileError::InternalError(
                    "Pattern match must have at least one arm".to_string(),
                    None,
                ));
            }

            let mut test_blocks = Vec::new();
            let mut body_blocks = Vec::new();

            for i in 0..arms.len() {
                let test_block = compiler
                    .context
                    .append_basic_block(current_fn, &format!("arm_{}_test", i));
                let body_block = compiler
                    .context
                    .append_basic_block(current_fn, &format!("arm_{}_body", i));
                test_blocks.push(test_block);
                body_blocks.push(body_block);
            }

            compiler.builder.position_at_end(current_block);
            compiler
                .builder
                .build_unconditional_branch(test_blocks[0])?;

            let mut incoming_values: Vec<(
                BasicValueEnum<'ctx>,
                inkwell::basic_block::BasicBlock<'ctx>,
            )> = Vec::new();
            let mut first_arm_value: Option<BasicValueEnum<'ctx>> = None;

            for (i, arm) in arms.iter().enumerate() {
                compiler.builder.position_at_end(test_blocks[i]);

                let (matches, bindings) = compiler.compile_pattern_test_with_type(
                    &scrutinee_val,
                    &arm.pattern,
                    scrutinee_type.as_ref(),
                )?;

                let next_test_block = if i < arms.len() - 1 {
                    test_blocks[i + 1]
                } else {
                    default_block
                };

                compiler.builder.build_conditional_branch(
                    matches,
                    body_blocks[i],
                    next_test_block,
                )?;

                compiler.builder.position_at_end(body_blocks[i]);
                compiler.apply_pattern_bindings(&bindings);

                let arm_value = match &arm.body {
                    Expression::Block(stmts) => {
                        if stmts.is_empty() {
                            compiler.context.i32_type().const_int(0, false).into()
                        } else {
                            for stmt in &stmts[..stmts.len() - 1] {
                                compiler.compile_statement(stmt)?;
                            }
                            let last = &stmts[stmts.len() - 1];
                            if let Statement::Expression { expr, .. } = last {
                                compiler.compile_expression(expr)?
                            } else {
                                compiler.compile_statement(last)?;
                                compiler.context.i32_type().const_int(0, false).into()
                            }
                        }
                    }
                    _ => compiler.compile_expression(&arm.body)?,
                };

                if first_arm_value.is_none() {
                    first_arm_value = Some(arm_value);
                }

                let arm_end_block = compiler.current_block()?;
                if arm_end_block.get_terminator().is_none() {
                    // Coerce arm value to match the first arm's type if possible
                    let coerced_value = if let Some(first_val) = first_arm_value {
                        coerce_to_match_type(compiler, arm_value, first_val)?
                    } else {
                        arm_value
                    };
                    incoming_values.push((coerced_value, arm_end_block));
                    compiler.builder.build_unconditional_branch(merge_block)?;
                }
            }

            compiler.builder.position_at_end(default_block);
            let default_value: BasicValueEnum = first_arm_value
                .map(|v| zero_value_like(compiler.context, v))
                .unwrap_or_else(|| compiler.context.i32_type().const_zero().into());
            incoming_values.push((default_value, default_block));
            compiler.builder.build_unconditional_branch(merge_block)?;

            compiler.builder.position_at_end(merge_block);

            match incoming_values.len() {
                0 => Ok(compiler.context.i32_type().const_zero().into()),
                1 => Ok(incoming_values[0].0),
                _ => {
                    let first_val = incoming_values[0].0;
                    if let Some(phi) = build_phi_for_value(compiler, first_val, "match_result")? {
                        for (val, block) in &incoming_values {
                            phi.add_incoming(&[(val, *block)]);
                        }
                        Ok(phi.as_basic_value())
                    } else {
                        Ok(compiler.context.i32_type().const_zero().into())
                    }
                }
            }
        }
        Expression::Conditional { scrutinee, arms } => {
            let scrutinee_val = compiler.compile_expression(scrutinee)?;

            let current_fn = compiler.current_function.ok_or_else(|| {
                CompileError::InternalError(
                    "Conditional outside function".to_string(),
                    compiler.get_current_span(),
                )
            })?;

            let then_block = compiler.context.append_basic_block(current_fn, "then");
            let else_block = compiler.context.append_basic_block(current_fn, "else");
            let merge_block = compiler
                .context
                .append_basic_block(current_fn, "cond_merge");

            compiler.builder.build_conditional_branch(
                scrutinee_val.into_int_value(),
                then_block,
                else_block,
            )?;

            let mut incoming_values: Vec<(
                BasicValueEnum<'ctx>,
                inkwell::basic_block::BasicBlock<'ctx>,
            )> = Vec::new();

            compiler.builder.position_at_end(then_block);
            let then_value = if let Some(then_arm) = arms.first() {
                match &then_arm.body {
                    Expression::Block(stmts) => {
                        if stmts.is_empty() {
                            compiler.context.i32_type().const_int(0, false).into()
                        } else {
                            for stmt in &stmts[..stmts.len() - 1] {
                                compiler.compile_statement(stmt)?;
                            }
                            let last = &stmts[stmts.len() - 1];
                            if let Statement::Expression { expr, .. } = last {
                                compiler.compile_expression(expr)?
                            } else {
                                compiler.compile_statement(last)?;
                                compiler.context.i32_type().const_int(0, false).into()
                            }
                        }
                    }
                    _ => compiler.compile_expression(&then_arm.body)?,
                }
            } else {
                compiler.context.i32_type().const_int(0, false).into()
            };

            let then_end_block = compiler.current_block()?;
            if then_end_block.get_terminator().is_none() {
                incoming_values.push((then_value, then_end_block));
                compiler.builder.build_unconditional_branch(merge_block)?;
            }

            compiler.builder.position_at_end(else_block);
            let else_value = if arms.len() > 1 {
                if let Some(else_arm) = arms.get(1) {
                    match &else_arm.body {
                        Expression::Block(stmts) => {
                            if stmts.is_empty() {
                                compiler.context.i32_type().const_int(0, false).into()
                            } else {
                                for stmt in &stmts[..stmts.len() - 1] {
                                    compiler.compile_statement(stmt)?;
                                }
                                let last = &stmts[stmts.len() - 1];
                                if let Statement::Expression { expr, .. } = last {
                                    compiler.compile_expression(expr)?
                                } else {
                                    compiler.compile_statement(last)?;
                                    compiler.context.i32_type().const_int(0, false).into()
                                }
                            }
                        }
                        _ => compiler.compile_expression(&else_arm.body)?,
                    }
                } else {
                    compiler.context.i32_type().const_int(0, false).into()
                }
            } else {
                compiler.context.i32_type().const_int(0, false).into()
            };

            let else_end_block = compiler.current_block()?;
            if else_end_block.get_terminator().is_none() {
                incoming_values.push((else_value, else_end_block));
                compiler.builder.build_unconditional_branch(merge_block)?;
            }

            compiler.builder.position_at_end(merge_block);

            match incoming_values.len() {
                0 => Ok(compiler.context.i32_type().const_zero().into()),
                1 => Ok(incoming_values[0].0),
                _ => {
                    let first_val = incoming_values[0].0;
                    if let Some(phi) = build_phi_for_value(compiler, first_val, "cond_result")? {
                        for (val, block) in &incoming_values {
                            phi.add_incoming(&[(val, *block)]);
                        }
                        Ok(phi.as_basic_value())
                    } else {
                        Ok(compiler.context.i32_type().const_zero().into())
                    }
                }
            }
        }
        _ => Err(CompileError::InternalError(
            format!("Expected QuestionMatch or Conditional, got {:?}", expr),
            None,
        )),
    }
}

/// Coerce a value to match the type of a target value.
/// This is used to ensure all arms of a pattern match have compatible types for the PHI node.
fn coerce_to_match_type<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    value: BasicValueEnum<'ctx>,
    target: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // If types already match, return as-is
    if value.get_type() == target.get_type() {
        return Ok(value);
    }

    use std::cmp::Ordering;
    match (value, target) {
        // Integer coercion (truncate or extend)
        (BasicValueEnum::IntValue(val), BasicValueEnum::IntValue(tgt)) => {
            let target_type = tgt.get_type();
            match val
                .get_type()
                .get_bit_width()
                .cmp(&target_type.get_bit_width())
            {
                Ordering::Greater => Ok(compiler
                    .builder
                    .build_int_truncate(val, target_type, "trunc")?
                    .into()),
                Ordering::Less => Ok(compiler
                    .builder
                    .build_int_z_extend(val, target_type, "ext")?
                    .into()),
                Ordering::Equal => Ok(value),
            }
        }
        // Float coercion
        (BasicValueEnum::FloatValue(val), BasicValueEnum::FloatValue(tgt)) => {
            let target_type = tgt.get_type();
            if val.get_type() != target_type {
                Ok(compiler
                    .builder
                    .build_float_cast(val, target_type, "fcast")?
                    .into())
            } else {
                Ok(value)
            }
        }
        // Non-struct to struct: use target's zero
        (_, BasicValueEnum::StructValue(tgt)) => Ok(tgt.get_type().const_zero().into()),
        // Struct to int: use target's zero
        (BasicValueEnum::StructValue(_), BasicValueEnum::IntValue(tgt)) => {
            Ok(tgt.get_type().const_zero().into())
        }
        // Fallback: return original value
        _ => Ok(value),
    }
}
