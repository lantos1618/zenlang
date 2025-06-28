use super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, BasicValue};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Integer8(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Integer16(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Integer32(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Integer64(value) => {
                self.compile_integer_literal(*value)
            }
            Expression::Unsigned8(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Unsigned16(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Unsigned32(value) => {
                self.compile_integer_literal(*value as i64)
            }
            Expression::Unsigned64(value) => {
                self.compile_integer_literal(*value as i64)
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
            Expression::Comptime(expr) => {
                self.compile_comptime_expression(expr)
            }
            Expression::Range { start, end, inclusive } => {
                self.compile_range_expression(start, end, *inclusive)
            }
            Expression::PatternMatch { scrutinee, arms } => {
                self.compile_pattern_match(scrutinee, arms)
            }
        }
    }

    fn compile_conditional_expression(&mut self, scrutinee: &Expression, arms: &[crate::ast::ConditionalArm]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let parent_function = self.current_function
            .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;
        
        // Compile the scrutinee expression
        let cond_val = self.compile_expression(scrutinee)?;
        
        // Ensure it's a boolean value
        if !cond_val.is_int_value() || cond_val.into_int_value().get_type().get_bit_width() != 1 {
            return Err(CompileError::TypeMismatch {
                expected: "boolean (i1) for conditional expression".to_string(),
                found: format!("{:?}", cond_val.get_type()),
                span: None,
            });
        }
        
        // Create basic blocks
        let then_bb = self.context.append_basic_block(parent_function, "then");
        let else_bb = self.context.append_basic_block(parent_function, "else");
        let merge_bb = self.context.append_basic_block(parent_function, "ifcont");
        
        // Branch based on condition
        self.builder.build_conditional_branch(cond_val.into_int_value(), then_bb, else_bb)?;
        
        // Emit 'then' block
        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expression(&arms[0].body)?;
        self.builder.build_unconditional_branch(merge_bb)?;
        let then_bb_end = self.builder.get_insert_block().unwrap();
        
        // Emit 'else' block
        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expression(&arms[1].body)?;
        self.builder.build_unconditional_branch(merge_bb)?;
        let else_bb_end = self.builder.get_insert_block().unwrap();
        
        // Emit 'merge' block
        self.builder.position_at_end(merge_bb);
        let phi = self.builder.build_phi(
            then_val.get_type(),
            "iftmp"
        )?;
        
        // Add incoming values to phi node
        phi.add_incoming(&[(&then_val, then_bb_end), (&else_val, else_bb_end)]);
        
        Ok(phi.as_basic_value())
    }

    fn compile_array_literal(&mut self, elements: &[Expression]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, treat all arrays as arrays of i64
        let element_type = self.context.i64_type();
        let array_len = elements.len() as u32;
        let array_type = element_type.array_type(array_len);

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
        // For now, represent enums as a struct { tag: i64, payload: i64 }
        // Tag is the variant index, payload is the value (or 0 if none)
        // In the future, this should use the real enum type info
        let tag = 0; // TODO: look up variant index from enum definition
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
        Ok(loaded)
    }

    fn compile_member_access(&mut self, object: &Expression, member: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Delegate to the struct field access logic
        self.compile_struct_field(object, member)
    }

    fn compile_comptime_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, compile comptime expressions the same as regular expressions
        // In the future, this should evaluate at compile time and return constants
        self.compile_expression(expr)
    }

    fn compile_pattern_match(&mut self, _scrutinee: &Expression, arms: &[crate::ast::PatternArm]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, implement a simple pattern matching that evaluates the first arm
        // In the future, this should implement proper pattern matching with guards
        if let Some(first_arm) = arms.first() {
            self.compile_expression(&first_arm.body)
        } else {
            Err(CompileError::InternalError("Pattern match with no arms".to_string(), None))
        }
    }

    fn compile_range_expression(&mut self, start: &Expression, end: &Expression, inclusive: bool) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, represent ranges as a simple struct { start: i64, end: i64, inclusive: bool }
        let start_val = self.compile_expression(start)?;
        let end_val = self.compile_expression(end)?;
        
        // Create a simple struct type for the range
        let range_struct_type = self.context.struct_type(&[
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