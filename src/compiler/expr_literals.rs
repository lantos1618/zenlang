use super::*;

impl<'ctx> Compiler<'ctx> {
    pub fn compile_integer_literal(&self, value: i64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.i64_type().const_int(value as u64, false).into())
    }

    pub fn compile_float_literal(&self, value: f64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.f64_type().const_float(value).into())
    }

    pub fn compile_string_literal(&mut self, val: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr = self.builder.build_global_string_ptr(val, "str")?;
        let ptr_val = ptr.as_pointer_value();
        if let Some(func) = self.current_function {
            if let Some(return_type) = func.get_type().get_return_type() {
                if return_type.is_int_type() {
                    let ptr_int = self.builder.build_ptr_to_int(
                        ptr_val,
                        return_type.into_int_type(),
                        "str_to_int"
                    )?;
                    return Ok(ptr_int.into());
                }
            }
        }
        Ok(ptr_val.into())
    }

    pub fn compile_identifier(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // First check if this is a function name
        if let Some(function) = self.module.get_function(name) {
            // Return the function's address as a pointer value
            Ok(function.as_global_value().as_pointer_value().into())
        } else {
            // It's a variable, load it normally
            let (ptr, ast_type) = self.get_variable(name)?;
            let loaded: BasicValueEnum = match &ast_type {
                AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                    match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
                AstType::Function { .. } => {
                    match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
                _ => {
                    let elem_type = self.to_llvm_type(&ast_type)?;
                    let basic_type = self.expect_basic_type(elem_type)?;
                    match self.builder.build_load(basic_type, ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
            };
            Ok(loaded)
        }
    }
} 