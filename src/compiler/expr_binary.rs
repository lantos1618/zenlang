use super::*;

impl<'ctx> Compiler<'ctx> {
    pub fn compile_binary_operation(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;

        match op {
            BinaryOperator::Add => self.compile_add(left_val, right_val),
            BinaryOperator::Subtract => self.compile_subtract(left_val, right_val),
            BinaryOperator::Multiply => self.compile_multiply(left_val, right_val),
            BinaryOperator::Divide => self.compile_divide(left_val, right_val),
            BinaryOperator::Equals => self.compile_equals(left_val, right_val),
            BinaryOperator::NotEquals => self.compile_not_equals(left_val, right_val),
            BinaryOperator::LessThan => self.compile_less_than(left_val, right_val),
            BinaryOperator::GreaterThan => self.compile_greater_than(left_val, right_val),
            BinaryOperator::LessThanEquals => self.compile_less_than_equals(left_val, right_val),
            BinaryOperator::GreaterThanEquals => self.compile_greater_than_equals(left_val, right_val),
            BinaryOperator::StringConcat => self.compile_string_concat(left_val, right_val),
        }
    }

    fn compile_add(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if left_val.is_int_value() && right_val.is_int_value() {
            let result = self.builder.build_int_add(
                left_val.into_int_value(),
                right_val.into_int_value(),
                "addtmp"
            )?;
            Ok(result.into())
        } else if left_val.is_float_value() && right_val.is_float_value() {
            let result = self.builder.build_float_add(
                left_val.into_float_value(),
                right_val.into_float_value(),
                "addtmp"
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

    fn compile_subtract(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_multiply(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_divide(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_not_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_less_than(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_greater_than(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_less_than_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_greater_than_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_string_concat(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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