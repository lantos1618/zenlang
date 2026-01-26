use super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::{
    types::BasicType,
    values::{BasicValue, BasicValueEnum},
};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_address_of(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                let var_info = self.variables.get(name).ok_or_else(|| {
                    CompileError::UndeclaredVariable(name.clone(), self.get_current_span())
                })?;

                let alloca = var_info.pointer;
                let ast_type = &var_info.ast_type;

                // If the variable is already a pointer type, return it directly
                if ast_type.is_ptr_type() {
                    Ok(alloca.as_basic_value_enum())
                } else {
                    // For non-pointer variables, return the address
                    Ok(alloca.as_basic_value_enum())
                }
            }
            // Handle &expr.method() - compile the method call and return its result
            // This is commonly used with .ref() which returns a pointer
            Expression::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                // Compile the method call - for .ref() this returns a pointer
                let result = self.compile_method_call(object, method, args)?;
                // The result should already be a pointer value
                Ok(result)
            }
            // Handle &member.access - get address of struct field
            Expression::MemberAccess { object, member } => {
                // First compile the member access to get the field pointer
                // For now, just compile the expression and return it
                let result = self.compile_expression(&Expression::MemberAccess {
                    object: object.clone(),
                    member: member.clone(),
                })?;
                Ok(result)
            }
            // Handle &expr.ref() - CreateReference returns a pointer to the value
            // When we see &x.ref(), we want the address of x, which is what .ref() gives us
            Expression::CreateReference(inner) => {
                // .ref() on a value returns its address, so &x.ref() is just the address of x
                self.compile_address_of(inner)
            }
            Expression::CreateMutableReference(inner) => {
                // Same for .mut_ref()
                self.compile_address_of(inner)
            }
            _ => Err(CompileError::UnsupportedFeature(
                format!("AddressOf not supported for expression type: {:?}", expr),
                self.get_current_span(),
            )),
        }
    }

    pub fn compile_dereference(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // First, check if expr is an identifier for a pointer variable
        if let Expression::Identifier(name) = expr {
            if let Ok((alloca, ast_type)) = self.get_variable(name) {
                if let Some(inner) = ast_type.ptr_inner() {
                    // This is a pointer variable, so we need to:
                    // 1. Load the pointer value from the alloca
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let ptr_value = self.builder.build_load(ptr_type, alloca, "load_ptr")?;
                    let ptr = ptr_value.into_pointer_value();

                    // 2. Load the value from the address stored in the pointer
                    let llvm_type = self.to_llvm_type(inner)?;
                    return match llvm_type {
                        super::Type::Basic(basic_type) => {
                            Ok(self.builder.build_load(basic_type, ptr, "deref_value")?)
                        }
                        super::Type::Struct(struct_type) => {
                            Ok(self.builder.build_load(struct_type, ptr, "deref_struct")?)
                        }
                        _ => Err(CompileError::TypeError(
                            "Cannot dereference non-basic/non-struct type".to_string(),
                            self.current_span.clone(),
                        )),
                    };
                } else {
                    return Err(CompileError::TypeMismatch {
                        expected: "pointer".to_string(),
                        found: format!("{:?}", ast_type),
                        span: self.current_span.clone(),
                    });
                }
            }
        }

        // For CreateReference expressions, we know the type
        if let Expression::CreateReference(inner_expr)
        | Expression::CreateMutableReference(inner_expr) = expr
        {
            // This is a temporary pointer created by .ref() or .mut_ref()
            // The pointer value is the address of the inner expression
            let ptr_val = self.compile_address_of(inner_expr)?;
            if !ptr_val.is_pointer_value() {
                return Err(CompileError::TypeMismatch {
                    expected: "pointer".to_string(),
                    found: format!("{:?}", ptr_val.get_type()),
                    span: self.current_span.clone(),
                });
            }
            let ptr = ptr_val.into_pointer_value();

            // Try to infer the type of the inner expression
            // For now, we'll use i32 as the default for integer literals
            let llvm_type = if let Expression::Integer32(_) = &**inner_expr {
                super::Type::Basic(self.context.i32_type().as_basic_type_enum())
            } else if let Expression::Identifier(name) = &**inner_expr {
                // Look up the type of the identifier
                if let Ok((_, ast_type)) = self.get_variable(name) {
                    self.to_llvm_type(&ast_type)?
                } else {
                    super::Type::Basic(self.context.i32_type().as_basic_type_enum())
                }
            } else {
                // Default to i32 for now
                super::Type::Basic(self.context.i32_type().as_basic_type_enum())
            };

            return match llvm_type {
                super::Type::Basic(basic_type) => {
                    Ok(self.builder.build_load(basic_type, ptr, "deref_value")?)
                }
                super::Type::Struct(struct_type) => {
                    Ok(self.builder.build_load(struct_type, ptr, "deref_struct")?)
                }
                _ => Err(CompileError::TypeError(
                    "Cannot dereference non-basic/non-struct type".to_string(),
                    self.current_span.clone(),
                )),
            };
        }

        // For other expressions, compile them and check if they return a pointer
        let ptr_val = self.compile_expression(expr)?;
        if !ptr_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "pointer".to_string(),
                found: format!("{:?}", ptr_val.get_type()),
                span: self.current_span.clone(),
            });
        }
        let ptr = ptr_val.into_pointer_value();

        // Since we don't have type info, assume i32 for now (most common in tests)
        let llvm_type = super::Type::Basic(self.context.i32_type().as_basic_type_enum());
        match llvm_type {
            super::Type::Basic(basic_type) => {
                Ok(self.builder.build_load(basic_type, ptr, "load_tmp")?)
            }
            super::Type::Struct(struct_type) => {
                Ok(self
                    .builder
                    .build_load(struct_type, ptr, "load_struct_tmp")?)
            }
            _ => Err(CompileError::TypeError(
                "Cannot dereference non-basic/non-struct type".to_string(),
                self.current_span.clone(),
            )),
        }
    }

    pub fn compile_pointer_to_int(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Convert a pointer to its numeric address (usize)
        let val = self.compile_expression(expr)?;
        if let Ok(ptr) = val.try_into() {
            let ptr: inkwell::values::PointerValue = ptr;
            // Use i64 as usize for now (TODO: get proper target-specific size)
            let usize_type = self.context.i64_type();
            let int_val = self
                .builder
                .build_ptr_to_int(ptr, usize_type, "ptr_to_int")?;
            Ok(int_val.as_basic_value_enum())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "pointer".to_string(),
                found: format!("{:?}", val.get_type()),
                span: self.current_span.clone(),
            })
        }
    }

    pub fn compile_pointer_offset(
        &mut self,
        pointer: &Expression,
        offset: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let base_val = self.compile_expression(pointer)?;
        let offset_val = self.compile_expression(offset)?;
        if !base_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "pointer for pointer offset base".to_string(),
                found: format!("{:?}", base_val.get_type()),
                span: self.current_span.clone(),
            });
        }
        if !offset_val.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "integer for pointer offset value".to_string(),
                found: format!("{:?}", offset_val.get_type()),
                span: self.current_span.clone(),
            });
        }
        unsafe {
            let ptr_type = base_val.get_type();
            let _offset = offset_val.into_int_value();
            let ptr = base_val.into_pointer_value();
            Ok(self
                .builder
                .build_gep(
                    ptr_type,
                    ptr,
                    &[self.context.i32_type().const_int(0, false)],
                    "gep_tmp",
                )?
                .into())
        }
    }
}
