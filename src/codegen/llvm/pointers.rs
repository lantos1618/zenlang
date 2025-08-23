use super::LLVMCompiler;
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::{
    types::BasicType,
    values::{BasicValue, BasicValueEnum},
};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_address_of(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                let (alloca, ast_type) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::UndeclaredVariable(name.clone(), None))?;
                
                // If the variable is already a pointer type, return it directly
                if matches!(ast_type, AstType::Pointer(_)) {
                    Ok(alloca.as_basic_value_enum())
                } else {
                    // For non-pointer variables, return the address
                    Ok(alloca.as_basic_value_enum())
                }
            }
            _ => Err(CompileError::UnsupportedFeature(
                "AddressOf only supported for identifiers".to_string(),
                None,
            )),
        }
    }

    pub fn compile_dereference(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr_val = self.compile_expression(expr)?;
        if !ptr_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "pointer".to_string(),
                found: format!("{:?}", ptr_val.get_type()),
                span: None,
            });
        }
        let ptr = ptr_val.into_pointer_value();
        
        // Get the pointed-to type from the variable table
        let pointed_type = if let Expression::Identifier(name) = expr {
            if let Ok((_, ast_type)) = self.get_variable(name) {
                if let AstType::Pointer(inner) = ast_type {
                    Some(*inner.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        let llvm_type = if let Some(ref ast_type) = pointed_type {
            self.to_llvm_type(ast_type)?
        } else {
            // Final fallback: i64
            super::Type::Basic(self.context.i64_type().as_basic_type_enum())
        };
        
        // If expr is an identifier for a pointer variable, load twice
        if let (Expression::Identifier(name), Some(ref ast_type)) = (expr, pointed_type.as_ref()) {
            // First load: get the address stored in the pointer variable
            let llvm_ptr_type = match self.to_llvm_type(ast_type)? {
                super::Type::Basic(basic_type) => basic_type.ptr_type(inkwell::AddressSpace::default()),
                super::Type::Struct(struct_type) => struct_type.ptr_type(inkwell::AddressSpace::default()),
                _ => return Err(CompileError::TypeError("Cannot dereference non-basic/non-struct pointer type".to_string(), None)),
            };
            let address = self.builder.build_load(llvm_ptr_type, ptr, "deref_ptr_addr")?;
            let address_ptr = address.into_pointer_value();
            // Second load: get the value at that address
            return match llvm_type {
                super::Type::Basic(basic_type) => Ok(self.builder.build_load(basic_type, address_ptr, "load_tmp")?.into()),
                super::Type::Struct(struct_type) => Ok(self.builder.build_load(struct_type, address_ptr, "load_struct_tmp")?.into()),
                _ => Err(CompileError::TypeError("Cannot dereference non-basic/non-struct type".to_string(), None)),
            };
        }
        
        match llvm_type {
            super::Type::Basic(basic_type) => Ok(self.builder.build_load(basic_type, ptr, "load_tmp")?.into()),
            super::Type::Struct(struct_type) => Ok(self.builder.build_load(struct_type, ptr, "load_struct_tmp")?.into()),
            _ => Err(CompileError::TypeError("Cannot dereference non-basic/non-struct type".to_string(), None)),
        }
    }

    pub fn compile_pointer_offset(&mut self, pointer: &Expression, offset: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let base_val = self.compile_expression(pointer)?;
        let offset_val = self.compile_expression(offset)?;
        if !base_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "pointer for pointer offset base".to_string(),
                found: format!("{:?}", base_val.get_type()),
                span: None,
            });
        }
        if !offset_val.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "integer for pointer offset value".to_string(),
                found: format!("{:?}", offset_val.get_type()),
                span: None,
            });
        }
        unsafe {
            let ptr_type = base_val.get_type();
            let _offset = offset_val.into_int_value();
            let ptr = base_val.into_pointer_value();
            Ok(self.builder.build_gep(ptr_type, ptr, &[self.context.i32_type().const_int(0, false)], "gep_tmp")?.into())
        }
    }
} 