use super::*;
use crate::ast::Pattern;

// All code related to expression codegen will be moved here.
// ... existing code from mod.rs for compile_expression and helpers ... 

impl<'ctx> Compiler<'ctx> {
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Integer8(val) => Ok(self.context.i8_type().const_int(*val as u64, false).into()),
            Expression::Integer16(val) => Ok(self.context.i16_type().const_int(*val as u64, false).into()),
            Expression::Integer32(val) => Ok(self.context.i32_type().const_int(*val as u64, false).into()),
            Expression::Integer64(val) => Ok(self.context.i64_type().const_int(*val as u64, false).into()),
            Expression::Unsigned8(val) => Ok(self.context.i8_type().const_int(*val as u64, false).into()),
            Expression::Unsigned16(val) => Ok(self.context.i16_type().const_int(*val as u64, false).into()),
            Expression::Unsigned32(val) => Ok(self.context.i32_type().const_int(*val as u64, false).into()),
            Expression::Unsigned64(val) => Ok(self.context.i64_type().const_int(*val, false).into()),
            Expression::Float32(val) => Ok(self.context.f32_type().const_float((*val).into()).into()),
            Expression::Float64(val) => Ok(self.context.f64_type().const_float(*val).into()),
            Expression::Boolean(val) => Ok(self.context.bool_type().const_int(*val as u64, false).into()),
            Expression::String(val) => {
                let string_global = self.builder.build_global_string_ptr(val, "str").map_err(|e| CompileError::from(e))?;
                Ok(string_global.as_pointer_value().into())
            },
            Expression::Identifier(name) => {
                if let Some((alloca, _)) = self.variables.get(name) {
                    Ok(self.builder.build_load(alloca.get_type(), *alloca, name).map_err(|e| CompileError::from(e))?)
                } else {
                    Err(CompileError::UndeclaredVariable(name.clone(), None))
                }
            },
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;
                
