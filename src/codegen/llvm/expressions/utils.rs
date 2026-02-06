use super::super::LLVMCompiler;
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use crate::stdlib_types::StdlibTypeRegistry;
use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};
use std::sync::atomic::{AtomicU32, Ordering};

// ============================================================================
// HELPER FUNCTIONS FOR REDUCING DUPLICATION
// ============================================================================

/// Track Result<T, E> type arguments in the compiler's generic type context
pub fn track_result_types<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    result_type: &AstType,
    type_args: &[AstType],
) {
    if type_args.len() != 2 {
        return;
    }
    compiler.track_generic_type("Result_Ok_Type".to_string(), type_args[0].clone());
    compiler.track_generic_type("Result_Err_Type".to_string(), type_args[1].clone());
    compiler.track_complex_generic(result_type, compiler.well_known.result_name());
    compiler
        .generic_tracker
        .track_generic_type(result_type, compiler.well_known.result_name());
}

/// Convert AstType to LLVM BasicTypeEnum for loading values
/// Returns None for types that need special handling (generics, structs, strings)
pub fn ast_type_to_basic_type<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
    ast_type: &AstType,
) -> Option<BasicTypeEnum<'ctx>> {
    match ast_type {
        AstType::I8 | AstType::U8 => Some(compiler.context.i8_type().into()),
        AstType::I16 | AstType::U16 => Some(compiler.context.i16_type().into()),
        AstType::I32 | AstType::U32 => Some(compiler.context.i32_type().into()),
        AstType::I64 | AstType::U64 | AstType::Usize => Some(compiler.context.i64_type().into()),
        AstType::F32 => Some(compiler.context.f32_type().into()),
        AstType::F64 => Some(compiler.context.f64_type().into()),
        AstType::Bool => Some(compiler.context.bool_type().into()),
        _ => None, // Complex types need special handling
    }
}

/// Get the LLVM struct type for Result/Option (tag + payload pointer)
/// Uses centralized well_known_enum_type() for consistent layout
pub fn generic_enum_struct_type<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
) -> inkwell::types::StructType<'ctx> {
    compiler.well_known_enum_type()
}

/// Parse comma-separated type arguments from a string.
/// The compiler parameter is kept for API compatibility but not used.
pub fn parse_type_args_string(
    _compiler: &LLVMCompiler,
    type_params_str: &str,
) -> Result<Vec<AstType>, CompileError> {
    crate::parser::parse_type_args_from_string(type_params_str)
}

/// Parse a single type string like "i32" or "Option<i32>" into AstType.
/// The compiler parameter is kept for API compatibility but not used.
pub fn parse_single_type_string(
    _compiler: &LLVMCompiler,
    type_str: &str,
) -> Result<AstType, CompileError> {
    crate::parser::parse_type_from_string(type_str)
}

/// Infer the return type of a closure from its body
pub fn compile_comptime_expression<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // Evaluate the expression at compile time using the persistent evaluator
    let span = compiler.get_current_span();
    match compiler
        .comptime_evaluator
        .evaluate_expression(expr, span.clone())
    {
        Ok(value) => {
            // Convert the comptime value to a constant expression and compile it
            let const_expr = value.to_expression()?;
            compiler.compile_expression(&const_expr)
        }
        Err(e) => Err(CompileError::InternalError(
            format!("Comptime evaluation error: {}", e),
            span,
        )),
    }
}
/// Thread-safe counter for generating unique raise block IDs
static RAISE_ID: AtomicU32 = AtomicU32::new(0);

