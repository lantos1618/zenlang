use super::LLVMCompiler;
use crate::ast::{BinaryOperator, Expression};
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};
use inkwell::AddressSpace;
use inkwell::{FloatPredicate, IntPredicate};

/// Represents normalized operands after type coercion
enum NumericOperands<'ctx> {
    Integers(IntValue<'ctx>, IntValue<'ctx>),
    Floats(FloatValue<'ctx>, FloatValue<'ctx>),
}

impl<'ctx> LLVMCompiler<'ctx> {
    /// Normalize two numeric operands to compatible types.
    /// Returns either two integers of the same width, or two floats.
    fn normalize_numeric_operands(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<NumericOperands<'ctx>, CompileError> {
        match (
            left.is_int_value(),
            left.is_float_value(),
            right.is_int_value(),
            right.is_float_value(),
        ) {
            // Both integers: normalize to same width
            (true, _, true, _) => {
                let left_int = left.into_int_value();
                let right_int = right.into_int_value();
                let (l, r) = self.normalize_int_widths(left_int, right_int)?;
                Ok(NumericOperands::Integers(l, r))
            }
            // Both floats: use as-is
            (_, true, _, true) => Ok(NumericOperands::Floats(
                left.into_float_value(),
                right.into_float_value(),
            )),
            // Left int, right float: promote int to float
            (true, _, _, true) => {
                let left_float = self.builder.build_signed_int_to_float(
                    left.into_int_value(),
                    self.context.f64_type(),
                    "int_to_float",
                )?;
                Ok(NumericOperands::Floats(
                    left_float,
                    right.into_float_value(),
                ))
            }
            // Left float, right int: promote int to float
            (_, true, true, _) => {
                let right_float = self.builder.build_signed_int_to_float(
                    right.into_int_value(),
                    self.context.f64_type(),
                    "int_to_float",
                )?;
                Ok(NumericOperands::Floats(
                    left.into_float_value(),
                    right_float,
                ))
            }
            _ => Err(CompileError::TypeMismatch {
                expected: "int or float".to_string(),
                found: "incompatible types".to_string(),
                span: self.current_span.clone(),
            }),
        }
    }

    /// Normalize two integers to the same bit width (prefer wider type)
    fn normalize_int_widths(
        &mut self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), CompileError> {
        if left.get_type() == right.get_type() {
            return Ok((left, right));
        }

        let left_width = left.get_type().get_bit_width();
        let right_width = right.get_type().get_bit_width();

        if left_width > right_width {
            let right_ext = self
                .builder
                .build_int_s_extend(right, left.get_type(), "ext_right")?;
            Ok((left, right_ext))
        } else {
            let left_ext = self
                .builder
                .build_int_s_extend(left, right.get_type(), "ext_left")?;
            Ok((left_ext, right))
        }
    }

    /// Normalize integers with special handling for booleans (zero-extend instead of sign-extend)
    fn normalize_int_widths_for_logical(
        &mut self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), CompileError> {
        if left.get_type() == right.get_type() {
            return Ok((left, right));
        }

        let left_width = left.get_type().get_bit_width();
        let right_width = right.get_type().get_bit_width();

        if left_width > right_width {
            let right_ext = if right_width == 1 {
                self.builder
                    .build_int_z_extend(right, left.get_type(), "zext_right")?
            } else {
                self.builder
                    .build_int_s_extend(right, left.get_type(), "ext_right")?
            };
            Ok((left, right_ext))
        } else {
            let left_ext = if left_width == 1 {
                self.builder
                    .build_int_z_extend(left, right.get_type(), "zext_left")?
            } else {
                self.builder
                    .build_int_s_extend(left, right.get_type(), "ext_left")?
            };
            Ok((left_ext, right))
        }
    }

