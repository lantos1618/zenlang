use super::LLVMCompiler;
use crate::ast::AstType;
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, BasicValue};

impl<'ctx> LLVMCompiler<'ctx> {
    // Expression compilation methods for literals
    pub fn compile_integer_literal(&self, value: i64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.i64_type().const_int(value as u64, false).into())
    }

    pub fn compile_float_literal(&self, value: f64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.f64_type().const_float(value).into())
    }

    pub fn compile_string_literal(&mut self, val: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr = self.builder.build_global_string_ptr(val, "str")?;
        let ptr_val = ptr.as_pointer_value();
        // Always return the pointer value, don't convert to integer
        // This fixes the issue where string literals were being converted to integers
        // when used as function arguments, breaking string operations
        Ok(ptr_val.into())
    }

    pub fn compile_identifier(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // First check if this is a function name
        if let Some(function) = self.module.get_function(name) {
            // Return the function's address as a pointer value
            Ok(function.as_global_value().as_pointer_value().into())
        } else {
            // It's a variable, get the pointer
            let (ptr, ast_type) = self.get_variable(name)?;
            
            // For pointer types, return the pointer directly
            if matches!(ast_type, AstType::Pointer(_)) {
                Ok(ptr.as_basic_value_enum())
            } else {
                // For non-pointer types, load the value
                let loaded: BasicValueEnum = match &ast_type {
                    AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                        match self.builder.build_load(self.context.ptr_type(inkwell::AddressSpace::default()), ptr, name) {
                            Ok(val) => val.into(),
                            Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                        }
                    }
                    AstType::Function { .. } => {
                        match self.builder.build_load(self.context.ptr_type(inkwell::AddressSpace::default()), ptr, name) {
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
    
    pub fn compile_string_interpolation(&mut self, parts: &[crate::ast::StringPart]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        use crate::ast::StringPart;
        use inkwell::AddressSpace;
        use inkwell::values::BasicMetadataValueEnum;
        
        // First, calculate the total size needed for the string
        // For now, we'll use a simple approach with sprintf for numeric values
        
        // Declare sprintf if not already declared
        let sprintf_fn = self.module.get_function("sprintf").unwrap_or_else(|| {
            let i32_type = self.context.i32_type();
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let fn_type = i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], true);
            self.module.add_function("sprintf", fn_type, None)
        });
        
        // Build the format string and collect interpolated values
        let mut format_string = String::new();
        let mut values: Vec<BasicMetadataValueEnum> = Vec::new();
        
        for part in parts {
            match part {
                StringPart::Literal(s) => {
                    format_string.push_str(s);
                }
                StringPart::Interpolation(expr) => {
                    let val = self.compile_expression(expr)?;
                    
                    // Determine the format specifier based on the actual value type
                    let format_spec = if val.is_int_value() {
                        let int_val = val.into_int_value();
                        match int_val.get_type().get_bit_width() {
                            32 => "%d",
                            64 => "%lld",
                            _ => "%d",
                        }
                    } else if val.is_float_value() {
                        "%.6f"
                    } else if val.is_pointer_value() {
                        // Pointer could be a string - use %s
                        "%s"
                    } else {
                        // Default to string format
                        "%s"
                    };
                    
                    format_string.push_str(format_spec);
                    values.push(val.into());
                }
            }
        }
        
        // Allocate buffer for the result (using a reasonable max size)
        let buffer_size = 1024;
        let buffer_type = self.context.i8_type().array_type(buffer_size);
        let buffer = self.builder.build_alloca(buffer_type, "str_buffer")?;
        let buffer_ptr = self.builder.build_pointer_cast(
            buffer, 
            self.context.ptr_type(AddressSpace::default()),
            "buffer_ptr"
        )?;
        
        // Build the format string
        let format_ptr = self.builder.build_global_string_ptr(&format_string, "format")?;
        
        // Build the sprintf call with all arguments
        let mut sprintf_args: Vec<BasicMetadataValueEnum> = vec![
            buffer_ptr.into(),
            format_ptr.as_pointer_value().into(),
        ];
        sprintf_args.extend(values);
        
        self.builder.build_call(sprintf_fn, &sprintf_args, "sprintf_call")?;
        
        // Return the buffer pointer
        Ok(buffer_ptr.into())
    }
} 