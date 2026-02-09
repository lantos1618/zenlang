use super::{symbols, LLVMCompiler, Type};
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::types::BasicType;
use inkwell::{types::StructType, values::BasicValueEnum};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StructTypeInfo<'ctx> {
    pub llvm_type: StructType<'ctx>,
    pub fields: HashMap<String, (usize, AstType)>,
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

struct FieldInfo {
    index: usize,
    ast_type: AstType,
}

impl<'ctx> LLVMCompiler<'ctx> {
    /// Get field info from struct type
    fn get_field_info(
        &self,
        struct_name: &str,
        field_name: &str,
    ) -> Result<FieldInfo, CompileError> {
        let span = self.get_current_span();
        let struct_info = self.struct_types.get(struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct type '{}' not found", struct_name),
                span.clone(),
            )
        })?;

        let (index, ast_type) = struct_info.fields.get(field_name).ok_or_else(|| {
            CompileError::TypeError(
                format!(
                    "Field '{}' not found in struct '{}'",
                    field_name, struct_name
                ),
                span,
            )
        })?;

        Ok(FieldInfo {
            index: *index,
            ast_type: ast_type.clone(),
        })
    }

    /// Convert LLVM type to BasicTypeEnum for loads
    fn to_basic_type(
        &self,
        llvm_type: &Type<'ctx>,
    ) -> Result<inkwell::types::BasicTypeEnum<'ctx>, CompileError> {
        match llvm_type {
            Type::Basic(ty) => Ok(*ty),
            Type::Struct(st) => Ok(st.as_basic_type_enum()),
            _ => Err(CompileError::TypeError(
                "Field type must be basic type".to_string(),
                self.get_current_span(),
            )),
        }
    }

    /// Load a struct field given pointer and field info
    fn load_struct_field(
        &mut self,
        struct_ptr: inkwell::values::PointerValue<'ctx>,
        struct_name: &str,
        field_name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let struct_info = self.struct_types.get(struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct type '{}' not found", struct_name),
                self.get_current_span(),
            )
        })?;

        let field_info = self.get_field_info(struct_name, field_name)?;

        let field_ptr = self.builder.build_struct_gep(
            struct_info.llvm_type,
            struct_ptr,
            field_info.index as u32,
            &format!("{}_{}_ptr", struct_name, field_name),
        )?;

        let field_llvm_type = self.to_llvm_type(&field_info.ast_type)?;
        let basic_type = self.to_basic_type(&field_llvm_type)?;

        Ok(self
            .builder
            .build_load(basic_type, field_ptr, &format!("load_{}", field_name))?)
    }

    /// Get struct name from AstType
    fn struct_name_from_type(&self, ast_type: &AstType) -> Option<String> {
        match ast_type {
            AstType::Struct { name, .. } => Some(name.clone()),
            AstType::Generic { name, .. } if self.struct_types.contains_key(name) => {
                Some(name.clone())
            }
            _ => None,
        }
    }
}

// ============================================================================
// Struct Literal Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, Expression)],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Try to register struct from stdlib if not found locally
        self.ensure_struct_type(name)?;

        let (llvm_type, fields_with_info) = {
            let struct_info = self.struct_types.get(name).ok_or_else(|| {
                CompileError::TypeError(
                    format!("Undefined struct type: {}", name),
                    self.get_current_span(),
                )
            })?;

            let mut fields_with_info = Vec::new();
            for (field_name, field_expr) in fields {
                let (idx, ty) = struct_info.fields.get(field_name).ok_or_else(|| {
                    CompileError::TypeError(
                        format!("No field '{}' in struct '{}'", field_name, name),
                        self.get_current_span(),
                    )
                })?;
                fields_with_info.push((field_name.clone(), *idx, ty.clone(), field_expr.clone()));
            }
            fields_with_info.sort_by_key(|(_, idx, _, _)| *idx);
            (struct_info.llvm_type, fields_with_info)
        };

        let alloca = self
            .builder
            .build_alloca(llvm_type, &format!("{}_tmp", name))?;

        for (field_name, field_index, field_type, field_expr) in fields_with_info {
            let field_val = self.compile_expression(&field_expr)?;
            let field_ptr = self.builder.build_struct_gep(
                llvm_type,
                alloca,
                field_index as u32,
                &format!("{}_ptr", field_name),
            )?;

            let field_llvm_type = self.to_llvm_type(&field_type)?;
            match self.to_basic_type(&field_llvm_type) {
                Ok(expected_type) => {
                    self.coercing_store(
                        field_val,
                        field_ptr,
                        expected_type,
                        &format!("struct field '{}.{}'", name, field_name),
                    )?;
                }
                Err(_) => {
                    self.builder.build_store(field_ptr, field_val)?;
                }
            }
        }

        Ok(self
            .builder
            .build_load(llvm_type, alloca, &format!("{}_val", name))?)
    }
}