                match (left_val, right_val) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = match op {
                            BinaryOperator::Add => self.builder.build_int_add(l, r, "add").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::Subtract => self.builder.build_int_sub(l, r, "sub").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::Multiply => self.builder.build_int_mul(l, r, "mul").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::Divide => self.builder.build_int_signed_div(l, r, "div").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::Modulo => self.builder.build_int_signed_rem(l, r, "rem").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::Equals => self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::NotEquals => self.builder.build_int_compare(IntPredicate::NE, l, r, "ne").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::LessThan => self.builder.build_int_compare(IntPredicate::SLT, l, r, "lt").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::GreaterThan => self.builder.build_int_compare(IntPredicate::SGT, l, r, "gt").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::LessThanEquals => self.builder.build_int_compare(IntPredicate::SLE, l, r, "le").map_err(|e| CompileError::from(e))?,
                            BinaryOperator::GreaterThanEquals => self.builder.build_int_compare(IntPredicate::SGE, l, r, "ge").map_err(|e| CompileError::from(e))?,
                            _ => return Err(CompileError::TypeError("Unsupported binary operator for integers".to_string(), None)),
                        };
                        Ok(result.into())
                    },
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        match op {
                            BinaryOperator::Add => {
                                let result = self.builder.build_float_add(l, r, "fadd").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::Subtract => {
                                let result = self.builder.build_float_sub(l, r, "fsub").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::Multiply => {
                                let result = self.builder.build_float_mul(l, r, "fmul").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::Divide => {
                                let result = self.builder.build_float_div(l, r, "fdiv").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::Equals => {
                                let result = self.builder.build_float_compare(FloatPredicate::OEQ, l, r, "feq").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::NotEquals => {
                                let result = self.builder.build_float_compare(FloatPredicate::ONE, l, r, "fne").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::LessThan => {
                                let result = self.builder.build_float_compare(FloatPredicate::OLT, l, r, "flt").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::GreaterThan => {
                                let result = self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fgt").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::LessThanEquals => {
                                let result = self.builder.build_float_compare(FloatPredicate::OLE, l, r, "fle").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            BinaryOperator::GreaterThanEquals => {
                                let result = self.builder.build_float_compare(FloatPredicate::OGE, l, r, "fge").map_err(|e| CompileError::from(e))?;
                                Ok(result.into())
                            },
                            _ => Err(CompileError::TypeError("Unsupported binary operator for floats".to_string(), None)),
                        }
                    },
                    _ => Err(CompileError::TypeError("Type mismatch in binary operation".to_string(), None)),
                }
            },
            Expression::FunctionCall { name, args } => {
                if let Some(function) = self.module.get_function(name) {
                    let compiled_args: Result<Vec<BasicValueEnum<'ctx>>, CompileError> = args.iter().map(|arg| self.compile_expression(arg)).collect();
                    let compiled_args = compiled_args?;
                    
                    // Convert BasicValueEnum to BasicMetadataValueEnum for function calls
                    let metadata_args: Vec<BasicMetadataValueEnum<'ctx>> = compiled_args.iter().map(|arg| (*arg).into()).collect();
                    
                    let result = self.builder.build_call(function, &metadata_args, "call").map_err(|e| CompileError::from(e))?;
                    Ok(result.try_as_basic_value().left().unwrap_or(self.context.i64_type().const_zero().into()))
                } else {
                    Err(CompileError::UndeclaredFunction(name.clone(), None))
                }
            },
            Expression::Conditional { scrutinee, arms } => {
                // For now, implement a simple conditional
                // TODO: Implement full pattern matching
                if arms.len() >= 2 {
                    let scrutinee_val = self.compile_expression(scrutinee)?;
                    let condition = if let BasicValueEnum::IntValue(int_val) = scrutinee_val {
                        self.builder.build_int_compare(IntPredicate::NE, int_val, self.context.i64_type().const_zero(), "cond").map_err(|e| CompileError::from(e))?
                    } else {
                        return Err(CompileError::TypeError("Conditional scrutinee must be an integer".to_string(), None));
                    };
                    
                    let then_block = self.context.append_basic_block(self.current_function.unwrap(), "then");
                    let else_block = self.context.append_basic_block(self.current_function.unwrap(), "else");
                    let merge_block = self.context.append_basic_block(self.current_function.unwrap(), "merge");
                    
                    self.builder.build_conditional_branch(condition, then_block, else_block).map_err(|e| CompileError::from(e))?;
                    
                    // Compile then branch
                    self.builder.position_at_end(then_block);
                    let then_val = self.compile_expression(&arms[0].body)?;
                    self.builder.build_unconditional_branch(merge_block).map_err(|e| CompileError::from(e))?;
                    
                    // Compile else branch
                    self.builder.position_at_end(else_block);
                    let else_val = self.compile_expression(&arms[1].body)?;
                    self.builder.build_unconditional_branch(merge_block).map_err(|e| CompileError::from(e))?;
                    
                    // Merge
                    self.builder.position_at_end(merge_block);
                    let phi = self.builder.build_phi(then_val.get_type(), "phi").map_err(|e| CompileError::from(e))?;
                    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
                    
                    Ok(phi.as_basic_value())
                } else {
                    Err(CompileError::TypeError("Conditional must have at least 2 arms".to_string(), None))
                }
            },
            Expression::AddressOf(expr) => {
                // For now, just return the expression as a pointer
                // TODO: Implement proper address-of
                let val = self.compile_expression(expr)?;
                Ok(val)
            },
            Expression::Dereference(expr) => {
                // For now, just return the expression
                // TODO: Implement proper dereference
                let val = self.compile_expression(expr)?;
                Ok(val)
            },
            Expression::PointerOffset { pointer, offset } => {
                // For now, just return the pointer
                // TODO: Implement proper pointer arithmetic
                let _pointer_val = self.compile_expression(pointer)?;
                let _offset_val = self.compile_expression(offset)?;
                Ok(self.context.i64_type().const_zero().into())
            },
            Expression::StructLiteral { name: _, fields } => {
                // For now, just return the first field value
                // TODO: Implement proper struct literals
                if let Some((_, expr)) = fields.first() {
                    self.compile_expression(expr)
                } else {
                    Ok(self.context.i64_type().const_zero().into())
                }
            },
            Expression::StructField { struct_: _, field: _ } => {
                // For now, just return zero
                // TODO: Implement proper struct field access
                Ok(self.context.i64_type().const_zero().into())
            },
            Expression::ArrayLiteral(elements) => {
                // For now, just return the first element
                // TODO: Implement proper array literals
                if let Some(first) = elements.first() {
                    self.compile_expression(first)
                } else {
                    Ok(self.context.i64_type().const_zero().into())
                }
            },
            Expression::ArrayIndex { array: _, index: _ } => {
                // For now, just return zero
                // TODO: Implement proper array indexing
                Ok(self.context.i64_type().const_zero().into())
            },
            Expression::EnumVariant { enum_name: _, variant: _, payload: _ } => {
                // For now, just return zero
                // TODO: Implement proper enum variants
                Ok(self.context.i64_type().const_zero().into())
            },
            Expression::MemberAccess { object: _, member: _ } => {
                // For now, just return zero
                // TODO: Implement proper member access
                Ok(self.context.i64_type().const_zero().into())
            },
            Expression::StringLength(expr) => {
                // For now, just return a constant length
                // TODO: Implement proper string length
                let _string_val = self.compile_expression(expr)?;
                Ok(self.context.i64_type().const_int(5, false).into())
            },
            Expression::Comptime(_) => todo!("Comptime expressions not implemented"),
            Expression::Range { start, end: _, inclusive: _ } => {
                // For now, just return the start value
                // TODO: Implement proper range expressions
                self.compile_expression(start)
            },
            Expression::PatternMatch { scrutinee, arms } => {
                // For now, implement a simple pattern match
                // TODO: Implement full pattern matching
                if arms.len() >= 1 {
                    let _scrutinee_val = self.compile_expression(scrutinee)?;
                    self.compile_expression(&arms[0].body)
                } else {
                    Ok(self.context.i64_type().const_zero().into())
                }
            },
        }
    }

    fn compile_pattern(&mut self, pattern: &Pattern) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match pattern {
            Pattern::Literal(expr) => self.compile_expression(expr),
            Pattern::Identifier(name) => {
                if let Some((alloca, _)) = self.variables.get(name) {
                    Ok(self.builder.build_load(alloca.get_type(), *alloca, name).map_err(|e| CompileError::from(e))?)
                } else {
                    Err(CompileError::UndeclaredVariable(name.clone(), None))
                }
            },
            Pattern::Struct { name: _, fields: _ } => {
                // For now, just return zero
                // TODO: Implement proper struct patterns
                Ok(self.context.i64_type().const_zero().into())
            },
            Pattern::EnumVariant { enum_name: _, variant: _, payload: _ } => {
                // For now, just return zero
                // TODO: Implement proper enum variant patterns
                Ok(self.context.i64_type().const_zero().into())
            },
            Pattern::Wildcard => {
                // Wildcard pattern matches anything, return zero for now
                Ok(self.context.i64_type().const_zero().into())
            },
            Pattern::Or(patterns) => {
                // For now, just return the first pattern
                // TODO: Implement proper or patterns
                if let Some(first) = patterns.first() {
                    self.compile_pattern(first)
                } else {
                    Ok(self.context.i64_type().const_zero().into())
                }
            },
            Pattern::Range { start, end: _, inclusive: _ } => {
                // For now, just return the start value
                // TODO: Implement proper range patterns
                self.compile_expression(start)
            },
            Pattern::Binding { name: _, pattern: _ } => {
                // For now, just return zero
                // TODO: Implement proper binding patterns
                Ok(self.context.i64_type().const_zero().into())
            },
        }
    }
} 