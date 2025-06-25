use super::*;

impl<'ctx> Compiler<'ctx> {
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
            Statement::VariableDeclaration { name, type_, initializer } => {
                let llvm_type = self.to_llvm_type(type_)?;
                let alloca = match &llvm_type {
                    Type::Basic(basic_type) => self.builder.build_alloca(*basic_type, name)?,
                    Type::Function(_) => self.builder.build_alloca(self.context.ptr_type(AddressSpace::default()), name)?,
                    Type::Void => {
                        return Err(CompileError::InternalError(
                            "Cannot declare variable of type void".to_string(),
                            None,
                        ));
                    }
                    Type::Pointer(_) => {
                        return Err(CompileError::UnsupportedFeature(
                            "Cannot declare variable of pointer type directly".to_string(),
                            None,
                        ));
                    }
                    Type::Struct(_) => {
                        return Err(CompileError::UnsupportedFeature(
                            "Cannot declare variable of struct type directly".to_string(),
                            None,
                        ));
                    }
                };
                if let Some(expr) = initializer {
                    // Special case: initializing a function pointer with a function identifier
                    if matches!(type_, AstType::Function { .. }) {
                        if let Expression::Identifier(func_name) = expr {
                            if let Some(function) = self.module.get_function(func_name) {
                                let func_ptr = function.as_global_value().as_pointer_value();
                                self.builder.build_store(alloca, func_ptr)?;
                                self.variables.insert(name.clone(), (alloca, type_.clone()));
                                return Ok(());
                            }
                        }
                    } else if let AstType::Pointer(inner) = type_ {
                        if matches!(**inner, AstType::Function { .. }) {
                            if let Expression::Identifier(func_name) = expr {
                                if let Some(function) = self.module.get_function(func_name) {
                                    let func_ptr = function.as_global_value().as_pointer_value();
                                    self.builder.build_store(alloca, func_ptr)?;
                                    self.variables.insert(name.clone(), (alloca, type_.clone()));
                                    return Ok(());
                                }
                            }
                        }
                    }
                    let value = self.compile_expression(expr)?;
                    let value = match (value, type_) {
                        (BasicValueEnum::IntValue(int_val), AstType::Int32) => {
                            if int_val.get_type().get_bit_width() != 32 {
                                self.builder.build_int_truncate(int_val, self.context.i32_type(), "trunc").unwrap().into()
                            } else {
                                int_val.into()
                            }
                        }
                        (BasicValueEnum::IntValue(int_val), AstType::Int64) => {
                            if int_val.get_type().get_bit_width() != 64 {
                                self.builder.build_int_s_extend(int_val, self.context.i64_type(), "sext").unwrap().into()
                            } else {
                                int_val.into()
                            }
                        }
                        _ => value,
                    };
                    self.builder.build_store(alloca, value)?;
                }
                // Register the variable in the variables map
                self.variables.insert(name.clone(), (alloca, type_.clone()));
                Ok(())
            }
            Statement::VariableAssignment { name, value } => {
                // Look up the variable in the symbol table
                let (alloca, var_type) = self.get_variable(name)?;
                let value = self.compile_expression(value)?;
                let value = match (&value, &var_type) {
                    (BasicValueEnum::IntValue(int_val), AstType::Int32) => {
                        if int_val.get_type().get_bit_width() != 32 {
                            self.builder.build_int_truncate(*int_val, self.context.i32_type(), "trunc").unwrap().into()
                        } else {
                            (*int_val).into()
                        }
                    }
                    (BasicValueEnum::IntValue(int_val), AstType::Int64) => {
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
                    _ => {
                        let llvm_type = self.to_llvm_type(&var_type)?;
                        let _basic_type = self.expect_basic_type(llvm_type)?;
                        self.builder.build_store(alloca, value)?;
                    }
                }
                Ok(())
            }
            Statement::PointerAssignment { pointer, value } => {
                let ptr = self.compile_expression(&pointer)?;
                let val = self.compile_expression(&value)?;
                match ptr {
                    BasicValueEnum::PointerValue(ptr) => {
                        self.builder.build_store(ptr, val)?;
                        Ok(())
                    }
                    _ => Err(CompileError::TypeMismatch {
                        expected: "pointer".to_string(),
                        found: format!("{:?}", ptr.get_type()),
                        span: None,
                    }),
                }
            }
            Statement::Loop { condition, body } => {
                // Create the loop blocks
                let function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function".to_string(), None))?;
                let loop_cond = self.context.append_basic_block(function, "loop_cond");
                let loop_body = self.context.append_basic_block(function, "loop_body");
                let after_loop = self.context.append_basic_block(function, "after_loop");
                // Branch to the loop header
                self.builder.build_unconditional_branch(loop_cond)?;
                // Set up the loop header
                self.builder.position_at_end(loop_cond);
                let cond = self.compile_expression(condition)?;
                // Ensure condition is a boolean
                if !cond.is_int_value() || cond.into_int_value().get_type().get_bit_width() != 1 {
                    return Err(CompileError::InvalidLoopCondition(
                        "Loop condition must be a boolean (i1) value".to_string(),
                        None,
                    ));
                }
                self.builder.build_conditional_branch(cond.into_int_value(), loop_body, after_loop)?;
                // Compile the loop body
                self.builder.position_at_end(loop_body);
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                self.builder.build_unconditional_branch(loop_cond)?;
                // Set up the after loop block
                self.builder.position_at_end(after_loop);
                Ok(())
            }
        }
    }
} 