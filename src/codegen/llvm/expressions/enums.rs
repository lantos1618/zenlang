//! Enum variant compilation with type thinning for pointer types
//!
//! This module handles compilation of enum variants (e.g., `Result.Ok(value)`, `Option.Some(x)`).
//!
//! ## Type Thinning for Pointer Types
//!
//! Pointer wrapper types (Ptr<T>, MutPtr<T>, RawPtr<T>) use null-pointer optimization:
//! - `Ptr.Some(addr)` compiles to just `addr` (8 bytes)
//! - `Ptr.None` compiles to null (0)
//! - No struct wrapper, no heap allocation
//!
//! ## Direct Storage for Small Payloads
//!
//! For regular enums (Option, Result, etc.), payloads ≤ 8 bytes are stored directly
//! in the payload field using inttoptr conversion. No heap allocation needed.
//!
//! - `Option.Some(42)` → `{ tag: 0, payload: 42-as-ptr }`
//! - `Result.Ok(true)` → `{ tag: 0, payload: 1-as-ptr }`
//!
//! ## Explicit Allocation for Large Payloads
//!
//! Payloads > 8 bytes (like nested enums) require explicit heap allocation by the user:
//! - Use `Ptr<T>` to wrap large values
//! - Allocator-aware: user controls memory management
//! - No hidden malloc in compiler

use super::super::symbols;
use super::super::LLVMCompiler;
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

