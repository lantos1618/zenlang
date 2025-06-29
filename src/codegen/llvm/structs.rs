use super::{LLVMCompiler, Type};
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::{
    types::{AsTypeRef, StructType},
    values::BasicValueEnum,
};
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StructTypeInfo<'ctx> {
    /// The LLVM struct type
    pub llvm_type: StructType<'ctx>,
    /// Mapping from field name to (index, type)
    pub fields: HashMap<String, (usize, AstType)>,
}
// Move any struct registration, lookup, or field access helpers here as needed. 

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_struct_literal(&mut self, name: &str, fields: &[(String, Expression)]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let (llvm_type, fields_with_info) = {
            let struct_info = self.struct_types.get(name)
                .ok_or_else(|| CompileError::TypeError(
                    format!("Undefined struct type: {}", name), 
                    None
                ))?;
            let mut fields_with_info = Vec::new();
            for (field_name, field_expr) in fields {
                let (field_index, field_type) = struct_info.fields.get(field_name)
                    .ok_or_else(|| CompileError::TypeError(
                        format!("No field '{}' in struct '{}'", field_name, name),
                        None
                    ))?;
                fields_with_info.push((
                    field_name.clone(),
                    *field_index,
                    field_type.clone(),
                    field_expr.clone()
                ));
            }
            fields_with_info.sort_by_key(|&(_, idx, _, _)| idx);
            (struct_info.llvm_type, fields_with_info)
        };
        let alloca = self.builder.build_alloca(
            llvm_type, 
            &format!("{}_tmp", name)
        )?;
        for (field_name, field_index, _field_type, field_expr) in fields_with_info {
            let field_val = self.compile_expression(&field_expr)?;
            let field_ptr = self.builder.build_struct_gep(
                llvm_type,
                alloca,
                field_index as u32,
                &format!("{}_ptr", field_name)
            )?;
            self.builder.build_store(field_ptr, field_val)?;
        }
        match self.builder.build_load(
            llvm_type,
            alloca,
            &format!("{}_val", name)
        ) {
            Ok(val) => Ok(val),
            Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
        }
    }

    pub fn compile_struct_field(&mut self, struct_: &Expression, field: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let struct_val = self.compile_expression(struct_)?;
        let (struct_type, struct_name) = match struct_val.get_type() {
            BasicTypeEnum::StructType(ty) => {
                let struct_name = self.struct_types.iter()
                    .find(|(_, info)| info.llvm_type == ty)
                    .map(|(name, _)| name.clone())
                    .ok_or_else(|| CompileError::TypeError(
                        format!("Unknown struct type: {:?}", ty),
                        None
                    ))?;
                (ty, struct_name)
            },
            BasicTypeEnum::PointerType(ptr_type) => {
                // This is a pointer to a struct, we need to find the struct type
                // First, try to find a struct type that matches this pointer type
                let struct_name = self.struct_types.iter()
                    .find(|(_, _info)| {
                        // Check if this struct's pointer type matches our pointer type
                        let struct_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        struct_ptr_type.as_type_ref() == ptr_type.as_type_ref()
                    })
                    .map(|(name, _)| name.clone());
                
                if let Some(name) = struct_name {
                    let struct_info = self.struct_types.get(&name)
                        .ok_or_else(|| CompileError::TypeError(
                            format!("Undefined struct type: {}", name),
                            None
                        ))?;
                    (struct_info.llvm_type, name)
                } else {
                    // If we can't find a direct match, assume it's a pointer to a struct
                    // and try to find any struct type (this is a fallback)
                    let first_struct = self.struct_types.iter().next()
                        .ok_or_else(|| CompileError::TypeError(
                            "No struct types defined".to_string(),
                            None
                        ))?;
                    (first_struct.1.llvm_type, first_struct.0.clone())
                }
            },
            _ => {
                return Err(CompileError::TypeMismatch {
                    expected: "struct or struct pointer".to_string(),
                    found: format!("{:?}", struct_val.get_type()),
                    span: None,
                });
            }
        };
        
        // Get field information
        let (field_index, field_type) = {
            let struct_info = self.struct_types.get(&struct_name)
                .ok_or_else(|| CompileError::TypeError(
                    format!("Undefined struct type: {}", struct_name),
                    None
                ))?;
            let (field_index, field_type) = struct_info.fields.get(field)
                .ok_or_else(|| CompileError::TypeError(
                    format!("No field '{}' in struct '{}'", field, struct_name),
                    None
                ))?;
            (*field_index, field_type.clone())
        };
        
        // Handle the struct value - if it's a pointer, use it directly; otherwise, create a temporary
        let struct_ptr = if struct_val.is_pointer_value() {
            struct_val.into_pointer_value()
        } else {
            let alloca = self.builder.build_alloca(
                struct_val.get_type(),
                "struct_field_tmp"
            )?;
            self.builder.build_store(alloca, struct_val)?;
            alloca
        };
        
        let field_basic_type = match self.to_llvm_type(&field_type)? {
            Type::Basic(ty) => ty,
            _ => return Err(CompileError::TypeError(
                "Expected basic type for struct field".to_string(),
                None
            )),
        };
        
        let field_ptr = self.builder.build_struct_gep(
            struct_type,
            struct_ptr,
            field_index as u32,
            &format!("{}_field_{}_ptr", struct_name, field)
        )?;
        
        match self.builder.build_load(
            field_basic_type,
            field_ptr,
            &format!("{}_val", field)
        ) {
            Ok(val) => Ok(val),
            Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
        }
    }

    pub fn compile_struct_field_assignment(&mut self, struct_alloca: inkwell::values::PointerValue<'ctx>, field_name: &str, value: BasicValueEnum<'ctx>) -> Result<(), CompileError> {
        // Find the struct type info by matching the pointer type
        let struct_type_info = self.struct_types.values().find(|info| {
            info.llvm_type.ptr_type(inkwell::AddressSpace::default()) == struct_alloca.get_type()
        }).ok_or_else(|| CompileError::TypeError("Struct type info not found for assignment".to_string(), None))?;
        let struct_type = struct_type_info.llvm_type;
        let field_index = struct_type_info.fields.get(field_name).map(|(index, _)| *index)
            .ok_or_else(|| CompileError::TypeError(format!("Field '{}' not found in struct", field_name), None))?;
        // Create GEP to get the field pointer
        let field_ptr = unsafe {
            self.builder.build_struct_gep(struct_type, struct_alloca, field_index as u32, "field_ptr")
        }.map_err(|e| CompileError::from(e))?;
        // Store the value to the field
        self.builder.build_store(field_ptr, value).map_err(|e| CompileError::from(e))?;
        Ok(())
    }
} 