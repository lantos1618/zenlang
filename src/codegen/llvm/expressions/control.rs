use super::super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

/// Compile a loop expression: loop(() { ... })
/// Loops can return values via break with value
pub fn compile_loop<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let Expression::Loop { body } = expr {
        // Create blocks for the loop
        let current_fn = compiler.current_fn()?;
        let loop_body = compiler.context.append_basic_block(current_fn, "loop_body");
        let after_loop_block = compiler
            .context
            .append_basic_block(current_fn, "after_loop");

        // Extract the actual body from the closure wrapper if present
        let actual_body = match body.as_ref() {
            Expression::Closure {
                body: closure_body, ..
            } => closure_body.as_ref(),
            other => other,
        };

        // Infer return type from break expressions in the body
        let return_type = compiler.infer_expression_type(actual_body)?;

        // Check if loop returns a value
        let has_return_value = !matches!(return_type, crate::ast::AstType::Void);

        // Push loop context for break/continue
        compiler.loop_stack.push((loop_body, after_loop_block));

        // Jump to loop body
        compiler
            .builder
            .build_unconditional_branch(loop_body)
            .map_err(CompileError::from)?;
        compiler.builder.position_at_end(loop_body);

        // Compile actual body (unwrapped from closure)
        let body_value = compiler.compile_expression(actual_body)?;

        // If body didn't terminate (no break/return), loop back
        let current_block = compiler.current_block()?;
        if current_block.get_terminator().is_none() {
            compiler
                .builder
                .build_unconditional_branch(loop_body)
                .map_err(CompileError::from)?;
        }

        compiler.loop_stack.pop();
        compiler.builder.position_at_end(after_loop_block);

        // If loop returns a value, we need a phi node
        if has_return_value {
            Ok(body_value)
        } else {
            Ok(compiler.context.i64_type().const_zero().into())
        }
    } else {
        Err(CompileError::InternalError(
            "Expected Loop expression".to_string(),
            None,
        ))
    }
}

/// Compile a break expression: break or break(value)
/// Break can optionally return a value which becomes the loop's return value
pub fn compile_break<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let Expression::Break { label: _, value } = expr {
        // Get break target first to avoid borrow checker issues
        let break_target = compiler
            .loop_stack
            .last()
            .map(|(_continue_target, break_target)| *break_target)
            .ok_or_else(|| {
                CompileError::TypeError(
                    "break expression outside of loop".to_string(),
                    compiler.get_current_span(),
                )
            })?;

        // If break has a value, compile it
        let break_value = if let Some(val_expr) = value {
            compiler.compile_expression(val_expr)?
        } else {
            // No value - return void
            compiler.context.i64_type().const_zero().into()
        };

        // Branch to break target
        compiler
            .builder
            .build_unconditional_branch(break_target)
            .map_err(CompileError::from)?;

        // Return the break value (though we've already branched)
        Ok(break_value)
    } else {
        Err(CompileError::InternalError(
            "Expected Break expression".to_string(),
            None,
        ))
    }
}

/// Compile a continue expression: continue
pub fn compile_continue<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let Expression::Continue { label: _ } = expr {
        // Get continue target first to avoid borrow checker issues
        let continue_target = compiler
            .loop_stack
            .last()
            .map(|(continue_target, _break_target)| *continue_target)
            .ok_or_else(|| {
                CompileError::TypeError(
                    "continue expression outside of loop".to_string(),
                    compiler.get_current_span(),
                )
            })?;

        // Branch to continue target
        compiler
            .builder
            .build_unconditional_branch(continue_target)
            .map_err(CompileError::from)?;

        // Return void (though we've already branched)
        Ok(compiler.context.i64_type().const_zero().into())
    } else {
        Err(CompileError::InternalError(
            "Expected Continue expression".to_string(),
            None,
        ))
    }
}

/// Compile a return expression: return value
/// Returns from the current function
pub fn compile_return<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let Expression::Return(value_expr) = expr {
        // Compile the return value
        let return_value = compiler.compile_expression(value_expr)?;

        // Execute all deferred expressions before returning
        compiler.execute_deferred_expressions()?;

        // Check if function returns void
        let is_void_function = if let Some(func) = compiler.current_function {
            func.get_type().get_return_type().is_none()
        } else {
            false
        };

        // Build return instruction - use None for void functions
        if is_void_function {
            compiler.builder.build_return(None)?;
            // For void functions, return a dummy value
            Ok(compiler.context.i64_type().const_zero().into())
        } else {
            // Cast return value to match function return type using shared helper
            let final_value = if let Some(func) = compiler.current_function {
                if let Some(expected_ret_type) = func.get_type().get_return_type() {
                    compiler.cast_value_to_type(return_value, expected_ret_type)?
                } else {
                    return_value
                }
            } else {
                return_value
            };

            compiler.builder.build_return(Some(&final_value))?;
            Ok(final_value)
        }
    } else {
        Err(CompileError::InternalError(
            "Expected Return expression".to_string(),
            None,
        ))
    }
}

/// Compile a range expression: (start..end) or (start..=end)
/// Creates a Range struct value
pub fn compile_range_expression<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let Expression::Range {
        start,
        end,
        inclusive,
    } = expr
    {
        // Compile start and end values
        let start_value = compiler.compile_expression(start)?;
        let end_value = compiler.compile_expression(end)?;

        // Ensure both are integers (ranges work with integers)
        let start_int = match start_value {
            BasicValueEnum::IntValue(iv) => iv,
            _ => {
                return Err(CompileError::TypeError(
                    "Range start must be an integer".to_string(),
                    None,
                ));
            }
        };

        let end_int = match end_value {
            BasicValueEnum::IntValue(iv) => iv,
            _ => {
                return Err(CompileError::TypeError(
                    "Range end must be an integer".to_string(),
                    None,
                ));
            }
        };

        // Create Range struct type: { i64 start, i64 end, bool inclusive }
        let range_struct_type = compiler.context.struct_type(
            &[
                start_int.get_type().into(),
                end_int.get_type().into(),
                compiler.context.bool_type().into(),
            ],
            false,
        );

        // Create inclusive bool value
        let inclusive_bool = compiler
            .context
            .bool_type()
            .const_int(*inclusive as u64, false);

        // Create a single alloca for the range struct
        let range_alloca = compiler
            .builder
            .build_alloca(range_struct_type, "range_alloca")?;

        // Get pointers to each field
        let start_ptr =
            compiler
                .builder
                .build_struct_gep(range_struct_type, range_alloca, 0, "start_ptr")?;
        let end_ptr =
            compiler
                .builder
                .build_struct_gep(range_struct_type, range_alloca, 1, "end_ptr")?;
        let inclusive_ptr = compiler.builder.build_struct_gep(
            range_struct_type,
            range_alloca,
            2,
            "inclusive_ptr",
        )?;

        // Store values into struct fields
        compiler.builder.build_store(start_ptr, start_int)?;
        compiler.builder.build_store(end_ptr, end_int)?;
        compiler
            .builder
            .build_store(inclusive_ptr, inclusive_bool)?;

        // Load the struct value
        let range_value = compiler
            .builder
            .build_load(range_struct_type, range_alloca, "range")?;

        Ok(range_value)
    } else {
        Err(CompileError::InternalError(
            "Expected Range expression".to_string(),
            None,
        ))
    }
}