pub fn compile_enum_variant<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    enum_name: &str,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // ========================================================================
    // THIN POINTER TYPES: Ptr<T>, MutPtr<T>, RawPtr<T>
    // These compile to just 8 bytes (the address), with null = None variant
    // ========================================================================
    if compiler.well_known.is_ptr(enum_name) {
        return compile_thin_pointer_variant(compiler, enum_name, variant, payload);
    }

    // Save the current generic context before potentially overwriting it with nested compilation
    let _saved_ok_type = compiler.generic_type_context.get("Result_Ok_Type").cloned();
    let _saved_err_type = compiler
        .generic_type_context
        .get("Result_Err_Type")
        .cloned();

    // Track Result<T, E> type information when compiling Result variants
    // This happens BEFORE we compile the payload, so we know what type this Result should be
    if compiler.well_known.is_result(enum_name) && payload.is_some() {
        if let Some(ref payload_expr) = payload {
            let payload_type = super::inference::infer_expression_type(compiler, payload_expr);
            if let Ok(t) = payload_type {
                let key = if compiler.well_known.is_ok(variant) {
                    "Result_Ok_Type".to_string()
                } else {
                    "Result_Err_Type".to_string()
                };
                compiler.track_generic_type(key.clone(), t.clone());
                // Also track nested generics recursively
                if matches!(t, AstType::Generic { .. }) {
                    let prefix = if compiler.well_known.is_ok(variant) {
                        "Result_Ok"
                    } else {
                        "Result_Err"
                    };
                    compiler.track_complex_generic(&t, prefix);
                    // Use the generic tracker for better nested type handling
                    compiler.generic_tracker.track_generic_type(&t, prefix);
                }
                // Track custom enum types too
                if matches!(t, AstType::EnumType { .. }) {
                    let prefix = if compiler.well_known.is_ok(variant) {
                        "Result_Ok"
                    } else {
                        "Result_Err"
                    };
                    compiler.generic_tracker.track_generic_type(&t, prefix);
                }
            }
        }
    }

    // Track Option<T> type information when compiling Option variants
    if compiler.well_known.is_option(enum_name) && payload.is_some() {
        if let Some(ref payload_expr) = payload {
            let payload_type = super::inference::infer_expression_type(compiler, payload_expr);
            if let Ok(t) = payload_type {
                compiler.track_generic_type("Option_Some_Type".to_string(), t.clone());
                // Track nested generics recursively (Vec, DynVec, HashMap, etc. are now Generic)
                if matches!(t, AstType::Generic { .. }) {
                    compiler.track_complex_generic(&t, "Option_Some");
                }
                // Track custom enum types too
                if matches!(t, AstType::EnumType { .. }) {
                    compiler
                        .generic_tracker
                        .track_generic_type(&t, "Option_Some");
                }
            }
        }
    }

    // Helper: Find enum info and variant tag from symbol table
    let find_enum_info = |compiler: &LLVMCompiler<'ctx>,
                          enum_name: &str,
                          variant: &str|
     -> Option<(symbols::EnumInfo, u64)> {
        if !enum_name.is_empty() {
            // Try direct lookup by enum name
            if let Some(symbols::Symbol::EnumType(info)) = compiler.symbols.lookup(enum_name) {
                if let Some(&tag) = info.variant_indices.get(variant) {
                    return Some((info.clone(), tag));
                }
            }
        } else {
            // Search all enums for one containing this variant
            for symbol in compiler.symbols.all_symbols() {
                if let symbols::Symbol::EnumType(info) = symbol {
                    if let Some(&tag) = info.variant_indices.get(variant) {
                        return Some((info.clone(), tag));
                    }
                }
            }
        }
        None
    };

    // Look up the enum info from the symbol table
    let (enum_info, tag) =
        if let Some((info, tag_val)) = find_enum_info(compiler, enum_name, variant) {
            (Some(info), tag_val)
        } else {
            // If enum not found, infer enum name from variant for common types
            let inferred_enum = if compiler.well_known.is_result_variant(variant)
                || compiler.well_known.is_option_variant(variant)
            {
                compiler.well_known.get_variant_parent_name(variant)
            } else {
                None
            };

            if let Some(inferred_name) = inferred_enum {
                // Try lookup with inferred name
                if let Some((info, tag_val)) = find_enum_info(compiler, inferred_name, variant) {
                    (Some(info), tag_val)
                } else {
                    return Err(CompileError::UndeclaredVariable(
                        format!(
                            "Enum variant '{}' not found. Expected enum '{}' to be registered.",
                            variant, inferred_name
                        ),
                        None,
                    ));
                }
            } else {
                return Err(CompileError::UndeclaredVariable(
                    format!(
                        "Enum variant '{}' not found in symbol table. Cannot infer enum type.",
                        variant
                    ),
                    None,
                ));
            }
        };

    // Use the enum info from symbol table - we should always have it at this point
    let enum_info = enum_info.ok_or_else(|| {
        CompileError::InternalError(
            format!("Enum info not found for type '{}'", enum_name),
            compiler.get_current_span(),
        )
    })?;

    // Use the enum's LLVM type from the symbol table
    let enum_struct_type = enum_info.llvm_type;

    // Create alloca and store tag using centralized helper
    let alloca = compiler.builder.build_alloca(
        enum_struct_type,
        &format!("{}_{}_enum_tmp", enum_name, variant),
    )?;
    compiler.store_enum_tag(
        enum_struct_type,
        alloca,
        tag,
        &format!("{}_{}", enum_name, variant),
    )?;

    // Handle payload if present
    if let Some(expr) = payload {
        let compiled = compiler.compile_expression(expr)?;

        // Check if enum struct has payload field (some enums might not have payloads)
        if enum_struct_type.count_fields() > 1 {
            let payload_ptr =
                compiler
                    .builder
                    .build_struct_gep(enum_struct_type, alloca, 1, "payload_ptr")?;

            // ================================================================
            // DIRECT PAYLOAD STORAGE (no malloc)
            // Store values directly in the payload field using inttoptr
            // ================================================================
            let ptr_type = compiler.context.ptr_type(AddressSpace::default());

            let payload_value = if compiled.is_pointer_value() {
                // Pointer values: store directly (already the right type)
                compiled
            } else if compiled.is_int_value() {
                // Integer values: convert to pointer using inttoptr
                // This stores the value directly in the 8-byte payload field
                let int_val = compiled.into_int_value();
                let int_type = int_val.get_type();

                // Extend smaller integers to i64 first
                let i64_val = if int_type.get_bit_width() < 64 {
                    compiler.builder.build_int_z_extend(
                        int_val,
                        compiler.context.i64_type(),
                        "extend_payload",
                    )?
                } else if int_type.get_bit_width() > 64 {
                    // Truncate if somehow larger (shouldn't happen for standard types)
                    compiler.builder.build_int_truncate(
                        int_val,
                        compiler.context.i64_type(),
                        "trunc_payload",
                    )?
                } else {
                    int_val
                };

                // Convert i64 to pointer - this stores the VALUE in the pointer field
                compiler
                    .builder
                    .build_int_to_ptr(i64_val, ptr_type, "val_as_ptr")?
                    .into()
            } else if compiled.is_float_value() {
                // Float values: bitcast to i64, then inttoptr
                let float_val = compiled.into_float_value();
                let float_type = float_val.get_type();

                let i64_val = if float_type == compiler.context.f32_type() {
                    // f32 -> i32 -> i64
                    let i32_val = compiler.builder.build_bit_cast(
                        float_val,
                        compiler.context.i32_type(),
                        "f32_as_i32",
                    )?;
                    compiler.builder.build_int_z_extend(
                        i32_val.into_int_value(),
                        compiler.context.i64_type(),
                        "extend_f32",
                    )?
                } else {
                    // f64 -> i64
                    compiler
                        .builder
                        .build_bit_cast(float_val, compiler.context.i64_type(), "f64_as_i64")?
                        .into_int_value()
                };

                compiler
                    .builder
                    .build_int_to_ptr(i64_val, ptr_type, "float_as_ptr")?
                    .into()
            } else if compiled.is_struct_value() {
                // Struct values: check size
                let struct_val = compiled.into_struct_value();
                let struct_type = struct_val.get_type();
                let field_count = struct_type.count_fields();

                // Nested enums (2 fields = tag + payload) are 16 bytes - too large!
                if field_count == 2 {
                    return Err(CompileError::TypeError(
                        format!(
                            "Enum payload is too large (nested enum detected). \
                            Use Ptr<T> to wrap the inner value. Example: {}.{}(Ptr.from(inner_value))",
                            enum_name, variant
                        ),
                        compiler.get_current_span(),
                    ));
                }

                // Small structs (≤ 8 bytes) could potentially be packed, but for now error
                return Err(CompileError::TypeError(
                    format!(
                        "Struct payloads in enums must use Ptr<T>. \
                        Allocate with: ptr = Ptr.allocate(sizeof<YourStruct>()); then use {}.{}(ptr)",
                        enum_name, variant
                    ),
                    compiler.get_current_span(),
                ));
            } else {
                // Unknown value type - try to store as-is (may fail at LLVM level)
                compiled
            };

            compiler.builder.build_store(payload_ptr, payload_value)?;
        }
    } else {
        // For variants without payload (like None), store null pointer if struct has payload field
        if enum_struct_type.count_fields() > 1 {
            let ptr_type = compiler.context.ptr_type(inkwell::AddressSpace::default());
            let payload_ptr =
                compiler
                    .builder
                    .build_struct_gep(enum_struct_type, alloca, 1, "payload_ptr")?;
            let null_ptr = ptr_type.const_null();
            compiler.builder.build_store(payload_ptr, null_ptr)?;
        }
    }

    let loaded = compiler.builder.build_load(
        enum_struct_type,
        alloca,
        &format!("{}_{}_enum_val", enum_name, variant),
    )?;
    Ok(loaded)
}

