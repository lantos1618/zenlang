use super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::{
    types::{AsTypeRef, BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, PointerValue},
    AddressSpace,
};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_address_of(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                let (alloca, _type_) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::UndeclaredVariable(name.clone(), None))?;
                Ok(alloca.as_basic_value_enum())
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
        
        // Try to determine the element type from the pointer
        // For struct pointers, we need to find the struct type
        let element_type: BasicTypeEnum = if let BasicTypeEnum::PointerType(_) = ptr_val.get_type() {
            // Check if this is a pointer to a struct
            let struct_name = self.struct_types.iter()
                .find(|(_, _info)| {
                    let struct_ptr_type = self.context.ptr_type(AddressSpace::default());
                    struct_ptr_type.as_type_ref() == ptr_val.get_type().as_type_ref()
                })
                .map(|(name, _)| name.clone());
            
            if let Some(name) = struct_name {
                let struct_info = self.struct_types.get(&name)
                    .ok_or_else(|| CompileError::TypeError(
                        format!("Undefined struct type: {}", name),
                        None
                    ))?;
                struct_info.llvm_type.as_basic_type_enum()
            } else {
                self.context.i64_type().as_basic_type_enum()
            }
        } else {
            self.context.i64_type().as_basic_type_enum()
        };
        
        Ok(self.builder.build_load(element_type, ptr, "load_tmp")?.into())
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