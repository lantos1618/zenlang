pub mod calls;
pub mod decl;

use super::LLVMCompiler;
use crate::ast;
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, FunctionValue};

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

    // Result helpers (used by collections, etc.)
    fn create_result_ok(
        &mut self,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        super::stdlib_codegen::helpers::create_result_ok(self, value)
    }

    fn create_result_ok_void(&mut self) -> Result<BasicValueEnum<'ctx>, CompileError> {
        super::stdlib_codegen::helpers::create_result_ok_void(self)
    }

    fn create_result_err(
        &mut self,
        error: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        super::stdlib_codegen::helpers::create_result_err(self, error)
    }
}
