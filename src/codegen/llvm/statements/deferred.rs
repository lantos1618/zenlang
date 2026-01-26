use crate::ast::Statement;
use crate::codegen::llvm::LLVMCompiler;
use crate::error::CompileError;

pub fn compile_defer<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statement: &Statement,
) -> Result<(), CompileError> {
    // Extract deferred statement
    match statement {
        Statement::Defer { .. } => {
            // Push the deferred statement onto the defer stack (LIFO order)
            compiler.defer_stack.push(crate::ast::Expression::Unit);
            // The actual execution happens in execute_deferred_expressions
            Ok(())
        }
        Statement::ThisDefer { expr, .. } => {
            // Push the deferred expression onto the defer stack
            compiler.defer_stack.push(expr.clone());
            Ok(())
        }
        _ => Err(CompileError::InternalError(
            "Expected Defer or ThisDefer statement".to_string(),
            None,
        )),
    }
}

pub fn execute_deferred_expressions<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
) -> Result<(), CompileError> {
    // Execute deferred expressions in reverse order (LIFO)
    while let Some(expr) = compiler.defer_stack.pop() {
        compiler.compile_expression(&expr)?;
    }
    Ok(())
}