// ============================================================================
// Struct Field Access
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_struct_field(
        &mut self,
        struct_: &Expression,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Try identifier access first (most common case)
        if let Expression::Identifier(name) = struct_ {
            return self.compile_identifier_field_access(name, field);
        }

        // Handle dereference case
        if let Expression::Dereference(inner) = struct_ {
            return self.compile_deref_field_access(inner, field);
        }

        // Handle nested MemberAccess
        if let Expression::MemberAccess { object, member } = struct_ {
            return self.compile_nested_field_access(object, member, field);
        }

        // Fallback: compile expression and access field
        self.compile_general_field_access(struct_, field)
    }

    fn compile_identifier_field_access(
        &mut self,
        name: &str,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Check if this is an enum type
        if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
            if enum_info.variant_indices.contains_key(field) {
                return self.compile_enum_variant(name, field, &None);
            }
            return Err(CompileError::TypeError(
                format!("Unknown variant '{}' for enum '{}'", field, name),
                self.get_current_span(),
            ));
        }

        // Get variable info
        let (alloca, var_type) = self.get_variable_info(name)?;

        // Handle stdlib module
        if var_type == AstType::StdModule {
            return self.compile_module_field_access(name, field);
        }

        // Handle pointer to struct
        if let Some(inner_type) = var_type.ptr_inner() {
            if let Some(struct_name) = self.struct_name_from_type(inner_type) {
                return self.compile_ptr_struct_field(alloca, name, &struct_name, field);
            }
        }

        // Handle regular struct
        if let Some(struct_name) = self.struct_name_from_type(&var_type) {
            return self.load_struct_field(alloca, &struct_name, field);
        }

        Err(CompileError::TypeError(
            format!("'{}' is not a struct type, it's {:?}", name, var_type),
            self.get_current_span(),
        ))
    }

    fn get_variable_info(
        &self,
        name: &str,
    ) -> Result<(inkwell::values::PointerValue<'ctx>, AstType), CompileError> {
        if let Some(var_info) = self.variables.get(name) {
            return Ok((var_info.pointer, var_info.ast_type.clone()));
        }

        if let Some(symbol) = self.symbols.lookup(name) {
            if let symbols::Symbol::Variable(ptr) = symbol {
                let inferred = if name == "self" {
                    self.infer_self_type()
                } else {
                    AstType::Void
                };
                return Ok((*ptr, inferred));
            }
            return Err(CompileError::TypeError(
                format!("'{}' is not a variable", name),
                self.get_current_span(),
            ));
        }

        Err(CompileError::UndeclaredVariable(
            name.to_string(),
            self.get_current_span(),
        ))
    }

    fn infer_self_type(&self) -> AstType {
        if let Some(impl_type) = &self.current_impl_type {
            if let Some(struct_info) = self.struct_types.get(impl_type) {
                let fields: Vec<_> = struct_info
                    .fields
                    .iter()
                    .map(|(n, (_, t))| (n.clone(), t.clone()))
                    .collect();
                return AstType::ptr(AstType::Struct {
                    name: impl_type.clone(),
                    fields,
                });
            }
            return AstType::Struct {
                name: impl_type.clone(),
                fields: vec![],
            };
        }

        if let Some((struct_name, _)) = self.struct_types.iter().next() {
            return AstType::Struct {
                name: struct_name.clone(),
                fields: vec![],
            };
        }

        AstType::Void
    }

    fn compile_module_field_access(
        &mut self,
        _name: &str,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match field {
            "init" => Ok(self.context.i64_type().const_int(1, false).into()),
            _ => Err(CompileError::TypeError(
                format!("Unknown module method '{}'", field),
                self.get_current_span(),
            )),
        }
    }

    fn compile_ptr_struct_field(
        &mut self,
        alloca: inkwell::values::PointerValue<'ctx>,
        var_name: &str,
        struct_name: &str,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let struct_info = self.struct_types.get(struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct type '{}' not found", struct_name),
                self.get_current_span(),
            )
        })?;
        let field_info = self.get_field_info(struct_name, field)?;

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let struct_ptr =
            self.builder
                .build_load(ptr_type, alloca, &format!("load_{}_ptr", var_name))?;
        let struct_ptr = struct_ptr.into_pointer_value();

        let field_ptr = self.builder.build_struct_gep(
            struct_info.llvm_type,
            struct_ptr,
            field_info.index as u32,
            &format!("{}_{}_ptr", var_name, field),
        )?;

        let field_llvm_type = self.to_llvm_type(&field_info.ast_type)?;
        let basic_type = self.to_basic_type(&field_llvm_type)?;
        Ok(self.builder.build_load(
            basic_type,
            field_ptr,
            &format!("load_{}_{}", var_name, field),
        )?)
    }

    fn compile_deref_field_access(
        &mut self,
        inner: &Expression,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr_val = self.compile_expression(inner)?;
        let BasicValueEnum::PointerValue(ptr) = ptr_val else {
            return Err(CompileError::TypeError(
                "Expected pointer value".to_string(),
                self.get_current_span(),
            ));
        };

        let struct_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let struct_ptr = self
            .builder
            .build_load(struct_ptr_type, ptr, "load_struct_ptr")?;
        let struct_ptr = struct_ptr.into_pointer_value();

        if let Expression::Identifier(ptr_name) = inner {
            let struct_name = self.variables.get(ptr_name).and_then(|v| {
                v.ast_type.ptr_inner().and_then(|inner_type| {
                    if let AstType::Struct { name, .. } = inner_type {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
            });

            if let Some(name) = struct_name {
                return self.load_struct_field(struct_ptr, &name, field);
            }
        }

        Err(CompileError::TypeError(
            "Cannot determine struct type for dereference".to_string(),
            self.get_current_span(),
        ))
    }

    fn compile_nested_field_access(
        &mut self,
        object: &Expression,
        inner_member: &str,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Get parent struct info
        let Expression::Identifier(name) = object else {
            let inner_val = self.compile_struct_field(object, inner_member)?;
            return self.compile_nested_from_value(object, inner_member, inner_val, field);
        };

        let var_info = self.variables.get(name).ok_or_else(|| {
            CompileError::UndeclaredVariable(name.clone(), self.get_current_span())
        })?;

        let parent_struct_name =
            self.struct_name_from_type(&var_info.ast_type)
                .ok_or_else(|| {
                    CompileError::TypeError(
                        format!("'{}' is not a struct type", name),
                        self.get_current_span(),
                    )
                })?;

        let parent_struct_ptr = var_info.pointer;

        // Get parent struct llvm type and inner field info
        let parent_info = self.struct_types.get(&parent_struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", parent_struct_name),
                self.get_current_span(),
            )
        })?;

        let inner_field_info = self.get_field_info(&parent_struct_name, inner_member)?;

        // Get nested struct name from inner field type
        let nested_struct_name = self
            .struct_name_from_type(&inner_field_info.ast_type)
            .ok_or_else(|| {
                CompileError::TypeError(
                    format!("Field '{}' is not a struct type", inner_member),
                    self.get_current_span(),
                )
            })?;

        let nested_info = self.struct_types.get(&nested_struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", nested_struct_name),
                self.get_current_span(),
            )
        })?;

        let final_field_info = self.get_field_info(&nested_struct_name, field)?;

        // Build chained GEPs
        let inner_ptr = self.builder.build_struct_gep(
            parent_info.llvm_type,
            parent_struct_ptr,
            inner_field_info.index as u32,
            &format!("{}_{}_ptr", parent_struct_name, inner_member),
        )?;

        let nested_ptr = self.builder.build_struct_gep(
            nested_info.llvm_type,
            inner_ptr,
            final_field_info.index as u32,
            &format!("{}_{}_{}_ptr", parent_struct_name, inner_member, field),
        )?;

        let field_llvm_type = self.to_llvm_type(&final_field_info.ast_type)?;
        let basic_type = self.to_basic_type(&field_llvm_type)?;
        Ok(self
            .builder
            .build_load(basic_type, nested_ptr, &format!("load_{}", field))?)
    }

    fn compile_nested_from_value(
        &mut self,
        object: &Expression,
        inner_member: &str,
        inner_val: BasicValueEnum<'ctx>,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Infer nested struct name from parent
        let Expression::Identifier(name) = object else {
            return Err(CompileError::TypeError(
                "Cannot infer struct type for nested access".to_string(),
                self.get_current_span(),
            ));
        };

        let var_info = self.variables.get(name).ok_or_else(|| {
            CompileError::UndeclaredVariable(name.clone(), self.get_current_span())
        })?;

        let parent_name = self
            .struct_name_from_type(&var_info.ast_type)
            .ok_or_else(|| {
                CompileError::TypeError(
                    format!("'{}' is not a struct type", name),
                    self.get_current_span(),
                )
            })?;

        let parent_info = self.struct_types.get(&parent_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", parent_name),
                self.get_current_span(),
            )
        })?;

        let (_, field_type) = parent_info.fields.get(inner_member).ok_or_else(|| {
            CompileError::TypeError(
                format!("Field '{}' not found", inner_member),
                self.get_current_span(),
            )
        })?;

        let nested_name = self.struct_name_from_type(field_type).ok_or_else(|| {
            CompileError::TypeError(
                format!("Field '{}' is not a struct type", inner_member),
                self.get_current_span(),
            )
        })?;

        let nested_info = self.struct_types.get(&nested_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", nested_name),
                self.get_current_span(),
            )
        })?;

        let field_info = self.get_field_info(&nested_name, field)?;

        // Store value and access field
        let temp = self
            .builder
            .build_alloca(nested_info.llvm_type, &format!("temp_{}", nested_name))?;
        self.builder.build_store(temp, inner_val)?;

        let field_ptr = self.builder.build_struct_gep(
            nested_info.llvm_type,
            temp,
            field_info.index as u32,
            &format!("{}_{}_ptr", nested_name, field),
        )?;

        let field_llvm_type = self.to_llvm_type(&field_info.ast_type)?;
        let basic_type = self.to_basic_type(&field_llvm_type)?;
        Ok(self
            .builder
            .build_load(basic_type, field_ptr, &format!("load_{}", field))?)
    }

    fn compile_general_field_access(
        &mut self,
        struct_: &Expression,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let struct_val = self.compile_expression(struct_)?;
        let struct_name = self.infer_struct_type_from_value(&struct_val, struct_)?;

        let struct_info = self.struct_types.get(&struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", struct_name),
                self.get_current_span(),
            )
        })?;
        let field_info = self.get_field_info(&struct_name, field)?;

        let struct_ptr = if struct_val.is_pointer_value() {
            struct_val.into_pointer_value()
        } else {
            let temp = self
                .builder
                .build_alloca(struct_info.llvm_type, &format!("temp_{}", struct_name))?;
            self.builder.build_store(temp, struct_val)?;
            temp
        };

        let field_ptr = self.builder.build_struct_gep(
            struct_info.llvm_type,
            struct_ptr,
            field_info.index as u32,
            &format!("field_{}_ptr", field),
        )?;

        let field_llvm_type = self.to_llvm_type(&field_info.ast_type)?;
        let basic_type = self.to_basic_type(&field_llvm_type)?;
        Ok(self
            .builder
            .build_load(basic_type, field_ptr, &format!("load_{}", field))?)
    }
}