pub fn compile_enum_literal<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // Delegate to the existing implementation
    if let Expression::EnumLiteral { variant, payload } = expr {
        // Try to infer the enum name from context or use Option as default
        compile_enum_variant(
            compiler,
            compiler.well_known.option_name(),
            variant,
            payload,
        )
    } else {
        Err(CompileError::InternalError(
            "Expected EnumLiteral".to_string(),
            compiler.get_current_span(),
        ))
    }
}

pub fn compile_some<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    value: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // Some(value) is Option::Some(value)
    compile_enum_variant(
        compiler,
        compiler.well_known.option_name(),
        compiler.well_known.some_name(),
        &Some(Box::new(value.clone())),
    )
}

pub fn compile_none<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // None is Option::None
    compile_enum_variant(
        compiler,
        compiler.well_known.option_name(),
        compiler.well_known.none_name(),
        &None,
    )
}

/// Compile thin pointer types (Ptr<T>, MutPtr<T>, RawPtr<T>)
/// These use null-pointer optimization: just 8 bytes, null = None
fn compile_thin_pointer_variant<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    enum_name: &str,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr_type = compiler.context.ptr_type(AddressSpace::default());

    // Check which variant we're compiling
    let is_some_variant = compiler.well_known.is_some(variant);
    let is_none_variant = compiler.well_known.is_none(variant);

    if is_none_variant || (!is_some_variant && payload.is_none()) {
        // None variant: return null pointer
        Ok(ptr_type.const_null().into())
    } else if let Some(expr) = payload {
        // Some variant: compile the payload and return it directly
        let compiled = compiler.compile_expression(expr)?;

        if compiled.is_pointer_value() {
            // Already a pointer - use directly
            Ok(compiled)
        } else if compiled.is_int_value() {
            // Integer (address) - convert to pointer
            let int_val = compiled.into_int_value();
            let int_type = int_val.get_type();

            // Extend to i64 if needed
            let i64_val = if int_type.get_bit_width() < 64 {
                compiler.builder.build_int_z_extend(
                    int_val,
                    compiler.context.i64_type(),
                    "extend_addr",
                )?
            } else if int_type.get_bit_width() > 64 {
                compiler.builder.build_int_truncate(
                    int_val,
                    compiler.context.i64_type(),
                    "trunc_addr",
                )?
            } else {
                int_val
            };

            Ok(compiler
                .builder
                .build_int_to_ptr(i64_val, ptr_type, "addr_as_ptr")?
                .into())
        } else {
            Err(CompileError::TypeError(
                format!(
                    "{}.Some expects an address (i64 or pointer), got {:?}",
                    enum_name,
                    compiled.get_type()
                ),
                compiler.get_current_span(),
            ))
        }
    } else {
        // Some variant without payload - error
        Err(CompileError::TypeError(
            format!("{}.Some requires an address payload", enum_name),
            compiler.get_current_span(),
        ))
    }
}
