//! Pattern matching code generation
//!
//! This module handles compilation of pattern matching for Zen's `?` operator.
//! It supports literal patterns, enum patterns, identifier bindings, wildcards, and tuples.

use crate::ast::{self, AstType};
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue, StructValue};

use super::{symbols, LLVMCompiler, VariableInfo};

impl<'ctx> LLVMCompiler<'ctx> {
    // ============================================================================
    // PATTERN MATCHING
    // ============================================================================

    /// Find enum variant tag from symbol table
    pub(super) fn find_enum_variant_tag(
        &self,
        variant: &str,
    ) -> Option<(u64, Option<inkwell::types::StructType<'ctx>>)> {
        for symbol in self.symbols.all_symbols() {
            if let symbols::Symbol::EnumType(info) = symbol {
                if let Some(&tag) = info.variant_indices.get(variant) {
                    return Some((tag, Some(info.llvm_type)));
                }
            }
        }
        None
    }

    /// Compare int value with expected tag using the actual discriminant type
    pub(super) fn compare_with_tag(
        &self,
        int_val: IntValue<'ctx>,
        expected_tag: u64,
    ) -> Result<IntValue<'ctx>, CompileError> {
        // Create constant with the SAME type as the loaded discriminant
        // This avoids unnecessary type extensions and keeps IR clean
        let tag_const = int_val.get_type().const_int(expected_tag, false);
        Ok(self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            int_val,
            tag_const,
            "enum_match",
        )?)
    }

    /// Extract payload bindings from pattern (returns binding name with prefix or empty)
    pub(super) fn extract_payload_binding(
        payload: &Option<Box<ast::Pattern>>,
        prefix: &str,
        value: BasicValueEnum<'ctx>,
    ) -> Vec<(String, BasicValueEnum<'ctx>)> {
        match payload.as_ref().map(|p| p.as_ref()) {
            Some(ast::Pattern::Identifier(name)) => vec![(format!("{}{}", prefix, name), value)],
            Some(ast::Pattern::Wildcard) | None => vec![],
            _ => vec![],
        }
    }

    /// Compare integer scrutinee with literal value
    pub(super) fn compare_int_literal(
        &self,
        scrutinee_int: IntValue<'ctx>,
        literal_val: IntValue<'ctx>,
        name: &str,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let matches = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            scrutinee_int,
            literal_val,
            name,
        )?;
        Ok((matches, vec![]))
    }

    /// Compile a pattern test, returning (matches: i1, bindings)
    pub fn compile_pattern_test_with_type(
        &mut self,
        scrutinee: &BasicValueEnum<'ctx>,
        pattern: &ast::Pattern,
        _scrutinee_type: Option<&AstType>,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let true_val = || self.context.bool_type().const_int(1, false);

        match pattern {
            ast::Pattern::Literal(expr) => self.compile_literal_pattern(scrutinee, expr),
            ast::Pattern::Wildcard => Ok((true_val(), vec![])),
            ast::Pattern::Identifier(name) => Ok((true_val(), vec![(name.clone(), *scrutinee)])),
            ast::Pattern::EnumLiteral { variant, payload } => {
                self.compile_enum_pattern(scrutinee, variant, payload)
            }
            ast::Pattern::Tuple(patterns) => {
                let mut all_bindings = vec![];
                for pattern in patterns {
                    let (_, bindings) =
                        self.compile_pattern_test_with_type(scrutinee, pattern, None)?;
                    all_bindings.extend(bindings);
                }
                Ok((true_val(), all_bindings))
            }
            _ => Err(CompileError::UnsupportedFeature(
                format!("Pattern type not yet implemented: {:?}", pattern),
                self.current_span.clone(),
            )),
        }
    }

    /// Compile literal pattern matching
    fn compile_literal_pattern(
        &self,
        scrutinee: &BasicValueEnum<'ctx>,
        expr: &ast::Expression,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let false_val = self.context.bool_type().const_int(0, false);

        let BasicValueEnum::IntValue(scrutinee_int) = scrutinee else {
            return Ok((false_val, vec![]));
        };

        match expr {
            ast::Expression::Boolean(b) => {
                let bit_width = scrutinee_int.get_type().get_bit_width();
                let literal_val = if bit_width == 1 {
                    self.context.bool_type().const_int(*b as u64, false)
                } else {
                    scrutinee_int.get_type().const_int(*b as u64, false)
                };
                self.compare_int_literal(*scrutinee_int, literal_val, "bool_match")
            }
            ast::Expression::Integer32(n) => {
                let literal_val = self.context.i32_type().const_int(*n as u64, false);
                self.compare_int_literal(*scrutinee_int, literal_val, "i32_match")
            }
            ast::Expression::Integer64(n) => {
                let literal_val = self.context.i64_type().const_int(*n as u64, false);
                self.compare_int_literal(*scrutinee_int, literal_val, "i64_match")
            }
            _ => Err(CompileError::UnsupportedFeature(
                format!("Unsupported literal pattern type: {:?}", expr),
                self.current_span.clone(),
            )),
        }
    }

    /// Compile enum pattern matching
    fn compile_enum_pattern(
        &mut self,
        scrutinee: &BasicValueEnum<'ctx>,
        variant: &str,
        payload: &Option<Box<ast::Pattern>>,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        match scrutinee {
            BasicValueEnum::PointerValue(ptr) => {
                self.compile_enum_pattern_ptr(*ptr, variant, payload)
            }
            BasicValueEnum::IntValue(int_val) => self.compile_enum_pattern_int(*int_val, variant),
            BasicValueEnum::StructValue(struct_val) => {
                self.compile_enum_pattern_struct(*struct_val, variant, payload)
            }
            _ => Err(CompileError::UnsupportedFeature(
                format!(
                    "Unsupported scrutinee type for enum pattern: {:?}",
                    scrutinee
                ),
                self.current_span.clone(),
            )),
        }
    }

    /// Compile enum pattern for pointer scrutinee
    fn compile_enum_pattern_ptr(
        &mut self,
        scrutinee_ptr: PointerValue<'ctx>,
        variant: &str,
        payload: &Option<Box<ast::Pattern>>,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let Some((tag, Some(struct_type))) = self.find_enum_variant_tag(variant) else {
            return Err(CompileError::UnsupportedFeature(
                format!("Enum variant '{}' not found in symbol table", variant),
                self.current_span.clone(),
            ));
        };

        // Load enum tag using centralized helper
        let tag_val = self.load_enum_tag(struct_type, scrutinee_ptr, "scrutinee")?;
        let matches = self.compare_with_tag(tag_val, tag)?;

        let bindings = if payload.is_some() {
            let payload_ptr =
                self.builder
                    .build_struct_gep(struct_type, scrutinee_ptr, 1, "payload_ptr_ptr")?;
            Self::extract_payload_binding(payload, "__enum_payload_gep__", payload_ptr.into())
        } else {
            vec![]
        };

        Ok((matches, bindings))
    }

    /// Compile enum pattern for integer scrutinee (simple enum)
    fn compile_enum_pattern_int(
        &self,
        scrutinee_int: IntValue<'ctx>,
        variant: &str,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let Some((tag, _)) = self.find_enum_variant_tag(variant) else {
            return Err(CompileError::UnsupportedFeature(
                format!("Enum variant '{}' not found in symbol table", variant),
                self.current_span.clone(),
            ));
        };
        let matches = self.compare_with_tag(scrutinee_int, tag)?;
        Ok((matches, vec![]))
    }

    /// Compile enum pattern for struct value scrutinee
    fn compile_enum_pattern_struct(
        &self,
        scrutinee_struct: StructValue<'ctx>,
        variant: &str,
        payload: &Option<Box<ast::Pattern>>,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let Some((tag, _)) = self.find_enum_variant_tag(variant) else {
            return Err(CompileError::UnsupportedFeature(
                format!("Enum variant '{}' not found in symbol table", variant),
                self.current_span.clone(),
            ));
        };

        let tag_val = self
            .builder
            .build_extract_value(scrutinee_struct, 0, "tag_val")?;
        let matches = self.compare_with_tag(tag_val.into_int_value(), tag)?;

        let bindings = if payload.is_some() {
            let payload_ptr =
                self.builder
                    .build_extract_value(scrutinee_struct, 1, "payload_ptr")?;
            Self::extract_payload_binding(payload, "__enum_payload_ptr__", payload_ptr)
        } else {
            vec![]
        };

        Ok((matches, bindings))
    }

    // ============================================================================
    // PATTERN BINDING APPLICATION
    // ============================================================================

    /// Get payload AST type from tracked generic types
    fn get_payload_ast_type(&self) -> Option<AstType> {
        self.generic_type_context
            .get("Option_Some_Type")
            .or_else(|| self.generic_type_context.get("Result_Ok_Type"))
            .or_else(|| self.generic_type_context.get("Result_Err_Type"))
            .cloned()
    }

    /// Load a value from pointer with fallback to i64
    fn load_with_type_or_i64(
        &self,
        ptr: PointerValue<'ctx>,
        llvm_type: Option<inkwell::types::BasicTypeEnum<'ctx>>,
    ) -> Option<BasicValueEnum<'ctx>> {
        if let Some(ty) = llvm_type {
            self.builder.build_load(ty, ptr, "payload_val").ok()
        } else {
            self.builder
                .build_load(self.context.i64_type(), ptr, "payload_val")
                .ok()
        }
    }

    /// Convert a pointer value to an integer type by using ptrtoint and truncation.
    /// This is used for extracting payload values from enum fields.
    fn ptr_to_int_type(
        &self,
        ptr: PointerValue<'ctx>,
        target_type: inkwell::types::IntType<'ctx>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let int_val = self
            .builder
            .build_ptr_to_int(ptr, self.ptr_sized_int_type(), "ptr_to_int")
            .ok()?;
        // Only truncate if target is smaller than ptr-sized int
        if target_type.get_bit_width() < self.ptr_sized_int_type().get_bit_width() {
            self.builder
                .build_int_truncate(int_val, target_type, "trunc")
                .ok()
                .map(|v| v.into())
        } else if target_type.get_bit_width() == self.ptr_sized_int_type().get_bit_width() {
            Some(int_val.into())
        } else {
            // Extend if target is larger (shouldn't happen for typical uses)
            self.builder
                .build_int_z_extend(int_val, target_type, "zext")
                .ok()
                .map(|v| v.into())
        }
    }

    /// Convert a pointer value to a float type via ptrtoint then bitcast.
    fn ptr_to_float_type(
        &self,
        ptr: PointerValue<'ctx>,
        target_type: inkwell::types::FloatType<'ctx>,
    ) -> Option<BasicValueEnum<'ctx>> {
        // Determine the integer type matching the float's bit width
        let int_type = if target_type == self.context.f32_type() {
            self.context.i32_type()
        } else {
            self.context.i64_type()
        };

        let int_val = self
            .builder
            .build_ptr_to_int(ptr, self.ptr_sized_int_type(), "ptr_to_int")
            .ok()?;

        // Truncate to match float size if needed
        let sized_int = if int_type.get_bit_width() < self.ptr_sized_int_type().get_bit_width() {
            self.builder
                .build_int_truncate(int_val, int_type, "trunc")
                .ok()?
        } else {
            int_val
        };

        self.builder
            .build_bit_cast(sized_int, target_type, "int_to_float")
            .ok()
    }

    /// Store a binding as a variable
    fn store_binding(&mut self, var_name: &str, value: BasicValueEnum<'ctx>, ast_type: AstType) {
        if let Ok(alloca) = self.builder.build_alloca(value.get_type(), var_name) {
            let _ = self.builder.build_store(alloca, value);
            self.variables.insert(
                var_name.to_string(),
                VariableInfo {
                    pointer: alloca,
                    ast_type,
                    is_mutable: false,
                    is_initialized: true,
                    definition_span: self.current_span.clone(),
                },
            );
        }
    }

    /// Infer AST type from LLVM value
    fn infer_ast_type(&self, value: &BasicValueEnum<'ctx>) -> AstType {
        match value {
            BasicValueEnum::IntValue(iv) => match iv.get_type().get_bit_width() {
                1 => AstType::Bool,
                32 => AstType::I32,
                _ => AstType::I64,
            },
            BasicValueEnum::FloatValue(fv) => {
                if fv.get_type() == self.context.f32_type() {
                    AstType::F32
                } else {
                    AstType::F64
                }
            }
            BasicValueEnum::PointerValue(_) => AstType::raw_ptr(AstType::Void),
            _ => AstType::I32,
        }
    }

    /// Apply pattern bindings to the current scope
    pub fn apply_pattern_bindings(&mut self, bindings: &[(String, BasicValueEnum<'ctx>)]) {
        for (name, value) in bindings {
            // Handle enum payload bindings (GEP needs 2 loads, ptr needs 1)
            let (prefix, needs_extra_load) = if name.starts_with("__enum_payload_gep__") {
                ("__enum_payload_gep__", true)
            } else if name.starts_with("__enum_payload_ptr__") {
                ("__enum_payload_ptr__", false)
            } else {
                // Regular binding
                let ast_type = self.infer_ast_type(value);
                self.store_binding(name, *value, ast_type);
                continue;
            };

            let var_name = &name[prefix.len()..];
            if !value.is_pointer_value() {
                continue;
            }

            let payload_ptr = if needs_extra_load {
                // GEP-based: first load the pointer from struct field
                let gep_ptr = value.into_pointer_value();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                match self.builder.build_load(ptr_type, gep_ptr, "payload_ptr") {
                    Ok(loaded) => loaded.into_pointer_value(),
                    Err(_) => continue,
                }
            } else {
                value.into_pointer_value()
            };

            let payload_ast_type = self.get_payload_ast_type();

            // ================================================================
            // DIRECT VALUE EXTRACTION (no pointer dereference)
            // The payload field contains the value directly (stored via inttoptr)
            // Convert back using ptrtoint for integer types
            // ================================================================
            let payload_val: Option<BasicValueEnum<'ctx>> = match &payload_ast_type {
                // Pointer types: the payload IS the pointer, use directly
                Some(ast_type) if ast_type.is_ptr_type() => Some(payload_ptr.into()),

                // Integer types: use centralized ptr_to_int_type helper
                Some(AstType::I8 | AstType::U8) => {
                    self.ptr_to_int_type(payload_ptr, self.context.i8_type())
                }
                Some(AstType::I16 | AstType::U16) => {
                    self.ptr_to_int_type(payload_ptr, self.context.i16_type())
                }
                Some(AstType::I32 | AstType::U32) => {
                    self.ptr_to_int_type(payload_ptr, self.context.i32_type())
                }
                Some(AstType::Bool) => self.ptr_to_int_type(payload_ptr, self.context.bool_type()),

                // Float types: use centralized ptr_to_float_type helper
                Some(AstType::F32) => self.ptr_to_float_type(payload_ptr, self.context.f32_type()),
                Some(AstType::F64) => self.ptr_to_float_type(payload_ptr, self.context.f64_type()),

                // Default: convert to ptr-sized int (covers I64, U64, Usize, and unknown types)
                _ => self.ptr_to_int_type(payload_ptr, self.ptr_sized_int_type()),
            };

            if let Some(val) = payload_val {
                let ast_type = payload_ast_type.unwrap_or(AstType::I64);
                self.store_binding(var_name, val, ast_type);
            }
        }
    }
}