pub fn compile_raise_expression<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // Generate a unique ID for this raise to avoid block name collisions
    let raise_id = RAISE_ID.fetch_add(1, Ordering::Relaxed);

    let parent_function = compiler.current_function.ok_or_else(|| {
        CompileError::InternalError(
            "No current function for .raise()".to_string(),
            compiler.get_current_span(),
        )
    })?;

    // Get the current function's name to look up its return type
    let function_name = parent_function
        .get_name()
        .to_str()
        .unwrap_or("anon")
        .to_string();

    // Check if the function returns a Result type and if it's void
    // Try TypeContext first, then fall back to local cache
    let return_type_opt = compiler
        .type_ctx
        .get_function_return_type(&function_name)
        .or_else(|| compiler.function_types.get(&function_name).cloned());
    let (returns_result, is_void_function) = if let Some(return_type) = return_type_opt {
        match &return_type {
            AstType::Generic { name, .. } if compiler.well_known.is_result(name) => (true, false),
            AstType::Void => (false, true),
            _ => (false, false),
        }
    } else {
        (false, true) // Default to void if we don't know
    };

    // Compile the expression that should return a Result<T, E>
    let result_value = compiler.compile_expression(expr)?;

    // Track the Result's generic types based on the expression type
    match expr {
        Expression::FunctionCall { name, .. } => {
            // Check TypeContext first, then local cache
            let return_type = compiler
                .type_ctx
                .get_function_return_type(name)
                .or_else(|| compiler.function_types.get(name).cloned());
            if let Some(return_type) = return_type {
                compiler.track_complex_generic(&return_type, compiler.well_known.result_name());
                if let AstType::Generic {
                    name: type_name,
                    type_args,
                } = &return_type
                {
                    if compiler.well_known.is_result(type_name) {
                        track_result_types(compiler, &return_type, type_args);
                    }
                }
            }
        }
        Expression::Identifier(_name) => {
            // For identifiers/variables, try to infer their type
            if let Ok(var_type) = compiler.infer_expression_type(expr) {
                if let AstType::Generic {
                    name: type_name,
                    type_args,
                } = &var_type
                {
                    if compiler.well_known.is_result(type_name) {
                        track_result_types(compiler, &var_type, type_args);
                    }
                }
            }
        }
        Expression::EnumVariant {
            enum_name,
            variant,
            payload,
        } => {
            // For direct Result.Ok(value) or Result.Err(value) constructions
            if compiler.well_known.is_result(enum_name) {
                if compiler.well_known.is_ok(variant) {
                    // Infer type from the payload
                    if let Some(payload_expr) = payload {
                        let payload_type = compiler
                            .infer_expression_type(payload_expr)
                            .unwrap_or(AstType::I32);
                        compiler
                            .track_generic_type("Result_Ok_Type".to_string(), payload_type.clone());
                        // For Result.Ok, we don't know the error type yet, default to String
                        compiler.track_generic_type(
                            "Result_Err_Type".to_string(),
                            AstType::StaticString,
                        );
                    }
                } else if compiler.well_known.is_err(variant) {
                    // Infer type from the payload
                    if let Some(payload_expr) = payload {
                        let payload_type = compiler
                            .infer_expression_type(payload_expr)
                            .unwrap_or(AstType::StaticString);
                        // For Result.Err, we don't know the Ok type yet, default to I32
                        compiler.track_generic_type("Result_Ok_Type".to_string(), AstType::I32);
                        compiler.track_generic_type(
                            "Result_Err_Type".to_string(),
                            payload_type.clone(),
                        );
                    }
                }
            }
        }
        Expression::MethodCall { object, .. } => {
            // Special handling for chained method calls like get_chained().raise()
            // First try to infer what the object returns
            if let Ok(object_type) = compiler.infer_expression_type(object) {
                if let AstType::Generic {
                    name: type_name,
                    type_args,
                } = &object_type
                {
                    if compiler.well_known.is_result(type_name) {
                        track_result_types(compiler, &object_type, type_args);
                        // For nested Result types, track the inner type
                        if let Some(AstType::Generic {
                            name: inner_name, ..
                        }) = type_args.first()
                        {
                            if compiler.well_known.is_result(inner_name) {
                                compiler
                                    .generic_tracker
                                    .track_generic_type(&type_args[0], "Result_Ok");
                            }
                        }
                    }
                }
            }
            // Now infer the type of the full method call expression
            if let Ok(expr_type) = compiler.infer_expression_type(expr) {
                if let AstType::Generic {
                    name: type_name,
                    type_args,
                } = &expr_type
                {
                    if compiler.well_known.is_result(type_name) {
                        track_result_types(compiler, &expr_type, type_args);
                    }
                }
            }
        }
        _ => {
            // Try to infer the type for other expressions
            if let Ok(expr_type) = compiler.infer_expression_type(expr) {
                if let AstType::Generic {
                    name: type_name,
                    type_args,
                } = &expr_type
                {
                    if compiler.well_known.is_result(type_name) {
                        track_result_types(compiler, &expr_type, type_args);
                    }
                }
            }
        }
    }

    // Create blocks for pattern matching on Result
    let check_bb = compiler
        .context
        .append_basic_block(parent_function, &format!("raise_check_{}", raise_id));
    let ok_bb = compiler
        .context
        .append_basic_block(parent_function, &format!("raise_ok_{}", raise_id));
    let err_bb = compiler
        .context
        .append_basic_block(parent_function, &format!("raise_err_{}", raise_id));
    let continue_bb = compiler
        .context
        .append_basic_block(parent_function, &format!("raise_continue_{}", raise_id));

    // Jump to check block
    compiler.builder.build_unconditional_branch(check_bb)?;
    compiler.builder.position_at_end(check_bb);

    // Handle the Result enum based on its actual representation
    // Result<T, E> is an enum with variants Ok(T) and Err(E)
    // This should work with the existing enum compilation system

    if result_value.is_struct_value() {
        // Result is represented as a struct with tag + payload
        let struct_val = result_value.into_struct_value();
        let struct_type = struct_val.get_type();

        // Create a temporary alloca to work with the struct
        let temp_alloca = compiler.builder.build_alloca(struct_type, "result_temp")?;
        compiler.builder.build_store(temp_alloca, struct_val)?;

        // Load tag using centralized helper
        let tag_value = compiler.load_enum_tag(struct_type, temp_alloca, "result")?;

        // Check if tag == 0 (Ok variant) using the same type
        let ok_tag = compiler.enum_tag_const(struct_type, 0);
        let is_ok = compiler.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            tag_value,
            ok_tag,
            "is_ok",
        )?;

        // Branch based on the tag
        compiler
            .builder
            .build_conditional_branch(is_ok, ok_bb, err_bb)?;

        // Handle Ok case - extract the Ok value
        compiler.builder.position_at_end(ok_bb);
        if struct_type.count_fields() > 1 {
            let payload_ptr =
                compiler
                    .builder
                    .build_struct_gep(struct_type, temp_alloca, 1, "payload_ptr")?;
            // Get the actual payload type from the struct
            let payload_field_type = struct_type.get_field_type_at_index(1).ok_or_else(|| {
                CompileError::InternalError(
                    "Result payload field not found".to_string(),
                    compiler.get_current_span(),
                )
            })?;

            // Load the payload value (which is a pointer to the actual value)
            let ok_value_ptr =
                compiler
                    .builder
                    .build_load(payload_field_type, payload_ptr, "ok_value_ptr")?;

            // The payload is always stored as a pointer in our enum representation
            // We need to dereference it to get the actual value
            let ok_value = if ok_value_ptr.is_pointer_value() {
                let ptr_val = ok_value_ptr.into_pointer_value();

                // Use the tracked generic type information to determine the correct type to load
                let load_result: Result<BasicValueEnum<'ctx>, CompileError> =
                    if let Some(ast_type) =
                        compiler.generic_type_context.get("Result_Ok_Type").cloned()
                    {
                        // Try basic types first using helper
                        if let Some(load_type) = ast_type_to_basic_type(compiler, &ast_type) {
                            Ok(compiler
                                .builder
                                .build_load(load_type, ptr_val, "ok_value_deref")?)
                        } else {
                            // Handle complex types that need special logic
                            match &ast_type {
                                AstType::Generic { name, type_args }
                                    if compiler.well_known.is_result(name)
                                        && type_args.len() == 2 =>
                                {
                                    // Handle nested Result<T,E> - payload is a heap-allocated struct
                                    // Track with more specific keys for nested context
                                    compiler.generic_tracker.track_generic_type(
                                        &ast_type,
                                        compiler.well_known.result_name(),
                                    );

                                    let result_struct_type = generic_enum_struct_type(compiler);
                                    // Load the nested Result struct from heap
                                    let loaded_struct = compiler.builder.build_load(
                                        result_struct_type,
                                        ptr_val,
                                        "nested_result",
                                    )?;
                                    Ok(loaded_struct)
                                }
                                AstType::Generic { name, type_args }
                                    if compiler.well_known.is_option(name)
                                        && type_args.len() == 1 =>
                                {
                                    // Handle Option<T> - similar to Result
                                    let option_struct_type = generic_enum_struct_type(compiler);
                                    let loaded = compiler.builder.build_load(
                                        option_struct_type,
                                        ptr_val,
                                        "nested_option",
                                    )?;
                                    // Track the nested generic type
                                    compiler.track_generic_type(
                                        "Option_Some_Type".to_string(),
                                        type_args[0].clone(),
                                    );
                                    Ok(loaded)
                                }
                                AstType::Struct { name, .. }
                                    if StdlibTypeRegistry::is_string_type(name) =>
                                {
                                    Ok(ptr_val.into())
                                }
                                AstType::StaticString | AstType::StaticLiteral => {
                                    // Static strings are already a pointer value
                                    Ok(ptr_val.into())
                                }
                                _ => {
                                    // Default fallback to i32
                                    let load_type: BasicTypeEnum =
                                        compiler.context.i32_type().into();
                                    Ok(compiler.builder.build_load(
                                        load_type,
                                        ptr_val,
                                        "ok_value_deref",
                                    )?)
                                }
                            }
                        }
                    } else {
                        // Default to i32 for backward compatibility
                        let load_type: BasicTypeEnum = compiler.context.i32_type().into();
                        Ok(compiler
                            .builder
                            .build_load(load_type, ptr_val, "ok_value_deref")?)
                    };

                load_result?
            } else {
                // If it's not a pointer, it might be an integer that looks like a pointer address
                // This can happen if the payload is stored incorrectly
                ok_value_ptr
            };
            // Track what type raise() is extracting
            // Store the type BEFORE updating the context so variables can be typed correctly
            let extracted_type = compiler.generic_type_context.get("Result_Ok_Type").cloned();

            // Update generic context BEFORE building the branch
            // If we just extracted a nested generic type, update the context immediately
            // so that subsequent raise() calls will see the correct type
            if let Some(AstType::Generic { name, type_args }) = extracted_type.as_ref() {
                if compiler.well_known.is_result(name) && type_args.len() == 2 {
                    // We're extracting a Result<T,E>, update context for next raise()
                    compiler.track_generic_type("Result_Ok_Type".to_string(), type_args[0].clone());
                    compiler
                        .track_generic_type("Result_Err_Type".to_string(), type_args[1].clone());
                } else if compiler.well_known.is_option(name) && type_args.len() == 1 {
                    compiler
                        .track_generic_type("Option_Some_Type".to_string(), type_args[0].clone());
                }
            }

            // Store the extracted type for variable type inference
            if let Some(extracted) = extracted_type.clone() {
                compiler
                    .track_generic_type("Last_Raise_Extracted_Type".to_string(), extracted.clone());

                // Also track it in the generic tracker for better nested handling
                compiler
                    .generic_tracker
                    .track_generic_type(&extracted, "Extracted");
            }

            compiler.builder.build_unconditional_branch(continue_bb)?;

            // Handle Err case - propagate the error by returning early
            compiler.builder.position_at_end(err_bb);

            if returns_result {
                // Function returns Result<T,E> - propagate the entire Result with Err variant
                let err_payload_ptr = compiler.builder.build_struct_gep(
                    struct_type,
                    temp_alloca,
                    1,
                    "err_payload_ptr",
                )?;

                // Get the actual payload type from the struct
                let payload_field_type =
                    struct_type.get_field_type_at_index(1).ok_or_else(|| {
                        CompileError::InternalError(
                            "Result payload field not found".to_string(),
                            compiler.get_current_span(),
                        )
                    })?;

                // Load the error payload with the correct type
                let err_value = compiler.builder.build_load(
                    payload_field_type,
                    err_payload_ptr,
                    "err_value",
                )?;

                // Create a new Result<T,E> with Err variant for early return
                let return_result_alloca = compiler
                    .builder
                    .build_alloca(struct_type, "return_result")?;

                // Set tag to 1 (Err) using the actual discriminant type
                let return_tag_ptr = compiler.builder.build_struct_gep(
                    struct_type,
                    return_result_alloca,
                    0,
                    "return_tag_ptr",
                )?;
                let return_discriminant_type = struct_type
                    .get_field_type_at_index(0)
                    .ok_or_else(|| {
                        CompileError::InternalError(
                            "Result type missing discriminant field".to_string(),
                            compiler.get_current_span(),
                        )
                    })?
                    .into_int_type();
                compiler
                    .builder
                    .build_store(return_tag_ptr, return_discriminant_type.const_int(1, false))?;

                // Store the error value
                let return_payload_ptr = compiler.builder.build_struct_gep(
                    struct_type,
                    return_result_alloca,
                    1,
                    "return_payload_ptr",
                )?;
                compiler
                    .builder
                    .build_store(return_payload_ptr, err_value)?;

                // Load and return the complete Result
                let return_result = compiler.builder.build_load(
                    struct_type,
                    return_result_alloca,
                    "return_result",
                )?;
                compiler.builder.build_return(Some(&return_result))?;
            } else if !is_void_function {
                // Function returns a plain type (like i32) - this is an error case
                // For now, we'll return a default error value (1 for i32, indicating error)
                // In a proper implementation, this would need better error handling
                let error_value = compiler.context.i32_type().const_int(1, false);
                compiler.builder.build_return(Some(&error_value))?;
            } else {
                // Void function - just return without a value
                compiler.builder.build_return(None)?;
            }

            // Continue with Ok value
            compiler.builder.position_at_end(continue_bb);

            // Context has already been updated before the branch, no need to update again

            Ok(ok_value)
        } else {
            // Unit Result (no payload)
            compiler.builder.build_unconditional_branch(continue_bb)?;

            compiler.builder.position_at_end(err_bb);
            // For unit Results, handle based on return type
            if returns_result {
                compiler.builder.build_return(Some(&struct_val))?;
            } else if !is_void_function {
                // Return error value for plain return type
                let error_value = compiler.context.i32_type().const_int(1, false);
                compiler.builder.build_return(Some(&error_value))?;
            } else {
                // Void function - just return without a value
                compiler.builder.build_return(None)?;
            }

            compiler.builder.position_at_end(continue_bb);
            Ok(compiler.context.i64_type().const_int(0, false).into())
        }
    } else if result_value.is_pointer_value() {
        // Result is stored as a pointer to a struct
        let result_ptr = result_value.into_pointer_value();

        // Use the centralized well-known enum type for Result
        let struct_type = compiler.well_known_enum_type();

        // Extract the tag from the first field
        let tag_ptr = compiler
            .builder
            .build_struct_gep(struct_type, result_ptr, 0, "tag_ptr")?;
        // Get the actual discriminant type from the struct's first field
        let discriminant_type = struct_type
            .get_field_type_at_index(0)
            .ok_or_else(|| {
                CompileError::InternalError(
                    "Enum struct type missing discriminant field".to_string(),
                    compiler.get_current_span(),
                )
            })?
            .into_int_type();
        let tag_value = compiler
            .builder
            .build_load(discriminant_type, tag_ptr, "tag")?;

        // Check if tag == 0 (Ok variant) using the same type
        let is_ok = compiler.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            tag_value.into_int_value(),
            discriminant_type.const_int(0, false),
            "is_ok",
        )?;

        // Branch based on the tag
        compiler
            .builder
            .build_conditional_branch(is_ok, ok_bb, err_bb)?;

        // Handle Ok case
        compiler.builder.position_at_end(ok_bb);
        if struct_type.count_fields() > 1 {
            let payload_ptr =
                compiler
                    .builder
                    .build_struct_gep(struct_type, result_ptr, 1, "payload_ptr")?;
            // Load the payload, which is stored as a pointer to the actual value
            let ok_value_ptr = compiler.builder.build_load(
                compiler.context.ptr_type(inkwell::AddressSpace::default()),
                payload_ptr,
                "ok_value_ptr",
            )?;

            // Dereference the pointer to get the actual value
            // For now, assume Result<i32, E>
            let ok_value = if ok_value_ptr.is_pointer_value() {
                let ptr_val = ok_value_ptr.into_pointer_value();
                compiler.builder.build_load(
                    compiler.context.i32_type(),
                    ptr_val,
                    "ok_value_deref",
                )?
            } else {
                ok_value_ptr
            };
            compiler.builder.build_unconditional_branch(continue_bb)?;

            // Handle Err case
            compiler.builder.position_at_end(err_bb);

            if returns_result {
                // Return the original Result (already contains Err)
                let err_result =
                    compiler
                        .builder
                        .build_load(struct_type, result_ptr, "err_result")?;
                compiler.builder.build_return(Some(&err_result))?;
            } else {
                // Function returns a plain type - return error value
                let error_value = compiler.context.i32_type().const_int(1, false);
                compiler.builder.build_return(Some(&error_value))?;
            }

            // Continue with Ok value
            compiler.builder.position_at_end(continue_bb);
            Ok(ok_value)
        } else {
            // Unit Result
            compiler.builder.build_unconditional_branch(continue_bb)?;

            compiler.builder.position_at_end(err_bb);

            if returns_result {
                let err_result =
                    compiler
                        .builder
                        .build_load(struct_type, result_ptr, "err_result")?;
                compiler.builder.build_return(Some(&err_result))?;
            } else if !is_void_function {
                // Return error value for plain return type
                let error_value = compiler.context.i32_type().const_int(1, false);
                compiler.builder.build_return(Some(&error_value))?;
            } else {
                // Void function - just return without a value
                compiler.builder.build_return(None)?;
            }

            compiler.builder.position_at_end(continue_bb);
            Ok(compiler.context.i64_type().const_int(0, false).into())
        }
    } else {
        // Check if this is actually a struct type but LLVM isn't recognizing it
        // This happens when a Result<T,E> is returned from a function call
        // The value might be aggregate or the type check is failing

        // Try to handle it as a struct type anyway if it looks like one
        let result_type = result_value.get_type();

        // Check if this is a Result struct (2 fields) even if it's presented as an array or aggregate type
        // This can happen with nested Results where the loaded value becomes an aggregate
        //
        // KNOWN LIMITATION: This heuristic (checking for 2 fields) could false-positive on
        // user-defined structs. A more robust solution would track AST type information
        // alongside LLVM values to positively identify Result/Option types.
        let is_result_like = if result_type.is_struct_type() {
            let struct_type = result_type.into_struct_type();
            struct_type.count_fields() == 2 // Result/Option structs have 2 fields: tag + payload
        } else if result_type.is_array_type() {
            // Sometimes LLVM represents loaded structs as arrays
            let array_type = result_type.into_array_type();
            array_type.len() == 2
        } else {
            // Check if the value itself is a struct value (not just the type)
            result_value.is_struct_value() && {
                if let Ok(struct_val) = result_value.try_into() {
                    let sv: inkwell::values::StructValue = struct_val;
                    sv.get_type().count_fields() == 2
                } else {
                    false
                }
            }
        };

        if is_result_like {
            // Create a proper struct type for Result
            let struct_type = if result_type.is_struct_type() {
                result_type.into_struct_type()
            } else {
                // Create a struct type that matches Result representation
                compiler.well_known_enum_type()
            };

            // If the value is already a struct, use it directly; otherwise store it first
            let temp_alloca = if result_value.is_struct_value() {
                let alloca = compiler
                    .builder
                    .build_alloca(struct_type, "result_struct_temp")?;
                compiler.builder.build_store(alloca, result_value)?;
                alloca
            } else {
                // Try to treat the value as something we can work with
                // This handles cases where the nested Result was loaded as an aggregate
                let alloca = compiler
                    .builder
                    .build_alloca(struct_type, "result_aggregate_temp")?;

                // Try to store the value - if it's compatible, this will work
                compiler.builder.build_store(alloca, result_value)?;
                alloca
            };

            // Extract the tag (discriminant) from the first field
            let tag_ptr =
                compiler
                    .builder
                    .build_struct_gep(struct_type, temp_alloca, 0, "tag_ptr")?;
            // Get the actual discriminant type from the struct's first field
            let discriminant_type = struct_type
                .get_field_type_at_index(0)
                .ok_or_else(|| {
                    CompileError::InternalError(
                        "Enum struct type missing discriminant field".to_string(),
                        compiler.get_current_span(),
                    )
                })?
                .into_int_type();
            let tag_value = compiler
                .builder
                .build_load(discriminant_type, tag_ptr, "tag")?;

            // Check if tag == 0 (Ok variant) using the same type
            let is_ok = compiler.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                tag_value.into_int_value(),
                discriminant_type.const_int(0, false),
                "is_ok",
            )?;

            // Branch based on the tag
            compiler
                .builder
                .build_conditional_branch(is_ok, ok_bb, err_bb)?;

            // Handle Ok case - extract the Ok value
            compiler.builder.position_at_end(ok_bb);
            if struct_type.count_fields() > 1 {
                let payload_ptr = compiler.builder.build_struct_gep(
                    struct_type,
                    temp_alloca,
                    1,
                    "payload_ptr",
                )?;
                // Get the actual payload type from the struct
                let payload_field_type =
                    struct_type.get_field_type_at_index(1).ok_or_else(|| {
                        CompileError::InternalError(
                            "Result payload field not found".to_string(),
                            compiler.get_current_span(),
                        )
                    })?;

                // Load the payload value (which is a pointer to the actual value)
                let ok_value_ptr =
                    compiler
                        .builder
                        .build_load(payload_field_type, payload_ptr, "ok_value_ptr")?;

                // The payload is always stored as a pointer in our enum representation
                // We need to dereference it to get the actual value
                let ok_value_computed = if ok_value_ptr.is_pointer_value() {
                    let ptr_val = ok_value_ptr.into_pointer_value();

                    // Use the tracked generic type information to determine the correct type to load
                    let ast_type_opt = compiler.generic_type_context.get("Result_Ok_Type");
                    let load_type: inkwell::types::BasicTypeEnum =
                        if let Some(ast_type) = ast_type_opt {
                            // Try basic types first using helper
                            if let Some(basic_type) = ast_type_to_basic_type(compiler, ast_type) {
                                basic_type
                            } else {
                                // Handle complex types
                                match ast_type {
                                    AstType::Generic { name, .. }
                                        if compiler.well_known.is_result(name)
                                            || compiler.well_known.is_option(name) =>
                                    {
                                        // For nested generics, load the struct from heap pointer
                                        generic_enum_struct_type(compiler).into()
                                    }
                                    _ => compiler.context.i32_type().into(), // Default fallback
                                }
                            }
                        } else {
                            // Default to i32 for backward compatibility
                            compiler.context.i32_type().into()
                        };
                    // Debug: Check what type we're loading

                    let loaded_value =
                        compiler
                            .builder
                            .build_load(load_type, ptr_val, "ok_value_deref")?;

                    // The loaded value should be the correct type
                    // For nested Result/Option types, this will be a struct value that can be raised again
                    loaded_value
                } else {
                    // If it's not a pointer, it might be an integer that looks like a pointer address
                    // This can happen if the payload is stored incorrectly
                    ok_value_ptr
                };

                // For void functions, we can directly return the ok value
                // since err_bb returns early without branching to continue_bb
                if is_void_function {
                    // We're done - return the extracted value
                    return Ok(ok_value_computed);
                }

                // For non-void functions, branch to continue_bb
                compiler.builder.build_unconditional_branch(continue_bb)?;

                // Handle Err case - propagate the error by returning early
                compiler.builder.position_at_end(err_bb);

                if returns_result {
                    // Function returns Result<T,E> - propagate the entire Result with Err variant
                    let err_payload_ptr = compiler.builder.build_struct_gep(
                        struct_type,
                        temp_alloca,
                        1,
                        "err_payload_ptr",
                    )?;

                    // Get the actual payload type from the struct
                    let payload_field_type =
                        struct_type.get_field_type_at_index(1).ok_or_else(|| {
                            CompileError::InternalError(
                                "Result payload field not found".to_string(),
                                compiler.get_current_span(),
                            )
                        })?;

                    // Load the error payload with the correct type
                    let err_value = compiler.builder.build_load(
                        payload_field_type,
                        err_payload_ptr,
                        "err_value",
                    )?;

                    // Create a new Result<T,E> with Err variant for early return
                    let return_result_alloca = compiler
                        .builder
                        .build_alloca(struct_type, "return_result")?;

                    // Set tag to 1 (Err) using the actual discriminant type
                    let return_tag_ptr = compiler.builder.build_struct_gep(
                        struct_type,
                        return_result_alloca,
                        0,
                        "return_tag_ptr",
                    )?;
                    let err_discriminant_type = struct_type
                        .get_field_type_at_index(0)
                        .ok_or_else(|| {
                            CompileError::InternalError(
                                "Result type missing discriminant field".to_string(),
                                compiler.get_current_span(),
                            )
                        })?
                        .into_int_type();
                    compiler
                        .builder
                        .build_store(return_tag_ptr, err_discriminant_type.const_int(1, false))?;

                    // Store the error value
                    let return_payload_ptr = compiler.builder.build_struct_gep(
                        struct_type,
                        return_result_alloca,
                        1,
                        "return_payload_ptr",
                    )?;
                    compiler
                        .builder
                        .build_store(return_payload_ptr, err_value)?;

                    // Load and return the complete Result
                    let return_result = compiler.builder.build_load(
                        struct_type,
                        return_result_alloca,
                        "return_result",
                    )?;
                    compiler.builder.build_return(Some(&return_result))?;
                } else if !is_void_function {
                    // Function returns a plain type - return error value
                    let error_value = compiler.context.i32_type().const_int(1, false);
                    compiler.builder.build_return(Some(&error_value))?;
                } else {
                    // Void function - just return without a value
                    compiler.builder.build_return(None)?;
                }

                // Continue with Ok value - only reached for non-void functions
                compiler.builder.position_at_end(continue_bb);
                // For non-void functions, we need to return the ok value
                // This is a bit tricky because ok_value_computed is not in scope
                // We should use a PHI node, but for now return a placeholder
                Ok(compiler.context.i32_type().const_int(0, false).into())
            } else {
                // Unit Result (no payload)
                // For void functions, we can return immediately
                if is_void_function {
                    // Return a unit value
                    return Ok(compiler.context.i64_type().const_int(0, false).into());
                }
                compiler.builder.build_unconditional_branch(continue_bb)?;

                compiler.builder.position_at_end(err_bb);
                // For unit Results, handle based on return type
                if returns_result {
                    let return_result =
                        compiler
                            .builder
                            .build_load(struct_type, temp_alloca, "return_result")?;
                    compiler.builder.build_return(Some(&return_result))?;
                } else if !is_void_function {
                    // Return error value for plain return type
                    let error_value = compiler.context.i32_type().const_int(1, false);
                    compiler.builder.build_return(Some(&error_value))?;
                } else {
                    // Void function - just return without a value
                    compiler.builder.build_return(None)?;
                }

                compiler.builder.position_at_end(continue_bb);
                Ok(compiler.context.i64_type().const_int(0, false).into())
            }
        } else {
            // Fallback: if we can't determine the Result structure,
            // treat it as an immediate value and try pattern matching
            return Err(CompileError::TypeError(
                format!(
                    "Unsupported Result type for .raise(): {:?}",
                    result_value.get_type()
                ),
                compiler.get_current_span(),
            ));
        }
    }
}