// ============================================================================
// Type Inference
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn infer_struct_type_from_value(
        &mut self,
        _val: &BasicValueEnum<'ctx>,
        expr: &Expression,
    ) -> Result<String, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    let effective = var_info.ast_type.ptr_inner().unwrap_or(&var_info.ast_type);
                    if let Some(n) = self.struct_name_from_type(effective) {
                        return Ok(n);
                    }
                }
                Err(CompileError::TypeError(
                    format!("Unknown variable '{}'", name),
                    self.get_current_span(),
                ))
            }
            Expression::StructLiteral { name, .. } => Ok(name.clone()),
            Expression::FunctionCall { module, name, .. } => {
                // Check TypeContext first, then local cache
                let lookup_name = match module {
                    Some(m) => format!("{}.{}", m, name),
                    None => name.clone(),
                };
                let return_type = self
                    .type_ctx
                    .get_function_return_type(&lookup_name)
                    .or_else(|| self.function_types.get(&lookup_name).cloned())
                    .or_else(|| self.function_types.get(name.as_str()).cloned());
                if let Some(AstType::Struct { name: sn, .. }) = return_type {
                    return Ok(sn.clone());
                }
                Err(CompileError::TypeError(
                    format!("Function '{}' does not return a struct", lookup_name),
                    self.get_current_span(),
                ))
            }
            Expression::MemberAccess { object, member } => {
                let obj_type = self.infer_expression_type(object)?;
                let obj_struct = self
                    .struct_name_from_type(obj_type.ptr_inner().unwrap_or(&obj_type))
                    .ok_or_else(|| {
                        CompileError::TypeError(
                            "Cannot infer object type".to_string(),
                            self.get_current_span(),
                        )
                    })?;

                if let Some(info) = self.struct_types.get(&obj_struct) {
                    if let Some((_, field_type)) = info.fields.get(member) {
                        if let Some(n) = self.struct_name_from_type(field_type) {
                            return Ok(n);
                        }
                    }
                }
                Err(CompileError::TypeError(
                    format!("Field '{}' is not a struct", member),
                    self.get_current_span(),
                ))
            }
            Expression::PointerDereference(ptr_expr) => {
                if let Expression::Identifier(name) = ptr_expr.as_ref() {
                    if let Some(var_info) = self.variables.get(name) {
                        if let Some(inner) = var_info.ast_type.ptr_inner() {
                            if let Some(n) = self.struct_name_from_type(inner) {
                                return Ok(n);
                            }
                        }
                    }
                }
                Err(CompileError::TypeError(
                    "Cannot infer struct type from pointer".to_string(),
                    self.get_current_span(),
                ))
            }
            _ => Err(CompileError::TypeError(
                format!("Cannot infer struct type from expression: {:?}", expr),
                self.get_current_span(),
            )),
        }
    }
}

// ============================================================================
// Struct Field Assignment
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_struct_field_assignment(
        &mut self,
        struct_alloca: inkwell::values::PointerValue<'ctx>,
        field_name: &str,
        value: BasicValueEnum<'ctx>,
        struct_name: &str,
    ) -> Result<(), CompileError> {
        let struct_info = self.struct_types.get(struct_name).ok_or_else(|| {
            CompileError::TypeError(
                format!("Struct '{}' not found", struct_name),
                self.get_current_span(),
            )
        })?;

        let field_info = self.get_field_info(struct_name, field_name)?;

        let field_ptr = self.builder.build_struct_gep(
            struct_info.llvm_type,
            struct_alloca,
            field_info.index as u32,
            "field_ptr",
        )?;

        let field_llvm_type = self.to_llvm_type(&field_info.ast_type)?;
        match self.to_basic_type(&field_llvm_type) {
            Ok(expected_type) => {
                self.coercing_store(
                    value,
                    field_ptr,
                    expected_type,
                    &format!("struct field '{}.{}'", struct_name, field_name),
                )?;
            }
            Err(_) => {
                self.builder.build_store(field_ptr, value)?;
            }
        }

        Ok(())
    }
}
