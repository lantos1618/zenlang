use super::{LLVMCompiler, Type};
use crate::ast::{AstType, Expression, Statement};
use crate::error::CompileError;
use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::{BasicValueEnum, BasicValue},
};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                Ok(())
            }
            Statement::Return(expr) => {
                let value = self.compile_expression(expr)?;
                self.builder.build_return(Some(&value))?;
                Ok(())
            }
            Statement::VariableDeclaration { name, type_, initializer, is_mutable: _, declaration_type: _ } => {
                // Handle type inference or explicit type
                let llvm_type = match type_ {
                    Some(type_) => self.to_llvm_type(type_)?,
                    None => {
                        // Type inference - try to infer from initializer
                        if let Some(init_expr) = initializer {
                            let init_value = self.compile_expression(init_expr)?;
                            match init_value {
                                BasicValueEnum::IntValue(int_val) => {
                                    if int_val.get_type().get_bit_width() <= 32 {
                                        Type::Basic(self.context.i32_type().into())
                                    } else {
                                        Type::Basic(self.context.i64_type().into())
                                    }
                                }
                                BasicValueEnum::FloatValue(_) => {
                                    // For now, assume all floats are f64
                                    Type::Basic(self.context.f64_type().into())
                                }
                                BasicValueEnum::PointerValue(_) => {
                                    Type::Basic(self.context.i64_type().into()) // Assume pointer-sized
                                }
                                _ => Type::Basic(self.context.i64_type().into()), // Default to i64
                            }
                        } else {
                            return Err(CompileError::TypeError("Cannot infer type without initializer".to_string(), None));
                        }
                    }
                };

                // Extract BasicTypeEnum for build_alloca
                let basic_type = match llvm_type {
                    Type::Basic(basic) => basic,
                    Type::Struct(struct_type) => struct_type.as_basic_type_enum(),
                    Type::Function(fn_type) => fn_type.ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum(),
                    _ => return Err(CompileError::TypeError("Cannot allocate non-basic or struct type".to_string(), None)),
                };

                let alloca = self.builder.build_alloca(basic_type, name).map_err(|e| CompileError::from(e))?;

                if let Some(init_expr) = initializer {
                    let value = self.compile_expression(init_expr)?;
                    
                    // Handle function pointers specially
                    if let Some(type_) = type_ {
                        if matches!(type_, AstType::Function { .. }) {
                            // For function pointers, we need to get the function and store its pointer
                            if let Expression::Identifier(func_name) = init_expr {
                                if let Some(function) = self.module.get_function(&func_name) {
                                    // Store the function pointer
                                    let func_ptr = function.as_global_value().as_pointer_value();
                                    self.builder.build_store(alloca, func_ptr).map_err(|e| CompileError::from(e))?;
                                    self.variables.insert(name.clone(), (alloca, type_.clone()));
                                    Ok(())
                                } else {
                                    Err(CompileError::UndeclaredFunction(func_name.clone(), None))
                                }
                            } else {
                                Err(CompileError::TypeError("Function pointer initializer must be a function name".to_string(), None))
                            }
                        } else if let AstType::Pointer(inner) = type_ {
                            // For pointers, if the initializer is AddressOf, use the pointer inside the alloca
                            let ptr_value = match init_expr {
                                Expression::AddressOf(inner_expr) => {
                                    // Compile the inner expression to get the alloca pointer
                                    match **inner_expr {
                                        Expression::Identifier(ref id) => {
                                            let (inner_alloca, _) = self.variables.get(id).ok_or_else(|| CompileError::UndeclaredVariable(id.clone(), None))?;
                                            inner_alloca.as_basic_value_enum()
                                        }
                                        _ => value.clone(),
                                    }
                                }
                                _ => value.clone(),
                            };
                            self.builder.build_store(alloca, ptr_value).map_err(|e| CompileError::from(e))?;
                            self.variables.insert(name.clone(), (alloca, type_.clone()));
                            Ok(())
                        } else {
                            // Regular value assignment
                            let value = match (value, type_) {
                                (BasicValueEnum::IntValue(int_val), AstType::I32) => {
                                    self.builder.build_int_truncate(int_val, self.context.i32_type(), "trunc").map_err(|e| CompileError::from(e))?.into()
                                }
                                (BasicValueEnum::IntValue(int_val), AstType::I64) => {
                                    self.builder.build_int_s_extend(int_val, self.context.i64_type(), "extend").map_err(|e| CompileError::from(e))?.into()
                                }
                                (BasicValueEnum::FloatValue(float_val), AstType::F32) => {
                                    self.builder.build_float_trunc(float_val, self.context.f32_type(), "trunc").map_err(|e| CompileError::from(e))?.into()
                                }
                                (BasicValueEnum::FloatValue(float_val), AstType::F64) => {
                                    self.builder.build_float_ext(float_val, self.context.f64_type(), "extend").map_err(|e| CompileError::from(e))?.into()
                                }
                                (BasicValueEnum::PointerValue(ptr_val), AstType::Struct { .. }) => {
                                    // If the value is a pointer and the type is a struct, load the struct value
                                    let struct_type = match self.to_llvm_type(type_)? {
                                        Type::Struct(st) => st,
                                        _ => return Err(CompileError::TypeError("Expected struct type".to_string(), None)),
                                    };
                                    self.builder.build_load(struct_type, ptr_val, "load_struct_init")?.into()
                                }
                                _ => value,
                            };
                            self.builder.build_store(alloca, value).map_err(|e| CompileError::from(e))?;
                            self.variables.insert(name.clone(), (alloca, type_.clone()));
                            Ok(())
                        }
                    } else {
                        // Type inference case
                        self.builder.build_store(alloca, value).map_err(|e| CompileError::from(e))?;
                        // For inferred types, we need to determine the type from the value
                        let inferred_type = match value {
                            BasicValueEnum::IntValue(int_val) => {
                                if int_val.get_type().get_bit_width() <= 32 {
                                    AstType::I32
                                } else {
                                    AstType::I64
                                }
                            }
                            BasicValueEnum::FloatValue(_) => {
                                // For now, assume all floats are f64
                                AstType::F64
                            }
                            BasicValueEnum::PointerValue(_) => AstType::Pointer(Box::new(AstType::I64)), // Assume pointer to i64
                            _ => AstType::I64, // Default
                        };
                        self.variables.insert(name.clone(), (alloca, inferred_type));
                        Ok(())
                    }
                } else {
                    // No initializer - initialize to zero/default
                    let zero: BasicValueEnum = match llvm_type {
                        Type::Basic(BasicTypeEnum::IntType(int_type)) => {
                            int_type.const_zero().into()
                        }
                        Type::Basic(BasicTypeEnum::FloatType(float_type)) => {
                            float_type.const_zero().into()
                        }
                        Type::Basic(BasicTypeEnum::PointerType(_)) => {
                            self.context.i64_type().const_zero().into()
                        }
                        _ => self.context.i64_type().const_zero().into(),
                    };
                    self.builder.build_store(alloca, zero).map_err(|e| CompileError::from(e))?;
                    
                    if let Some(type_) = type_ {
                        self.variables.insert(name.clone(), (alloca, type_.clone()));
                        Ok(())
                    } else {
                        // For inferred types without initializer, default to i64
                        self.variables.insert(name.clone(), (alloca, AstType::I64));
                        Ok(())
                    }
                }
            }
            Statement::VariableAssignment { name, value } => {
                // Check if this is a field assignment (e.g., "s.x")
                if let Some(dot_pos) = name.find('.') {
                    let struct_name = &name[..dot_pos];
                    let field_name = &name[dot_pos + 1..];
                    // Get the struct variable
                    let (struct_alloca, struct_type) = self.get_variable(struct_name)?;
                    // Compile the value to assign
                    let value = self.compile_expression(value)?;
                    // Handle type conversion if needed
                    let value = match (&value, &struct_type) {
                        (BasicValueEnum::IntValue(int_val), AstType::Struct { .. }) => {
                            if int_val.get_type().get_bit_width() != 64 {
                                self.builder.build_int_s_extend(*int_val, self.context.i64_type(), "sext").unwrap().into()
                            } else {
                                (*int_val).into()
                            }
                        }
                        _ => value.clone(),
                    };
                    // Use struct field assignment
                    self.compile_struct_field_assignment(struct_alloca, field_name, value)?;
                    return Ok(());
                }
                // Regular variable assignment
                let (alloca, var_type) = self.get_variable(name)?;
                let value = self.compile_expression(value)?;
                let value = match (&value, &var_type) {
                    (BasicValueEnum::IntValue(int_val), AstType::I32) => {
                        if int_val.get_type().get_bit_width() != 32 {
                            self.builder.build_int_truncate(*int_val, self.context.i32_type(), "trunc").unwrap().into()
                        } else {
                            (*int_val).into()
                        }
                    }
                    (BasicValueEnum::IntValue(int_val), AstType::I64) => {
                        if int_val.get_type().get_bit_width() != 64 {
                            self.builder.build_int_s_extend(*int_val, self.context.i64_type(), "sext").unwrap().into()
                        } else {
                            (*int_val).into()
                        }
                    }
                    _ => value.clone(),
                };
                match &var_type {
                    AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                        // Store function pointer directly
                        self.builder.build_store(alloca, value)?;
                    }
                    AstType::Struct { .. } => {
                        // For struct types, we can store directly without checking basic type
                        self.builder.build_store(alloca, value)?;
                    }
                    _ => {
                        let llvm_type = self.to_llvm_type(&var_type)?;
                        let _basic_type = self.expect_basic_type(llvm_type)?;
                        self.builder.build_store(alloca, value)?;
                    }
                }
                Ok(())
            }
            Statement::PointerAssignment { pointer, value } => {
                let ptr_val = self.compile_expression(&pointer)?;
                let val = self.compile_expression(&value)?;
                
                // For pointer variables, we need to load the address first, then store to that address
                if ptr_val.is_pointer_value() {
                    let ptr = ptr_val.into_pointer_value();
                    // Load the address stored in the pointer variable
                    let address = self.builder.build_load(self.context.i64_type().ptr_type(inkwell::AddressSpace::default()), ptr, "deref_ptr")?;
                    // Store the value at that address
                    let address_ptr = address.into_pointer_value();
                    self.builder.build_store(address_ptr, val)?;
                    Ok(())
                } else {
                    Err(CompileError::TypeMismatch {
                        expected: "pointer".to_string(),
                        found: format!("{:?}", ptr_val.get_type()),
                        span: None,
                    })
                }
            }
            Statement::Loop { condition, body, label: _ } => {
                // Create the loop structure
                let loop_block = self.context.append_basic_block(self.current_function.unwrap(), "loop");
                let after_loop_block = self.context.append_basic_block(self.current_function.unwrap(), "after_loop");
                
                // Branch to loop block
                self.builder.build_unconditional_branch(loop_block).map_err(|e| CompileError::from(e))?;
                
                // Emit loop block
                self.builder.position_at_end(loop_block);
                
                // Compile loop body
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                // If there's a condition, check it
                if let Some(cond_expr) = condition {
                    let cond_value = self.compile_expression(cond_expr)?;
                    if let BasicValueEnum::IntValue(int_val) = cond_value {
                        self.builder.build_conditional_branch(int_val, loop_block, after_loop_block).map_err(|e| CompileError::from(e))?;
                    } else {
                        return Err(CompileError::TypeError("Loop condition must be an integer".to_string(), None));
                    }
                } else {
                    // Infinite loop - just branch back
                    self.builder.build_unconditional_branch(loop_block).map_err(|e| CompileError::from(e))?;
                }
                
                // Position builder at after_loop block
                self.builder.position_at_end(after_loop_block);
                Ok(())
            },
            Statement::Break { label: _ } => {
                // For now, just return from the function
                // TODO: Implement proper break with labels
                let return_value = self.context.i64_type().const_zero();
                self.builder.build_return(Some(&return_value)).map_err(|e| CompileError::from(e))?;
                Ok(())
            },
            Statement::Continue { label: _ } => {
                // For now, just skip to next iteration
                // TODO: Implement proper continue with labels
                let return_value = self.context.i64_type().const_zero();
                self.builder.build_return(Some(&return_value)).map_err(|e| CompileError::from(e))?;
                Ok(())
            },
            Statement::ComptimeBlock(_) => {
                // For now, ignore comptime blocks during codegen
                Ok(())
            },
            Statement::ModuleImport { .. } => {
                // Module imports are handled during parsing, not codegen
                Ok(())
            },
        }
    }
} 