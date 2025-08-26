//! The high-level compiler orchestrator.
//! This module ties the frontend (parser) and the backend (codegen) together.

use crate::ast::{Program, Declaration};
use crate::codegen::llvm::LLVMCompiler;
use crate::error::{CompileError, Result};
use crate::module_system::{ModuleSystem, resolver::ModuleResolver};
use crate::type_system::Monomorphizer;
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
        // Process module imports
        let processed_program = self.process_imports(program)?;
        
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(&processed_program)?;
        
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
        // Process module imports
        let processed_program = self.process_imports(program)?;
        
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(&processed_program)?;
        
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
    
    /// Process module imports and merge imported modules
    fn process_imports(&self, program: &Program) -> Result<Program> {
        let mut module_system = ModuleSystem::new();
        let mut resolver = ModuleResolver::new();
        
        // Process all module imports
        for decl in &program.declarations {
            if let Declaration::ModuleImport { alias, module_path } = decl {
                // Load the module
                module_system.load_module(module_path)?;
                
                // Register the import with the resolver
                resolver.add_import(alias.clone(), module_path.clone());
                
                // Extract and register exports
                if let Some(module) = module_system.get_modules().get(module_path) {
                    let exports = ModuleResolver::extract_exports(module);
                    resolver.add_exports(module_path.clone(), exports);
                }
            }
        }
        
        // Merge all modules into a single program
        let mut merged_program = module_system.merge_programs(program.clone());
        
        // Resolve module references
        resolver.resolve_program(&mut merged_program)
            .map_err(|e| CompileError::InternalError(e, None))?;
        
        Ok(merged_program)
    }
} 