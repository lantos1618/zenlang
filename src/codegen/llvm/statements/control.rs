use crate::ast::{LoopKind, Statement};
use crate::codegen::llvm::LLVMCompiler;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

pub fn compile_return<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &crate::ast::Expression,
) -> Result<(), CompileError> {
    let value = compiler.compile_expression(expr)?;

    // Execute all deferred expressions before returning
    compiler.execute_deferred_expressions()?;

    // Cast return value to match function return type using shared helper
    let final_value = if let Some(func) = compiler.current_function {
        if let Some(expected_ret_type) = func.get_type().get_return_type() {
            compiler.cast_value_to_type(value, expected_ret_type)?
        } else {
            value
        }
    } else {
        value
    };

    compiler.builder.build_return(Some(&final_value))?;
    Ok(())
}

pub fn compile_loop<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statement: &Statement,
) -> Result<(), CompileError> {
    match statement {
        Statement::Loop { kind, body, .. } => {
            match kind {
                LoopKind::Infinite => {
                    // Create blocks for infinite loop
                    let current_fn = compiler.current_fn()?;
                    let loop_body = compiler.context.append_basic_block(current_fn, "loop_body");
                    let after_loop_block = compiler
                        .context
                        .append_basic_block(current_fn, "after_loop");

                    // Push loop context for break/continue
                    compiler.loop_stack.push((loop_body, after_loop_block));

                    // Jump to loop body
                    compiler
                        .builder
                        .build_unconditional_branch(loop_body)
                        .map_err(CompileError::from)?;
                    compiler.builder.position_at_end(loop_body);

                    // Compile body
                    for stmt in body {
                        compiler.compile_statement(stmt)?;
                    }

                    // Loop back if no terminator
                    let current_block = compiler.current_block()?;
                    if current_block.get_terminator().is_none() {
                        compiler
                            .builder
                            .build_unconditional_branch(loop_body)
                            .map_err(CompileError::from)?;
                    }

                    compiler.loop_stack.pop();
                    compiler.builder.position_at_end(after_loop_block);
                    Ok(())
                }
                LoopKind::Condition(cond_expr) => {
                    // Create blocks
                    let current_fn = compiler.current_fn()?;
                    let loop_header = compiler
                        .context
                        .append_basic_block(current_fn, "loop_header");
                    let loop_body = compiler.context.append_basic_block(current_fn, "loop_body");
                    let after_loop_block = compiler
                        .context
                        .append_basic_block(current_fn, "after_loop");

                    compiler.loop_stack.push((loop_header, after_loop_block));

                    // Jump to header
                    compiler
                        .builder
                        .build_unconditional_branch(loop_header)
                        .map_err(CompileError::from)?;
                    compiler.builder.position_at_end(loop_header);

                    // Evaluate condition
                    let cond_value = compiler.compile_expression(cond_expr)?;
                    if let BasicValueEnum::IntValue(int_val) = cond_value {
                        if int_val.get_type().get_bit_width() == 1 {
                            compiler
                                .builder
                                .build_conditional_branch(int_val, loop_body, after_loop_block)
                                .map_err(CompileError::from)?;
                        } else {
                            let zero = int_val.get_type().const_zero();
                            let condition = compiler
                                .builder
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    int_val,
                                    zero,
                                    "loop_condition",
                                )
                                .map_err(CompileError::from)?;
                            compiler
                                .builder
                                .build_conditional_branch(condition, loop_body, after_loop_block)
                                .map_err(CompileError::from)?;
                        }
                    } else {
                        return Err(CompileError::TypeError(
                            "Loop condition must be an integer".to_string(),
                            None,
                        ));
                    }

                    // Compile body
                    compiler.builder.position_at_end(loop_body);
                    for stmt in body {
                        compiler.compile_statement(stmt)?;
                    }

                    // Loop back if no terminator
                    let current_block = compiler.current_block()?;
                    if current_block.get_terminator().is_none() {
                        compiler
                            .builder
                            .build_unconditional_branch(loop_header)
                            .map_err(CompileError::from)?;
                    }

                    compiler.loop_stack.pop();
                    compiler.builder.position_at_end(after_loop_block);
                    Ok(())
                }
            }
        }
        _ => Err(CompileError::InternalError(
            "Expected Loop statement".to_string(),
            None,
        )),
    }
}

pub fn compile_break<'ctx>(compiler: &mut LLVMCompiler<'ctx>) -> Result<(), CompileError> {
    if let Some((_continue_target, break_target)) = compiler.loop_stack.last() {
        compiler
            .builder
            .build_unconditional_branch(*break_target)
            .map_err(CompileError::from)?;
        Ok(())
    } else {
        Err(CompileError::TypeError(
            "break statement outside of loop".to_string(),
            None,
        ))
    }
}

pub fn compile_continue<'ctx>(compiler: &mut LLVMCompiler<'ctx>) -> Result<(), CompileError> {
    if let Some((continue_target, _break_target)) = compiler.loop_stack.last() {
        compiler
            .builder
            .build_unconditional_branch(*continue_target)
            .map_err(CompileError::from)?;
        Ok(())
    } else {
        Err(CompileError::TypeError(
            "continue statement outside of loop".to_string(),
            None,
        ))
    }
}
