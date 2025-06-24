use super::*;

// All code related to expression codegen will be moved here.
// ... existing code from mod.rs for compile_expression and helpers ... 

impl<'ctx> Compiler<'ctx> {
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Integer8(n) => Ok(self.context.i8_type().const_int(*n as u64, false).into()),
            Expression::Integer16(n) => Ok(self.context.i16_type().const_int(*n as u64, false).into()),
            Expression::Integer32(n) => Ok(self.context.i32_type().const_int(*n as u64, false).into()),
            Expression::Integer64(n) => Ok(self.context.i64_type().const_int(*n as u64, false).into()),
            Expression::Float(n) => Ok(self.context.f64_type().const_float(*n).into()),
            Expression::String(val) => {
                let ptr = self.builder.build_global_string_ptr(val, "str")?;
                let ptr_val = ptr.as_pointer_value();
                if let Some(func) = self.current_function {
                    if let Some(return_type) = func.get_type().get_return_type() {
                        if return_type.is_int_type() {
                            let ptr_int = self.builder.build_ptr_to_int(
                                ptr_val,
                                return_type.into_int_type(),
                                "str_to_int"
                            )?;
                            return Ok(ptr_int.into());
                        }
                    }
                }
                Ok(ptr_val.into())
            },
            Expression::Identifier(name) => {
                // First check if this is a function name
                if let Some(function) = self.module.get_function(name) {
                    // Return the function's address as a pointer value
                    Ok(function.as_global_value().as_pointer_value().into())
                } else {
                    // It's a variable, load it normally
                    let (ptr, ast_type) = self.get_variable(name)?;
                    let loaded: BasicValueEnum = match &ast_type {
                        AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                            match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
                                Ok(val) => val.into(),
                                Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                            }
                        }
                        AstType::Function { .. } => {
                            match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
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
            },
            Expression::BinaryOp { op, left, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;

                match op {
                    BinaryOperator::Add => {
                        println!("Left value type: {:?}", left_val.get_type());
                        println!("Right value type: {:?}", right_val.get_type());
                        println!("Left is int: {}, right is int: {}", 
                               left_val.is_int_value(), 
                               right_val.is_int_value());
                        println!("Left is float: {}, right is float: {}", 
                               left_val.is_float_value(), 
                               right_val.is_float_value());

                        if left_val.is_int_value() && right_val.is_int_value() {
                            println!("Performing integer addition");
                            let result = self.builder.build_int_add(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            println!("Performing float addition");
                            let result = self.builder.build_float_add(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            println!("Type mismatch in addition");
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Subtract => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_sub(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "subtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_sub(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "subtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Multiply => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_mul(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "multmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_mul(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "multmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Divide => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_signed_div(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "divtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_div(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "divtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Equals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::EQ,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "eqtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OEQ,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "eqtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_pointer_value() && right_val.is_pointer_value() {
                            // String comparison: call strcmp and check for zero
                            let strcmp_fn = match self.module.get_function("strcmp") {
                                Some(f) => f,
                                None => {
                                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                    let fn_type = self.context.i32_type().fn_type(
                                        &[i8_ptr_type.into(), i8_ptr_type.into()],
                                        false
                                    );
                                    self.module.add_function("strcmp", fn_type, None)
                                }
                            };
                            let left_ptr = left_val.into_pointer_value();
                            let right_ptr = right_val.into_pointer_value();
                            let call = self.builder.build_call(
                                strcmp_fn,
                                &[left_ptr.into(), right_ptr.into()],
                                "strcmp_call"
                            )?;
                            let cmp_result = call.try_as_basic_value().left().ok_or_else(||
                                CompileError::InternalError("strcmp did not return a value".to_string(), None)
                            )?.into_int_value();
                            let zero = self.context.i32_type().const_int(0, false);
                            let result = self.builder.build_int_compare(
                                IntPredicate::EQ,
                                cmp_result,
                                zero,
                                "strcmp_eq"
                            )?;
                            // Zero-extend i1 to i64 for test compatibility
                            let zext = self.builder.build_int_z_extend(result, self.context.i64_type(), "zext_strcmp_eq")?;
                            Ok(zext.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float or string".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::NotEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::NE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "netmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::ONE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "netmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_pointer_value() && right_val.is_pointer_value() {
                            // String comparison: call strcmp and check for nonzero
                            let strcmp_fn = match self.module.get_function("strcmp") {
                                Some(f) => f,
                                None => {
                                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                    let fn_type = self.context.i32_type().fn_type(
                                        &[i8_ptr_type.into(), i8_ptr_type.into()],
                                        false
                                    );
                                    self.module.add_function("strcmp", fn_type, None)
                                }
                            };
                            let left_ptr = left_val.into_pointer_value();
                            let right_ptr = right_val.into_pointer_value();
                            let call = self.builder.build_call(
                                strcmp_fn,
                                &[left_ptr.into(), right_ptr.into()],
                                "strcmp_call"
                            )?;
                            let cmp_result = call.try_as_basic_value().left().ok_or_else(||
                                CompileError::InternalError("strcmp did not return a value".to_string(), None)
                            )?.into_int_value();
                            let zero = self.context.i32_type().const_int(0, false);
                            let result = self.builder.build_int_compare(
                                IntPredicate::NE,
                                cmp_result,
                                zero,
                                "strcmp_ne"
                            )?;
                            // Zero-extend i1 to i64 for test compatibility
                            let zext = self.builder.build_int_z_extend(result, self.context.i64_type(), "zext_strcmp_ne")?;
                            Ok(zext.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float or string".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::LessThan => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SLT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "lttmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OLT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "lttmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::GreaterThan => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SGT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "gttmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OGT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "gttmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::LessThanEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SLE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "letmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OLE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "letmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::GreaterThanEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SGE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "getmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OGE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "getmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::StringConcat => {
                        // Ensure both operands are string pointers (i8* in LLVM)
                        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                        
                        let left_ptr = if left_val.is_pointer_value() {
                            left_val.into_pointer_value()
                        } else if left_val.is_int_value() {
                            // Convert from integer to pointer
                            let int_val = left_val.into_int_value();
                            self.builder.build_int_to_ptr(
                                int_val,
                                i8_ptr_type,
                                "str_ptr"
                            )?
                        } else {
                            return Err(CompileError::TypeMismatch {
                                expected: "string or string pointer".to_string(),
                                found: left_val.get_type().to_string(),
                                span: None,
                            });
                        };
                        
                        let right_ptr = if right_val.is_pointer_value() {
                            right_val.into_pointer_value()
                        } else if right_val.is_int_value() {
                            // Convert from integer to pointer
                            let int_val = right_val.into_int_value();
                            self.builder.build_int_to_ptr(
                                int_val,
                                i8_ptr_type,
                                "str_ptr"
                            )?
                        } else {
                            return Err(CompileError::TypeMismatch {
                                expected: "string or string pointer".to_string(),
                                found: right_val.get_type().to_string(),
                                span: None,
                            });
                        };
                        
                        // Declare the strcat function if it doesn't exist
                        let strcat_fn = match self.module.get_function("strcat") {
                            Some(f) => f,
                            None => {
                                // Declare strcat: i8* @strcat(i8*, i8*)
                                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                let fn_type = self.context.i8_type().fn_type(
                                    &[i8_ptr_type.into(), i8_ptr_type.into()], 
                                    false
                                );
                                self.module.add_function("strcat", fn_type, None)
                            }
                        };
                        
                        // Declare malloc if it doesn't exist
                        let malloc_fn = match self.module.get_function("malloc") {
                            Some(f) => f,
                            None => {
                                // Declare malloc: i8* @malloc(i64)
                                let i64_type = self.context.i64_type();
                                let fn_type = self.context.ptr_type(AddressSpace::default())
                                    .fn_type(&[i64_type.into()], false);
                                self.module.add_function("malloc", fn_type, None)
                            }
                        };
                        
                        // Declare strlen if it doesn't exist
                        let strlen_fn = match self.module.get_function("strlen") {
                            Some(f) => f,
                            None => {
                                // Declare strlen: i64 @strlen(i8*)
                                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                let fn_type = self.context.i64_type().fn_type(
                                    &[i8_ptr_type.into()], 
                                    false
                                );
                                self.module.add_function("strlen", fn_type, None)
                            }
                        };
                        
                        // Get lengths of both strings
                        let left_len = {
                            let call = self.builder.build_call(
                                strlen_fn, 
                                &[left_ptr.into()], 
                                "left_len"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("strlen did not return a value".to_string(), None)
                            )?.into_int_value()
                        };
                        
                        let right_len = {
                            let call = self.builder.build_call(
                                strlen_fn, 
                                &[right_ptr.into()], 
                                "right_len"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("strlen did not return a value".to_string(), None)
                            )?.into_int_value()
                        };
                        
                        // Calculate total length needed (left + right + 1 for null terminator)
                        let total_len = self.builder.build_int_add(
                            left_len,
                            right_len,
                            "total_len"
                        )?;
                        
                        let one = self.context.i64_type().const_int(1, false);
                        let total_len = self.builder.build_int_add(total_len, one, "total_len_with_null")?;
                        
                        // Allocate memory for the new string
                        let new_str_ptr = {
                            let call = self.builder.build_call(
                                malloc_fn,
                                &[total_len.into()],
                                "new_str"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("malloc did not return a value".to_string(), None)
                            )?.into_pointer_value()
                        };
                        
                        // Cast the result to i8*
                        let new_str_ptr = self.builder.build_pointer_cast(
                            new_str_ptr,
                            self.context.ptr_type(AddressSpace::default()),
                            "new_str_ptr"
                        )?;
                        
                        // Copy first string
                        self.builder.build_store(new_str_ptr, self.context.i8_type().const_int(0, false))?;
                        let _ = self.builder.build_call(
                            strcat_fn,
                            &[new_str_ptr.into(), left_ptr.into()],
                            "concat1"
                        )?;
                        
                        // Concatenate second string
                        let _ = self.builder.build_call(
                            strcat_fn,
                            &[new_str_ptr.into(), right_ptr.into()],
                            "concat2"
                        )?;
                        
                        Ok(new_str_ptr.into())
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                // First check if this is a direct function call
                if let Some(function) = self.module.get_function(name) {
                    // Direct function call
                    let mut compiled_args = Vec::with_capacity(args.len());
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        compiled_args.push(val);
                    }
                    let args_metadata: Vec<BasicMetadataValueEnum> = compiled_args.iter()
                        .map(|arg| BasicMetadataValueEnum::try_from(*arg).map_err(|_| 
                            CompileError::InternalError("Failed to convert argument to metadata".to_string(), None)
                        ))
                        .collect::<Result<Vec<_>, _>>()?;
                    let call = self.builder.build_call(function, &args_metadata, "calltmp")?;
                    Ok(call.try_as_basic_value().left().ok_or_else(||
                        CompileError::InternalError("Function call did not return a value".to_string(), None)
                    )?)
                } else if let Ok((alloca, var_type)) = self.get_variable(name) {
                    // Function pointer call - load the function pointer from variable
                    let function_ptr = self.builder.build_load(
                        alloca.get_type(),
                        alloca,
                        "func_ptr"
                    )?;
                    
                    // Get function type from the variable type
                    let function_type = match &var_type {
                        AstType::Function { args, return_type } => {
                            let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                                .iter()
                                .map(|ty| {
                                    let llvm_ty = self.to_llvm_type(ty)?;
                                    match llvm_ty {
                                        Type::Basic(b) => Ok(b),
                                        _ => Err(CompileError::InternalError("Function argument type must be a basic type".to_string(), None)),
                                    }
                                })
                                .collect();
                            let param_types = param_types?;
                            let param_metadata: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|ty| (*ty).into()).collect();
                            let ret_type = self.to_llvm_type(return_type)?;
                            match ret_type {
                                Type::Basic(b) => b.fn_type(&param_metadata, false),
                                Type::Void => self.context.void_type().fn_type(&param_metadata, false),
                                _ => return Err(CompileError::InternalError("Function return type must be a basic type or void".to_string(), None)),
                            }
                        },
                        AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                            let inner_llvm_type = self.to_llvm_type(inner)?;
                            match inner_llvm_type {
                                Type::Basic(BasicTypeEnum::PointerType(_ptr_type)) => {
                                    // For function pointers, we need to get the function type
                                    // Since we can't get it directly from the pointer type in newer LLVM,
                                    // we'll create a function type based on the AST type
                                    if let AstType::Function { args, return_type } = &**inner {
                                        let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                                            .iter()
                                            .map(|ty| {
                                                let llvm_ty = self.to_llvm_type(ty)?;
                                                match llvm_ty {
                                                    Type::Basic(b) => Ok(b),
                                                    _ => Err(CompileError::InternalError("Function argument type must be a basic type".to_string(), None)),
                                                }
                                            })
                                            .collect();
                                        let param_types = param_types?;
                                        let param_metadata: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|ty| (*ty).into()).collect();
                                        let ret_type = self.to_llvm_type(return_type)?;
                                        match ret_type {
                                            Type::Basic(b) => b.fn_type(&param_metadata, false),
                                            Type::Void => self.context.void_type().fn_type(&param_metadata, false),
                                            _ => return Err(CompileError::InternalError("Function return type must be a basic type or void".to_string(), None)),
                                        }
                                    } else {
                                        return Err(CompileError::InternalError("Expected function type in pointer".to_string(), None));
                                    }
                                },
                                _ => return Err(CompileError::TypeMismatch {
                                    expected: "function pointer".to_string(),
                                    found: format!("{:?}", inner_llvm_type),
                                    span: None,
                                }),
                            }
                        },
                        _ => return Err(CompileError::TypeMismatch {
                            expected: "function pointer".to_string(),
                            found: format!("{:?}", var_type),
                            span: None,
                        }),
                    };
                    
                    // Compile arguments
                    let mut compiled_args = Vec::with_capacity(args.len());
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        compiled_args.push(val);
                    }
                    let args_metadata: Vec<BasicMetadataValueEnum> = compiled_args.iter()
                        .map(|arg| BasicMetadataValueEnum::try_from(*arg).map_err(|_| 
                            CompileError::InternalError("Failed to convert argument to metadata".to_string(), None)
                        ))
                        .collect::<Result<Vec<_>, _>>()?;
                    
                    // Cast the loaded pointer to the correct function type
                    let casted_function_ptr = self.builder.build_pointer_cast(
                        function_ptr.into_pointer_value(),
                        self.context.ptr_type(AddressSpace::default()),
                        "casted_func_ptr"
                    )?;
                    
                    // Make indirect call using build_indirect_call for function pointers
                    let call = self.builder.build_indirect_call(
                        function_type,
                        casted_function_ptr,
                        &args_metadata,
                        "indirect_call"
                    )?;
                    Ok(call.try_as_basic_value().left().ok_or_else(||
                        CompileError::InternalError("Function call did not return a value".to_string(), None)
                    )?)
                } else {
                    // Function not found
                    Err(CompileError::UndeclaredFunction(name.clone(), None))
                }
            }
            Expression::Conditional { scrutinee, arms } => {
                let parent_function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;
                let cond_val = self.compile_expression(scrutinee)?;
                if !cond_val.is_int_value() || cond_val.into_int_value().get_type().get_bit_width() != 1 {
                    return Err(CompileError::TypeMismatch {
                        expected: "boolean (i1) for conditional expression".to_string(),
                        found: format!("{:?}", cond_val.get_type()),
                        span: None,
                    });
                }
                let then_bb = self.context.append_basic_block(parent_function, "then");
                let else_bb = self.context.append_basic_block(parent_function, "else");
                let merge_bb = self.context.append_basic_block(parent_function, "ifcont");
                self.builder.build_conditional_branch(cond_val.into_int_value(), then_bb, else_bb)?;
                // Emit 'then' block
                self.builder.position_at_end(then_bb);
                let then_val = self.compile_expression(&arms[0].1)?;
                self.builder.build_unconditional_branch(merge_bb)?;
                let then_bb_end = self.builder.get_insert_block().unwrap();
                // Emit 'else' block
                self.builder.position_at_end(else_bb);
                let else_val = self.compile_expression(&arms[1].1)?;
                self.builder.build_unconditional_branch(merge_bb)?;
                let else_bb_end = self.builder.get_insert_block().unwrap();
                // Emit 'merge' block
                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(
                    then_val.get_type(),
                    "iftmp"
                )?;
                phi.add_incoming(&[(&then_val, then_bb_end), (&else_val, else_bb_end)]);
                Ok(phi.as_basic_value())
            }
            Expression::AddressOf(expr) => {
                match &**expr {
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
            Expression::Dereference(expr) => {
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
            Expression::PointerOffset { pointer, offset } => {
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
            Expression::StructLiteral { name, fields } => {
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
            Expression::StructField { struct_, field } => {
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
                                let struct_ptr_type = self.context.ptr_type(AddressSpace::default());
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
            },
            Expression::StringLength(expr) => {
                let str_val = self.compile_expression(expr)?;
                if !str_val.is_pointer_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "string (i8*) for length operation".to_string(),
                        found: format!("{:?}", str_val.get_type()),
                        span: None,
                    });
                }
                let str_ptr = str_val.into_pointer_value();
                let strlen_fn = match self.module.get_function("strlen") {
                    Some(f) => f,
                    None => {
                        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                        let fn_type = self.context.i64_type().fn_type(
                            &[i8_ptr_type.into()], 
                            false
                        );
                        self.module.add_function("strlen", fn_type, None)
                    }
                };
                let len = {
                    let call = self.builder.build_call(
                        strlen_fn,
                        &[str_ptr.into()],
                        "strlen"
                    )?;
                    match call.try_as_basic_value() {
                        Either::Left(bv) => {
                            let int_val = bv.into_int_value();
                            let target_int_type = self.context.i64_type();
                            if int_val.get_type().get_bit_width() != target_int_type.get_bit_width() {
                                self.builder.build_int_z_extend(int_val, target_int_type, "strlen_ext")?.into()
                            } else {
                                int_val.into()
                            }
                        },
                        Either::Right(_) => return Err(CompileError::InternalError(
                            "strlen did not return a basic value".to_string(),
                            None,
                        )),
                    }
                };
                Ok(len)
            },
        }
    }
} 