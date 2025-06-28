use super::core::Compiler;
use crate::ast::BinaryOperator;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;
use inkwell::{IntPredicate, FloatPredicate};
use inkwell::AddressSpace;

impl<'ctx> Compiler<'ctx> {
    pub fn compile_binary_operation(
        &mut self,
        op: &BinaryOperator,
        left: &ast::Expression,
        right: &ast::Expression,
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
            BinaryOperator::Modulo => self.compile_modulo(left_val, right_val),
            BinaryOperator::And => self.compile_and(left_val, right_val),
            BinaryOperator::Or => self.compile_or(left_val, right_val),
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
        // For now, just return a concatenated string pointer
        // TODO: Implement proper string concatenation
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let result = i8_ptr_type.const_null();
        Ok(result.into())
    }

    fn compile_modulo(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if left_val.is_int_value() && right_val.is_int_value() {
            let result = self.builder.build_int_signed_rem(
                left_val.into_int_value(),
                right_val.into_int_value(),
                "modtmp"
            )?;
            Ok(result.into())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: None,
            })
        }
    }

    fn compile_and(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if left_val.is_int_value() && right_val.is_int_value() {
            let result = self.builder.build_and(
                left_val.into_int_value(),
                right_val.into_int_value(),
                "andtmp"
            )?;
            Ok(result.into())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: None,
            })
        }
    }

    fn compile_or(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if left_val.is_int_value() && right_val.is_int_value() {
            let result = self.builder.build_or(
                left_val.into_int_value(),
                right_val.into_int_value(),
                "ortmp"
            )?;
            Ok(result.into())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: None,
            })
        }
    }
} 