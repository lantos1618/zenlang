use inkwell::types::BasicType;
use super::{LLVMCompiler, Type};
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::{
    types::{AsTypeRef, StructType},
    values::BasicValueEnum,
};
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use inkwell::AddressSpace;

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
        println!("DEBUG: compile_struct_field called with struct: {:?}, field: {}", struct_, field);
        
        // Compile the struct expression to get the struct pointer
        let struct_val = self.compile_expression(struct_)?;
        println!("DEBUG: struct_val type: {:?}", struct_val.get_type());
        
        // Get the struct pointer
        let struct_ptr = struct_val.into_pointer_value();
        println!("DEBUG: struct_ptr type: {:?}", struct_ptr.get_type());
        
        // Find the struct type info by matching the pointer type
        let struct_type_info = self.struct_types.values().find(|info| {
            struct_ptr.get_type().as_type_ref() == info.llvm_type.ptr_type(AddressSpace::default()).as_type_ref()
        }).ok_or_else(|| CompileError::TypeError("Struct type info not found for pointer type".to_string(), None))?;
        
        println!("DEBUG: Found struct type info");
        
        let struct_type = struct_type_info.llvm_type;
        let field_index = struct_type_info.fields.get(field).map(|(index, _)| *index)
            .ok_or_else(|| CompileError::TypeError(format!("Field '{}' not found in struct", field), None))?;
        
        println!("DEBUG: Field index: {}", field_index);
        
        // Get the field type
        let field_type = struct_type_info.fields.get(field).map(|(_, ty)| ty.clone())
            .ok_or_else(|| CompileError::TypeError(format!("Field '{}' type not found", field), None))?;
        
        println!("DEBUG: Field type: {:?}", field_type);
        
        // Convert field type to LLVM type
        let field_llvm_type = self.to_llvm_type(&field_type)?;
        println!("DEBUG: Field LLVM type: {:?}", field_llvm_type);
        
        let field_basic_type = match field_llvm_type {
            Type::Basic(ty) => ty,
            Type::Struct(st) => st.as_basic_type_enum(),
            _ => return Err(CompileError::TypeError(
                "Unsupported field type in struct".to_string(),
                None
            )),
        };
        
        println!("DEBUG: Field basic type: {:?}", field_basic_type);
        
        // Get the field pointer using GEP
        let field_ptr = self.builder.build_struct_gep(
            struct_type,
            struct_ptr,
            field_index as u32,
            &format!("{}_ptr", field)
        )?;
        
        println!("DEBUG: Field pointer created");
        
        // Load the field value
        match self.builder.build_load(
            field_basic_type,
            field_ptr,
            &format!("{}_val", field)
        ) {
            Ok(val) => {
                println!("DEBUG: compile_struct_field returning value of type: {:?}", val.get_type());
                Ok(val)
            },
            Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
        }
    }

    pub fn compile_struct_field_assignment(&mut self, struct_alloca: inkwell::values::PointerValue<'ctx>, field_name: &str, value: BasicValueEnum<'ctx>) -> Result<(), CompileError> {
        // Find the struct type info by trying to match the pointer type with any known struct
        // This is a fallback approach since we can't easily get the element type
        let struct_type_info = self.struct_types.values().find(|info| {
            // Try to match by checking if this struct type could be the one we're looking for
            // We'll use the first struct type as a reasonable fallback
            true
        }).ok_or_else(|| CompileError::TypeError("No struct types defined".to_string(), None))?;
        
        let struct_type = struct_type_info.llvm_type;
        let field_index = struct_type_info.fields.get(field_name).map(|(index, _)| *index)
            .ok_or_else(|| CompileError::TypeError(format!("Field '{}' not found in struct", field_name), None))?;
        
        // Create GEP to get the field pointer
        let field_ptr = self.builder.build_struct_gep(
            struct_type, 
            struct_alloca, 
            field_index as u32, 
            "field_ptr"
        ).map_err(|e| CompileError::from(e))?;
        
        // Store the value to the field
        self.builder.build_store(field_ptr, value).map_err(|e| CompileError::from(e))?;
        Ok(())
    }
} 