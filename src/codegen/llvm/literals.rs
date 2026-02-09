use super::{symbols, LLVMCompiler};
use crate::ast::AstType;
use crate::error::CompileError;
use crate::stdlib_types::StdlibTypeRegistry;
use inkwell::values::BasicValueEnum;

impl<'ctx> LLVMCompiler<'ctx> {
    // Expression compilation methods for literals
    pub fn compile_integer_literal(
        &self,
        value: i64,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self
            .context
            .i64_type()
            .const_int(value as u64, false)
            .into())
    }

    pub fn compile_float_literal(&self, value: f64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.f64_type().const_float(value).into())
    }

    pub fn compile_string_literal(
        &mut self,
        val: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr = self.builder.build_global_string_ptr(val, "str")?;
        let ptr_val = ptr.as_pointer_value();
        // Always return the pointer value, don't convert to integer
        // This fixes the issue where string literals were being converted to integers
        // when used as function arguments, breaking string operations
        Ok(ptr_val.into())
    }

    pub fn compile_identifier(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Check if this is an enum type name (will be handled later in member access)
        if let Some(symbols::Symbol::EnumType(_)) = self.symbols.lookup(name) {
            // Return a dummy value - this will be handled properly in member access
            // We can't create an enum variant without knowing which variant
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        // Check if this is a struct type name (e.g., Array) that has static methods
        if let Some(symbols::Symbol::StructType(_)) = self.symbols.lookup(name) {
            // Return a dummy value - this will be handled properly in member access
            // Static methods like Array.new() will be handled later
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        // Also check the struct_types map for built-in types like Array
        if self.struct_types.contains_key(name) {
            // Return a dummy value - this will be handled properly in member access
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        // First check if this is a function name
        if let Some(function) = self.module.get_function(name) {
            // Return the function's address as a pointer value
            Ok(function.as_global_value().as_pointer_value().into())
        } else {
            // It's a variable, get the pointer
            let (ptr, ast_type) = self.get_variable(name)?;

            // Load the value from the alloca based on type
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let load_name = format!("{}_load", name);

            let loaded: BasicValueEnum = match &ast_type {
                AstType::Bool => self.builder.build_load(self.context.bool_type(), ptr, "")?,
                AstType::I32 => {
                    self.builder
                        .build_load(self.context.i32_type(), ptr, &load_name)?
                }
                AstType::I64 | AstType::StdModule => {
                    self.builder
                        .build_load(self.context.i64_type(), ptr, &load_name)?
                }
                AstType::F32 => self.builder.build_load(self.context.f32_type(), ptr, "")?,
                AstType::F64 => self.builder.build_load(self.context.f64_type(), ptr, "")?,
                AstType::StaticLiteral | AstType::StaticString | AstType::Function { .. } => {
                    self.builder.build_load(ptr_type, ptr, "")?
                }
                AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => {
                    self.builder.build_load(ptr_type, ptr, "")?
                }
                t if t.is_ptr_type() => self.builder.build_load(ptr_type, ptr, "")?,
                AstType::EnumType { .. } => {
                    let enum_struct_type = self.context.struct_type(
                        &[
                            self.context.i64_type().into(),
                            self.context.i64_type().into(),
                        ],
                        false,
                    );
                    self.builder.build_load(enum_struct_type, ptr, "")?
                }
                AstType::Generic {
                    name: enum_name, ..
                } if self.well_known.is_option(enum_name)
                    || self.well_known.is_result(enum_name) =>
                {
                    let enum_struct_type = self.well_known_enum_type();
                    self.builder.build_load(enum_struct_type, ptr, "")?
                }
                _ => {
                    let elem_type = self.to_llvm_type(&ast_type)?;
                    let basic_type = self.expect_basic_type(elem_type)?;
                    self.builder.build_load(basic_type, ptr, "")?
                }
            };
            Ok(loaded)
        }
    }

    pub fn compile_string_interpolation(
        &mut self,
        parts: &[crate::ast::StringPart],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        use crate::ast::StringPart;
        use inkwell::values::BasicMetadataValueEnum;
        use inkwell::AddressSpace;

        // First, calculate the total size needed for the string
        // For now, we'll use a simple approach with sprintf for numeric values

        // Declare snprintf if not already declared (safer than sprintf - prevents buffer overflow)
        let snprintf_fn = self.module.get_function("snprintf").unwrap_or_else(|| {
            let i32_type = self.context.i32_type();
            let i64_type = self.context.i64_type();
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            // snprintf(char *str, size_t size, const char *format, ...)
            let fn_type =
                i32_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], true);
            self.module.add_function("snprintf", fn_type, None)
        });

        // Build the format string and collect interpolated values
        let mut format_string = String::new();
        let mut values: Vec<BasicMetadataValueEnum> = Vec::new();

        for part in parts {
            match part {
                StringPart::Literal(s) => {
                    format_string.push_str(s);
                }
                StringPart::Interpolation(expr) => {
                    let val = self.compile_expression(expr)?;

                    // Handle different value types for interpolation
                    let (format_spec, actual_val) = if val.is_struct_value() {
                        // This could be an enum (Option, Result, etc)
                        let struct_val = val.into_struct_value();

                        // For enums, we check if it has a discriminant field
                        if struct_val.get_type().count_fields() >= 1 {
                            // Extract the discriminant (tag)
                            let tag = self
                                .builder
                                .build_extract_value(struct_val, 0, "enum_tag")?;

                            // Check if this is Option type (Some=0, None=1)
                            let tag_int = tag.into_int_value();
                            let zero = tag_int.get_type().const_zero();
                            let is_some = self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ,
                                tag_int,
                                zero,
                                "is_some",
                            )?;

                            // Get the string representations
                            let some_str = if struct_val.get_type().count_fields() > 1 {
                                // Has payload - extract and format it
                                let _payload =
                                    self.builder.build_extract_value(struct_val, 1, "payload")?;
                                // For now, format the payload as an integer
                                // In a full implementation, we'd recursively format the payload
                                let formatted = self
                                    .builder
                                    .build_global_string_ptr("Some(...)", "some_str")?;
                                formatted.as_pointer_value()
                            } else {
                                let formatted =
                                    self.builder.build_global_string_ptr("Some", "some_str")?;
                                formatted.as_pointer_value()
                            };

                            let none_str =
                                self.builder.build_global_string_ptr("None", "none_str")?;

                            let result = self.builder.build_select(
                                is_some,
                                some_str,
                                none_str.as_pointer_value(),
                                "option_str",
                            )?;

                            ("%s", result.into())
                        } else {
                            // Unknown struct - use a default representation
                            (
                                "%s",
                                self.builder
                                    .build_global_string_ptr("<struct>", "struct_str")?
                                    .as_pointer_value()
                                    .into(),
                            )
                        }
                    } else if val.is_int_value() {
                        let int_val = val.into_int_value();
                        let bit_width = int_val.get_type().get_bit_width();
                        match bit_width {
                            1 => {
                                // This is a boolean - convert to "true"/"false"
                                let true_str =
                                    self.builder.build_global_string_ptr("true", "true_str")?;
                                let false_str =
                                    self.builder.build_global_string_ptr("false", "false_str")?;

                                let is_true = self.builder.build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    int_val,
                                    int_val.get_type().const_zero(),
                                    "is_true",
                                )?;

                                let result = self.builder.build_select(
                                    is_true,
                                    true_str.as_pointer_value(),
                                    false_str.as_pointer_value(),
                                    "bool_str",
                                )?;

                                ("%s", result.into())
                            }
                            8 => {
                                // For 8-bit integers, extend to 32-bit to ensure proper printing
                                let extended = self.builder.build_int_z_extend(
                                    int_val,
                                    self.context.i32_type(),
                                    "i8_to_i32",
                                )?;
                                ("%d", extended.into())
                            }
                            32 => ("%d", val.into()),
                            64 => ("%lld", val.into()),
                            _ => ("%d", val.into()),
                        }
                    } else if val.is_float_value() {
                        ("%.6f", val.into())
                    } else if val.is_pointer_value() {
                        // Pointer values are strings - use %s
                        ("%s", val.into())
                    } else {
                        // Default to string format
                        ("%s", val.into())
                    };

                    format_string.push_str(format_spec);
                    values.push(actual_val);
                }
            }
        }

        // Allocate buffer dynamically using malloc to avoid stack corruption issues
        // with multiple string interpolations in the same function
        let buffer_size = 256u64; // Reduced from 1024 - sufficient for most interpolations

        // Get or declare malloc
        let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
            let i64_type = self.context.i64_type();
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
            self.module.add_function("malloc", fn_type, None)
        });

        // Allocate the buffer
        let buffer_size_val = self.context.i64_type().const_int(buffer_size, false);
        let buffer_call =
            self.builder
                .build_call(malloc_fn, &[buffer_size_val.into()], "str_buffer")?;
        let buffer_ptr = buffer_call
            .try_as_basic_value()
            .left()
            .ok_or_else(|| {
                CompileError::InternalError(
                    "malloc should return a pointer".to_string(),
                    self.get_current_span(),
                )
            })?
            .into_pointer_value();

        // Build the format string
        let format_ptr = self
            .builder
            .build_global_string_ptr(&format_string, "format")?;

        // Build the snprintf call with buffer size for safety
        let mut snprintf_args: Vec<BasicMetadataValueEnum> = vec![
            buffer_ptr.into(),
            buffer_size_val.into(),
            format_ptr.as_pointer_value().into(),
        ];
        snprintf_args.extend(values);

        self.builder
            .build_call(snprintf_fn, &snprintf_args, "snprintf_call")?;

        // Return the buffer pointer
        Ok(buffer_ptr.into())
    }
}
