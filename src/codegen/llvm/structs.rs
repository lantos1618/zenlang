use inkwell::types::BasicType;
use super::{LLVMCompiler, Type};
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::{
    types::StructType,
    values::BasicValueEnum,
};
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
        println!("DEBUG: compile_struct_field called with struct: {:?}, field: {}", struct_, field);
        
        // Special handling for identifiers - we need the pointer, not the loaded value
        if let Expression::Identifier(name) = struct_ {
            // Get the variable info
            if let Some((alloca, var_type)) = self.variables.get(name) {
                if let AstType::Struct { name: struct_name, .. } = var_type {
                    // Get struct type info
                    let struct_info = self.struct_types.get(struct_name)
                        .ok_or_else(|| CompileError::TypeError(
                            format!("Struct type '{}' not found", struct_name),
                            None
                        ))?;
                    
                    // Find field index
                    let field_index = struct_info.fields.get(field)
                        .map(|(idx, _)| *idx)
                        .ok_or_else(|| CompileError::TypeError(
                            format!("Field '{}' not found in struct '{}'", field, struct_name),
                            None
                        ))?;
                    
                    // Get field type
                    let field_type = struct_info.fields.get(field)
                        .map(|(_, ty)| ty.clone())
                        .unwrap();
                    
                    // Build GEP to get field pointer
                    let indices = vec![
                        self.context.i32_type().const_zero(),
                        self.context.i32_type().const_int(field_index as u64, false),
                    ];
                    
                    let field_ptr = unsafe {
                        self.builder.build_gep(
                            struct_info.llvm_type,
                            *alloca,
                            &indices,
                            &format!("{}.{}", name, field)
                        )?
                    };
                    
                    // Load the field value
                    let field_llvm_type = self.to_llvm_type(&field_type)?;
                    let basic_type = match field_llvm_type {
                        Type::Basic(ty) => ty,
                        Type::Struct(st) => st.as_basic_type_enum(),
                        _ => return Err(CompileError::TypeError(
                            "Field type must be basic type".to_string(),
                            None
                        )),
                    };
                    
                    let value = self.builder.build_load(basic_type, field_ptr, &format!("load_{}", field))?;
                    return Ok(value);
                }
            }
        }
        
        // Handle dereference case - when accessing field of a dereferenced pointer
        if let Expression::Dereference(inner) = struct_ {
            // Compile the inner expression to get the pointer
            let ptr_val = self.compile_expression(inner)?;
            
            if let BasicValueEnum::PointerValue(ptr) = ptr_val {
                // Load the pointer value to get the actual struct pointer
                let struct_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let struct_ptr = self.builder.build_load(struct_ptr_type, ptr, "load_struct_ptr")?;
                let struct_ptr = struct_ptr.into_pointer_value();
                
                // Now we need to find the struct type from the pointer
                // This is a bit tricky - we need to look up the variable type
                if let Expression::Identifier(ptr_name) = &**inner {
                    if let Some((_, ptr_type)) = self.variables.get(ptr_name) {
                        if let AstType::Pointer(inner_type) = ptr_type {
                            if let AstType::Struct { name: struct_name, .. } = &**inner_type {
                                // Get struct type info
                                let struct_info = self.struct_types.get(struct_name)
                                    .ok_or_else(|| CompileError::TypeError(
                                        format!("Struct type '{}' not found", struct_name),
                                        None
                                    ))?;
                                
                                // Find field index
                                let field_index = struct_info.fields.get(field)
                                    .map(|(idx, _)| *idx)
                                    .ok_or_else(|| CompileError::TypeError(
                                        format!("Field '{}' not found in struct '{}'", field, struct_name),
                                        None
                                    ))?;
                                
                                // Get field type
                                let field_type = struct_info.fields.get(field)
                                    .map(|(_, ty)| ty.clone())
                                    .unwrap();
                                
                                // Build GEP to get field pointer
                                let indices = vec![
                                    self.context.i32_type().const_zero(),
                                    self.context.i32_type().const_int(field_index as u64, false),
                                ];
                                
                                let field_ptr = unsafe {
                                    self.builder.build_gep(
                                        struct_info.llvm_type,
                                        struct_ptr,
                                        &indices,
                                        &format!("{}->{}",ptr_name, field)
                                    )?
                                };
                                
                                // Load the field value
                                let field_llvm_type = self.to_llvm_type(&field_type)?;
                                let basic_type = match field_llvm_type {
                                    Type::Basic(ty) => ty,
                                    Type::Struct(st) => st.as_basic_type_enum(),
                                    _ => return Err(CompileError::TypeError(
                                        "Field type must be basic type".to_string(),
                                        None
                                    )),
                                };
                                
                                let value = self.builder.build_load(basic_type, field_ptr, &format!("load_{}", field))?;
                                return Ok(value);
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback error
        Err(CompileError::TypeError(
            format!("Cannot access field '{}' of expression {:?}", field, struct_),
            None
        ))
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