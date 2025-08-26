use super::{LLVMCompiler, symbols};
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, BasicValue};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Integer8(value) => {
                Ok(self.context.i8_type().const_int(*value as u64, true).into())
            }
            Expression::Integer16(value) => {
                Ok(self.context.i16_type().const_int(*value as u64, true).into())
            }
            Expression::Integer32(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, true).into())
            }
            Expression::Integer64(value) => {
                Ok(self.context.i64_type().const_int(*value as u64, true).into())
            }
            Expression::Unsigned8(value) => {
                Ok(self.context.i8_type().const_int(*value as u64, false).into())
            }
            Expression::Unsigned16(value) => {
                Ok(self.context.i16_type().const_int(*value as u64, false).into())
            }
            Expression::Unsigned32(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, false).into())
            }
            Expression::Unsigned64(value) => {
                Ok(self.context.i64_type().const_int(*value, false).into())
            }
            Expression::Float32(value) => {
                self.compile_float_literal(*value as f64)
            }
            Expression::Float64(value) => {
                self.compile_float_literal(*value)
            }
            Expression::Boolean(value) => {
                Ok(self.context.bool_type().const_int(*value as u64, false).into())
            }
            Expression::String(value) => {
                self.compile_string_literal(value)
            }
            Expression::Identifier(name) => {
                self.compile_identifier(name)
            }
            Expression::BinaryOp { left, op, right } => {
                self.compile_binary_operation(op, left, right)
            }
            Expression::FunctionCall { name, args } => {
                self.compile_function_call(name, args)
            }
            Expression::Conditional { scrutinee, arms } => {
                self.compile_conditional_expression(scrutinee, arms)
            }
            Expression::AddressOf(expr) => {
                self.compile_address_of(expr)
            }
            Expression::Dereference(expr) => {
                self.compile_dereference(expr)
            }
            Expression::PointerOffset { pointer, offset } => {
                self.compile_pointer_offset(pointer, offset)
            }
            Expression::StructLiteral { name, fields } => {
                self.compile_struct_literal(name, fields)
            }
            Expression::StructField { struct_, field } => {
                self.compile_struct_field(struct_, field)
            }
            Expression::ArrayLiteral(elements) => {
                self.compile_array_literal(elements)
            }
            Expression::ArrayIndex { array, index } => {
                self.compile_array_index(array, index)
            }
            Expression::EnumVariant { enum_name, variant, payload } => {
                self.compile_enum_variant(enum_name, variant, payload)
            }
            Expression::MemberAccess { object, member } => {
                self.compile_member_access(object, member)
            }
            Expression::StringLength(expr) => {
                self.compile_string_length(expr)
            }
            Expression::StringInterpolation { parts } => {
                self.compile_string_interpolation(parts)
            }
            Expression::Comptime(expr) => {
                self.compile_comptime_expression(expr)
            }
            Expression::Range { start, end, inclusive } => {
                self.compile_range_expression(start, end, *inclusive)
            }
            Expression::PatternMatch { scrutinee, arms } => {
                self.compile_pattern_match(scrutinee, arms)
            }
            Expression::StdModule(_module) => {
                // For now, std modules return a placeholder value
                // This will be expanded when we implement the module system
                Ok(self.context.i32_type().const_int(0, false).into())
            }
            Expression::Module(_module) => {
                // For now, modules return a placeholder value
                // This will be expanded when we implement the module system  
                Ok(self.context.i32_type().const_int(0, false).into())
            }
        }
    }

    fn compile_conditional_expression(&mut self, scrutinee: &Expression, arms: &[crate::ast::ConditionalArm]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let parent_function = self.current_function
            .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;
        
        // Compile the scrutinee expression
        let scrutinee_val = self.compile_expression(scrutinee)?;
        
        // Create the merge block where all arms will jump to
        let merge_bb = self.context.append_basic_block(parent_function, "match_merge");
        
        // We'll collect the values and blocks for the phi node
        let mut phi_values = Vec::new();
        
        // Track the current "next" block for fallthrough
        let mut _current_block = self.builder.get_insert_block().unwrap();
        
        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;
            
            // Test the pattern
            let (matches, bindings) = self.compile_pattern_test(&scrutinee_val, &arm.pattern)?;
            
            // Check guard if present
            let final_condition = if let Some(guard_expr) = &arm.guard {
                // Save current variables before applying bindings
                let saved_vars = self.apply_pattern_bindings(&bindings);
                
                // Compile the guard expression
                let guard_val = self.compile_expression(guard_expr)?;
                
                // Restore variables
                self.restore_variables(saved_vars);
                
                // The final condition is: pattern matches AND guard is true
                if !guard_val.is_int_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "boolean for guard expression".to_string(),
                        found: format!("{:?}", guard_val.get_type()),
                        span: None,
                    });
                }
                
                self.builder.build_and(matches, guard_val.into_int_value(), "guard_and_pattern")?   
            } else {
                matches
            };
            
            // Create blocks for this arm
            let match_bb = self.context.append_basic_block(parent_function, &format!("match_{}", i));
            let next_bb = if !is_last {
                self.context.append_basic_block(parent_function, &format!("test_{}", i + 1))
            } else {
                // For the last arm, we don't need a "next" block
                match_bb  // dummy value, won't be used
            };
            
            // Branch based on the condition
            if !is_last {
                self.builder.build_conditional_branch(final_condition, match_bb, next_bb)?;
            } else {
                // Last arm - if it doesn't match, it's an error (shouldn't happen with wildcard)
                self.builder.build_conditional_branch(final_condition, match_bb, match_bb)?;
            }
            
            // Generate code for the match block
            self.builder.position_at_end(match_bb);
            
            // Apply pattern bindings
            let saved_vars = self.apply_pattern_bindings(&bindings);
            
            // Compile the arm body
            let arm_val = self.compile_expression(&arm.body)?;
            
            // Restore variables
            self.restore_variables(saved_vars);
            
            // Jump to merge block
            self.builder.build_unconditional_branch(merge_bb)?;
            let match_bb_end = self.builder.get_insert_block().unwrap();
            
            // Save value and block for phi node
            phi_values.push((arm_val, match_bb_end));
            
            // Position at the next test block for the next iteration
            if !is_last {
                self.builder.position_at_end(next_bb);
                _current_block = next_bb;
            }
        }
        
        // Position at merge block and create phi node
        self.builder.position_at_end(merge_bb);
        
        if phi_values.is_empty() {
            return Err(CompileError::InternalError("No arms in conditional expression".to_string(), None));
        }
        
        // All values should have the same type
        let result_type = phi_values[0].0.get_type();
        let phi = self.builder.build_phi(result_type, "match_result")?;
        
        // Add all incoming values
        for (value, block) in &phi_values {
            phi.add_incoming(&[(value, *block)]);
        }
        
        Ok(phi.as_basic_value())
    }

    fn compile_array_literal(&mut self, elements: &[Expression]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, treat all arrays as arrays of i64
        let element_type = self.context.i64_type();
        let array_len = elements.len() as u32;
        let _array_type = element_type.array_type(array_len);

        // Allocate the array on the heap (malloc)
        let i64_type = self.context.i64_type();
        let elem_size = i64_type.size_of();
        let total_size = i64_type.const_int(array_len as u64, false);
        let malloc_fn = self.module.get_function("malloc").ok_or_else(|| CompileError::InternalError("No malloc function declared".to_string(), None))?;
        let size = self.builder.build_int_mul(elem_size, total_size, "arraysize");
        let raw_ptr = self.builder.build_call(malloc_fn, &[size?.into()], "arraymalloc")?.try_as_basic_value().left().unwrap().into_pointer_value();
        let array_ptr = self.builder.build_pointer_cast(raw_ptr, self.context.ptr_type(inkwell::AddressSpace::default()), "arrayptr")?;

        // Store each element
        for (i, expr) in elements.iter().enumerate() {
            let value = self.compile_expression(expr)?;
            let gep = unsafe {
                self.builder.build_gep(element_type, array_ptr, &[element_type.const_int(i as u64, false)], &format!("arrayidx{}", i))?
            };
            self.builder.build_store(gep, value)?;
        }
        Ok(array_ptr.as_basic_value_enum())
    }

    fn compile_array_index(&mut self, array: &Expression, index: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, treat all arrays as arrays of i64
        let element_type = self.context.i64_type();
        let array_ptr = self.compile_expression(array)?.into_pointer_value();
        let index_val = self.compile_expression(index)?;
        let gep = unsafe {
            self.builder.build_gep(element_type, array_ptr, &[index_val.into_int_value()], "arrayidx")?
        };
        let loaded = self.builder.build_load(element_type, gep, "arrayload")?;
        Ok(loaded)
    }

    fn compile_enum_variant(&mut self, enum_name: &str, variant: &str, payload: &Option<Box<Expression>>) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Look up the enum info from the symbol table
        let enum_info = match self.symbols.lookup(enum_name) {
            Some(symbols::Symbol::EnumType(info)) => info.clone(),
            _ => {
                // Fallback to basic representation if enum not found in symbol table
                // This maintains backward compatibility
                let tag = 0;
                let tag_val = self.context.i64_type().const_int(tag, false);
                let payload_val = if let Some(expr) = payload {
                    self.compile_expression(expr)?
                } else {
                    self.context.i64_type().const_int(0, false).into()
                };
                let enum_struct_type = self.context.struct_type(&[
                    self.context.i64_type().into(),
                    self.context.i64_type().into(),
                ], false);
                let alloca = self.builder.build_alloca(enum_struct_type, &format!("{}_{}_enum_tmp", enum_name, variant))?;
                let tag_ptr = self.builder.build_struct_gep(enum_struct_type, alloca, 0, "tag_ptr")?;
                self.builder.build_store(tag_ptr, tag_val)?;
                let payload_ptr = self.builder.build_struct_gep(enum_struct_type, alloca, 1, "payload_ptr")?;
                self.builder.build_store(payload_ptr, payload_val)?;
                let loaded = self.builder.build_load(enum_struct_type, alloca, &format!("{}_{}_enum_val", enum_name, variant))?;
                return Ok(loaded);
            }
        };
        
        // Look up the variant index
        let tag = enum_info.variant_indices.get(variant)
            .copied()
            .ok_or_else(|| CompileError::UndeclaredVariable(
                format!("Unknown variant '{}' for enum '{}'", variant, enum_name),
                None
            ))?;
        
        let tag_val = self.context.i64_type().const_int(tag, false);
        let payload_val = if let Some(expr) = payload {
            self.compile_expression(expr)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };
        
        // Use the enum's LLVM type
        let enum_struct_type = enum_info.llvm_type;
        let alloca = self.builder.build_alloca(enum_struct_type, &format!("{}_{}_enum_tmp", enum_name, variant))?;
        let tag_ptr = self.builder.build_struct_gep(enum_struct_type, alloca, 0, "tag_ptr")?;
        self.builder.build_store(tag_ptr, tag_val)?;
        let payload_ptr = self.builder.build_struct_gep(enum_struct_type, alloca, 1, "payload_ptr")?;
        self.builder.build_store(payload_ptr, payload_val)?;
        let loaded = self.builder.build_load(enum_struct_type, alloca, &format!("{}_{}_enum_val", enum_name, variant))?;
        Ok(loaded)
    }

    fn compile_member_access(&mut self, object: &Expression, member: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Delegate to the struct field access logic
        self.compile_struct_field(object, member)
    }

    fn compile_comptime_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Evaluate the expression at compile time using the persistent evaluator
        match self.comptime_evaluator.evaluate_expression(expr) {
            Ok(value) => {
                // Convert the comptime value to a constant expression and compile it
                let const_expr = value.to_expression();
                self.compile_expression(&const_expr)
            }
            Err(e) => {
                return Err(CompileError::InternalError(
                    format!("Comptime evaluation error: {}", e),
                    None
                ));
            }
        }
    }

    fn compile_pattern_match(&mut self, scrutinee: &Expression, arms: &[crate::ast::PatternArm]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let parent_function = self.current_function
            .ok_or_else(|| CompileError::InternalError("No current function for pattern match".to_string(), None))?;
        
        // Compile the scrutinee expression
        let scrutinee_val = self.compile_expression(scrutinee)?;
        
        // Create the merge block where all arms will jump to
        let merge_bb = self.context.append_basic_block(parent_function, "match_merge");
        
        // We'll collect the values and blocks for the phi node
        let mut phi_values = Vec::new();
        
        // Track the current "next" block for fallthrough
        let mut _current_block = self.builder.get_insert_block().unwrap();
        
        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;
            
            // Test the pattern
            let (matches, bindings) = self.compile_pattern_test(&scrutinee_val, &arm.pattern)?;
            
            // Check guard if present
            let final_condition = if let Some(guard_expr) = &arm.guard {
                // Save current variables before applying bindings
                let saved_vars = self.apply_pattern_bindings(&bindings);
                
                // Compile the guard expression
                let guard_val = self.compile_expression(guard_expr)?;
                
                // Restore variables
                self.restore_variables(saved_vars);
                
                // The final condition is: pattern matches AND guard is true
                if !guard_val.is_int_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "boolean for guard expression".to_string(),
                        found: format!("{:?}", guard_val.get_type()),
                        span: None,
                    });
                }
                
                self.builder.build_and(matches, guard_val.into_int_value(), "guard_and_pattern")?   
            } else {
                matches
            };
            
            // Create blocks for this arm
            let match_bb = self.context.append_basic_block(parent_function, &format!("match_{}", i));
            let next_bb = if !is_last {
                self.context.append_basic_block(parent_function, &format!("test_{}", i + 1))
            } else {
                // For the last arm, we don't need a "next" block
                match_bb  // dummy value, won't be used
            };
            
            // Branch based on the condition
            if !is_last {
                self.builder.build_conditional_branch(final_condition, match_bb, next_bb)?;
            } else {
                // Last arm - if it doesn't match, it's an error (shouldn't happen with wildcard)
                self.builder.build_conditional_branch(final_condition, match_bb, match_bb)?;
            }
            
            // Generate code for the match block
            self.builder.position_at_end(match_bb);
            
            // Apply pattern bindings
            let saved_vars = self.apply_pattern_bindings(&bindings);
            
            // Compile the arm body
            let arm_val = self.compile_expression(&arm.body)?;
            
            // Restore variables
            self.restore_variables(saved_vars);
            
            // Jump to merge block
            self.builder.build_unconditional_branch(merge_bb)?;
            let match_bb_end = self.builder.get_insert_block().unwrap();
            
            // Save value and block for phi node
            phi_values.push((arm_val, match_bb_end));
            
            // Position at the next test block for the next iteration
            if !is_last {
                self.builder.position_at_end(next_bb);
                _current_block = next_bb;
            }
        }
        
        // Position at merge block and create phi node
        self.builder.position_at_end(merge_bb);
        
        if phi_values.is_empty() {
            return Err(CompileError::InternalError("No arms in pattern match expression".to_string(), None));
        }
        
        // All values should have the same type
        let result_type = phi_values[0].0.get_type();
        let phi = self.builder.build_phi(result_type, "match_result")?;
        
        // Add all incoming values
        for (value, block) in &phi_values {
            phi.add_incoming(&[(value, *block)]);
        }
        
        Ok(phi.as_basic_value())
    }

    fn compile_range_expression(&mut self, start: &Expression, end: &Expression, inclusive: bool) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, represent ranges as a simple struct { start: i64, end: i64, inclusive: bool }
        let start_val = self.compile_expression(start)?;
        let end_val = self.compile_expression(end)?;
        
        // Create a simple struct type for the range
        let _range_struct_type = self.context.struct_type(&[
            start_val.get_type(),
            end_val.get_type(),
            self.context.bool_type().into(),
        ], false);
        
        // Create the range struct value
        let range_struct = self.context.const_struct(&[
            start_val,
            end_val,
            self.context.bool_type().const_int(inclusive as u64, false).into(),
        ], false);
        
        Ok(range_struct.as_basic_value_enum())
    }
} 