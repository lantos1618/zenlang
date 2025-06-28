//! The high-level compiler orchestrator.
//! This module ties the frontend (parser) and the backend (codegen) together.

use crate::ast::Program;
use crate::codegen::llvm::LLVMCompiler;
use crate::error::{CompileError, Result};
use inkwell::context::Context;

/// The main compiler structure.
pub struct Compiler<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Compiles a program using the LLVM backend.
    /// In the future, this could take a `target` enum.
    pub fn compile_llvm(&self, program: &Program) -> Result<String> {
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(program)?;

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler.module.print_to_string().to_string())
    }
} 