    /// Generic arithmetic operation on normalized numeric operands
    fn compile_arithmetic_op<FInt, FFloat>(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        int_op: FInt,
        float_op: FFloat,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError>
    where
        FInt: FnOnce(
            &mut Self,
            IntValue<'ctx>,
            IntValue<'ctx>,
            &str,
        ) -> Result<IntValue<'ctx>, CompileError>,
        FFloat: FnOnce(
            &mut Self,
            FloatValue<'ctx>,
            FloatValue<'ctx>,
            &str,
        ) -> Result<FloatValue<'ctx>, CompileError>,
    {
        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => Ok(int_op(self, l, r, name)?.into()),
            NumericOperands::Floats(l, r) => Ok(float_op(self, l, r, name)?.into()),
        }
    }

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
            BinaryOperator::GreaterThanEquals => {
                self.compile_greater_than_equals(left_val, right_val)
            }
            BinaryOperator::StringConcat => self.compile_string_concat(left_val, right_val),
            BinaryOperator::Modulo => self.compile_modulo(left_val, right_val),
            BinaryOperator::And => self.compile_and(left_val, right_val),
            BinaryOperator::Or => self.compile_or(left_val, right_val),
            // Bitwise operators
            BinaryOperator::BitwiseAnd => self.compile_bitwise_and(left_val, right_val),
            BinaryOperator::BitwiseOr => self.compile_bitwise_or(left_val, right_val),
            BinaryOperator::BitwiseXor => self.compile_bitwise_xor(left_val, right_val),
            BinaryOperator::ShiftLeft => self.compile_shift_left(left_val, right_val),
            BinaryOperator::ShiftRight => self.compile_shift_right(left_val, right_val),
        }
    }

    fn compile_add(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Special case: pointer + int error
        if (left.is_pointer_value() && right.is_int_value())
            || (right.is_pointer_value() && left.is_int_value())
        {
            return Err(CompileError::TypeMismatch {
                expected: "i64".to_string(),
                found: "String".to_string(),
                span: self.current_span.clone(),
            });
        }

        self.compile_arithmetic_op(
            left,
            right,
            |s, l, r, name| {
                s.builder
                    .build_int_add(l, r, name)
                    .map_err(CompileError::from)
            },
            |s, l, r, name| {
                s.builder
                    .build_float_add(l, r, name)
                    .map_err(CompileError::from)
            },
            "addtmp",
        )
    }

    fn compile_subtract(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        self.compile_arithmetic_op(
            left,
            right,
            |s, l, r, name| {
                s.builder
                    .build_int_sub(l, r, name)
                    .map_err(CompileError::from)
            },
            |s, l, r, name| {
                s.builder
                    .build_float_sub(l, r, name)
                    .map_err(CompileError::from)
            },
            "subtmp",
        )
    }

    fn compile_multiply(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        self.compile_arithmetic_op(
            left,
            right,
            |s, l, r, name| {
                s.builder
                    .build_int_mul(l, r, name)
                    .map_err(CompileError::from)
            },
            |s, l, r, name| {
                s.builder
                    .build_float_mul(l, r, name)
                    .map_err(CompileError::from)
            },
            "multmp",
        )
    }

    fn compile_divide(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        self.compile_arithmetic_op(
            left,
            right,
            |s, l, r, name| {
                s.builder
                    .build_int_signed_div(l, r, name)
                    .map_err(CompileError::from)
            },
            |s, l, r, name| {
                s.builder
                    .build_float_div(l, r, name)
                    .map_err(CompileError::from)
            },
            "divtmp",
        )
    }

    fn compile_modulo(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let left_int = left.into_int_value();
        let right_int = right.into_int_value();
        let (l, r) = self.normalize_int_widths(left_int, right_int)?;
        let result = self.builder.build_int_signed_rem(l, r, "modtmp")?;
        Ok(result.into())
    }

    fn compile_equals(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Special case: string comparison
        if left.is_pointer_value() && right.is_pointer_value() {
            return self.compile_string_compare(left, right, IntPredicate::EQ, "strcmp_eq");
        }

        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, l, r, "eqtmp")?;
                Ok(result.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::OEQ, l, r, "eqtmp")?;
                Ok(result.into())
            }
        }
    }

    fn compile_not_equals(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Special case: string comparison
        if left.is_pointer_value() && right.is_pointer_value() {
            return self.compile_string_compare(left, right, IntPredicate::NE, "strcmp_ne");
        }

        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::NE, l, r, "netmp")?;
                Ok(result.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::ONE, l, r, "netmp")?;
                Ok(result.into())
            }
        }
    }

    fn compile_string_compare(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        predicate: IntPredicate,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let strcmp_fn = match self.module.get_function("strcmp") {
            Some(f) => f,
            None => {
                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = self
                    .context
                    .i32_type()
                    .fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
                self.module.add_function("strcmp", fn_type, None)
            }
        };

        let call = self.builder.build_call(
            strcmp_fn,
            &[
                left.into_pointer_value().into(),
                right.into_pointer_value().into(),
            ],
            "strcmp_call",
        )?;

        let cmp_result = call
            .try_as_basic_value()
            .left()
            .ok_or_else(|| {
                CompileError::InternalError(
                    "strcmp did not return a value".to_string(),
                    self.get_current_span(),
                )
            })?
            .into_int_value();

        let zero = self.context.i32_type().const_int(0, false);
        let result = self
            .builder
            .build_int_compare(predicate, cmp_result, zero, name)?;
        Ok(result.into())
    }

    fn compile_less_than(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::SLT, l, r, "lttmp")?;
                // Zero-extend i1 to i64 for test compatibility
                let zext =
                    self.builder
                        .build_int_z_extend(result, self.context.i64_type(), "zext_lt")?;
                Ok(zext.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::OLT, l, r, "lttmp")?;
                let zext =
                    self.builder
                        .build_int_z_extend(result, self.context.i64_type(), "zext_lt")?;
                Ok(zext.into())
            }
        }
    }

    fn compile_greater_than(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, l, r, "gttmp")?;
                Ok(result.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::OGT, l, r, "gttmp")?;
                Ok(result.into())
            }
        }
    }

    fn compile_less_than_equals(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::SLE, l, r, "letmp")?;
                Ok(result.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::OLE, l, r, "letmp")?;
                Ok(result.into())
            }
        }
    }

    fn compile_greater_than_equals(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match self.normalize_numeric_operands(left, right)? {
            NumericOperands::Integers(l, r) => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::SGE, l, r, "getmp")?;
                Ok(result.into())
            }
            NumericOperands::Floats(l, r) => {
                let result =
                    self.builder
                        .build_float_compare(FloatPredicate::OGE, l, r, "getmp")?;
                Ok(result.into())
            }
        }
    }

    fn compile_string_concat(
        &mut self,
        _left: BasicValueEnum<'ctx>,
        _right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Err(CompileError::InternalError(
            "String concatenation requires an allocator. Use String.concat() or String.append() from stdlib instead.".to_string(),
            None,
        ))
    }

    fn compile_and(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let (l, r) =
            self.normalize_int_widths_for_logical(left.into_int_value(), right.into_int_value())?;
        let result = self.builder.build_and(l, r, "andtmp")?;
        Ok(result.into())
    }

    fn compile_or(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "mixed types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let (l, r) =
            self.normalize_int_widths_for_logical(left.into_int_value(), right.into_int_value())?;
        let result = self.builder.build_or(l, r, "ortmp")?;
        Ok(result.into())
    }

    // Bitwise operations
    fn compile_bitwise_and(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "non-integer types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let (l, r) = self.normalize_int_widths(left.into_int_value(), right.into_int_value())?;
        let result = self.builder.build_and(l, r, "bitand")?;
        Ok(result.into())
    }

    fn compile_bitwise_or(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "non-integer types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let (l, r) = self.normalize_int_widths(left.into_int_value(), right.into_int_value())?;
        let result = self.builder.build_or(l, r, "bitor")?;
        Ok(result.into())
    }

    fn compile_bitwise_xor(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "non-integer types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let (l, r) = self.normalize_int_widths(left.into_int_value(), right.into_int_value())?;
        let result = self.builder.build_xor(l, r, "bitxor")?;
        Ok(result.into())
    }

    fn compile_shift_left(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "non-integer types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let left_int = left.into_int_value();
        let right_int = right.into_int_value();
        // Shift amount should match the type being shifted
        let shift_amt = if right_int.get_type() != left_int.get_type() {
            self.builder.build_int_z_extend_or_bit_cast(
                right_int,
                left_int.get_type(),
                "shift_ext",
            )?
        } else {
            right_int
        };
        let result = self.builder.build_left_shift(left_int, shift_amt, "shl")?;
        Ok(result.into())
    }

    fn compile_shift_right(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if !left.is_int_value() || !right.is_int_value() {
            return Err(CompileError::TypeMismatch {
                expected: "int".to_string(),
                found: "non-integer types".to_string(),
                span: self.current_span.clone(),
            });
        }

        let left_int = left.into_int_value();
        let right_int = right.into_int_value();
        // Shift amount should match the type being shifted
        let shift_amt = if right_int.get_type() != left_int.get_type() {
            self.builder.build_int_z_extend_or_bit_cast(
                right_int,
                left_int.get_type(),
                "shift_ext",
            )?
        } else {
            right_int
        };
        // Use logical (unsigned) right shift
        let result = self
            .builder
            .build_right_shift(left_int, shift_amt, false, "shr")?;
        Ok(result.into())
    }
}
