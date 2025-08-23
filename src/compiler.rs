//! The high-level compiler orchestrator.
//! This module ties the frontend (parser) and the backend (codegen) together.

use crate::ast::Program;
use crate::codegen::llvm::LLVMCompiler;
use crate::error::{CompileError, Result};
use crate::type_system::monomorphization::Monomorphizer;
use inkwell::context::Context;
use inkwell::module::Module;

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
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(program)?;
        
        eprintln!("Compiler: monomorphized program has {} declarations", monomorphized_program.declarations.len());
        for decl in &monomorphized_program.declarations {
            if let crate::ast::Declaration::Function(func) = decl {
                eprintln!("  -> {}", func.name);
            }
        }
        
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(&monomorphized_program)?;

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler.module.print_to_string().to_string())
    }

    /// Gets the LLVM module after compilation for execution engine creation.
    pub fn get_module(&self, program: &Program) -> Result<Module<'ctx>> {
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(program)?;
        
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(&monomorphized_program)?;

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler.module)
    }
} 