pub mod calls;
pub mod decl;

use super::LLVMCompiler;
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, FunctionValue};
use crate::ast;

impl<'ctx> LLVMCompiler<'ctx> {
    // Function declaration/definition
    pub fn declare_external_function(
        &mut self,
        ext_func: &ast::ExternalFunction,
    ) -> Result<(), CompileError> {
        decl::declare_external_function(self, ext_func)
    }

    pub fn declare_function(
        &mut self,
        function: &ast::Function,
    ) -> Result<FunctionValue<'ctx>, CompileError> {
        decl::declare_function(self, function)
    }

    pub fn compile_function_body(&mut self, function: &ast::Function) -> Result<(), CompileError> {
        decl::compile_function_body(self, function)
    }

    pub fn define_and_compile_function(
        &mut self,
        function: &ast::Function,
    ) -> Result<FunctionValue<'ctx>, CompileError> {
        decl::define_and_compile_function(self, function)
    }

    // Function calls
    pub fn compile_function_call(
        &mut self,
        name: &str,
        args: &[ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        calls::compile_function_call(self, name, args)
    }

}
