use crate::codegen::llvm::LLVMCompiler;
use crate::codegen::llvm::Type;
use crate::ast::{AstType, Expression, Statement, VariableDeclarationType};
use crate::error::CompileError;
use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};

pub fn compile_expression_statement<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<(), CompileError> {
    compiler.compile_expression(expr)?;
    Ok(())
}

pub fn compile_variable_declaration<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statement: &Statement,
) -> Result<(), CompileError> {
    // Extract VariableDeclaration from statement
    let (name, type_, initializer, is_mutable, declaration_type) = match statement {
        Statement::VariableDeclaration {
            name,
            type_,
            initializer,
            is_mutable,
            declaration_type,
            ..
        } => (name, type_, initializer, is_mutable, declaration_type),
        _ => {
            return Err(CompileError::InternalError(
                "Expected VariableDeclaration statement".to_string(),
                None,
            ));
        }
    };

    // Check if this is an assignment to a forward-declared variable
    // This happens when we have: x: i32 (forward decl) then x = 10 (initialization)
    if let Some(init_expr) = initializer {
        // Check if variable already exists (forward declaration case)
        let existing_var = compiler.variables.get(name).cloned();
        if let Some(var_info) = existing_var {
            // Allow initialization of forward-declared variables with = operator
            // This works for both immutable (x: i32 then x = 10) and mutable (w:: i32 then w = 40)
            if !var_info.is_initialized
                && matches!(declaration_type, VariableDeclarationType::InferredImmutable)
            {
                // This is initialization of a forward-declared variable
                let value = compiler.compile_expression(init_expr)?;
                let alloca = var_info.pointer;
                compiler.builder.build_store(alloca, value)?;

                // Mark the variable as initialized
                if let Some(var_info) = compiler.variables.get_mut(name) {
                    var_info.is_initialized = true;
                }
                return Ok(());
            } else if var_info.is_initialized
                && var_info.is_mutable
                && matches!(declaration_type, VariableDeclarationType::InferredImmutable)
            {
                // This is a reassignment to an existing mutable variable
                // (e.g., w = 45 after w:: i32 and w = 40)
                let value = compiler.compile_expression(init_expr)?;
                let alloca = var_info.pointer;
                compiler.builder.build_store(alloca, value)?;
                // No need to update initialized flag - it's already true
                return Ok(());
            } else {
                // Variable already exists and is initialized or wrong declaration type
                return Err(CompileError::InternalError(
                    format!(
                        "Type error: Variable '{}' already declared in this scope",
                        name
                    ),
                    None,
                ));
            }
        }
    }

    // Handle type inference or explicit type
    // We may need to compile the expression for type inference, so save the value
    let mut compiled_value: Option<BasicValueEnum> = None;

    // Keep track of inferred AST type for closures
    let mut inferred_ast_type: Option<AstType> = None;

    // Save generic context before compiling to handle raise() correctly
    let _saved_ok_type = compiler.generic_type_context.get("Result_Ok_Type").cloned();

    let llvm_type = match type_ {
        Some(type_) => compiler.to_llvm_type(type_)?,
        None => {
            // Type inference - try to infer from initializer
            if let Some(init_expr) = initializer {
                // Check if the initializer is a closure BEFORE compiling it
                if let Expression::Closure {
                    params,
                    return_type,
                    body,
                } = init_expr
                {
                    // This is a closure - infer its function pointer type
                    let param_types: Vec<AstType> = params
                        .iter()
                        .map(|(_, opt_type)| opt_type.clone().unwrap_or(AstType::I32))
                        .collect();

                    // Use the explicit return type if provided, otherwise infer from body
                    let inferred_return_type = if let Some(rt) = return_type {
                        rt.clone()
                    } else {
                        // Try to infer the return type from the closure body
                        compiler
                            .infer_closure_return_type(body)
                            .unwrap_or(AstType::I32)
                    };

                    // Store the proper function pointer type
                    let func_type = AstType::FunctionPointer {
                        param_types: param_types.clone(),
                        return_type: Box::new(inferred_return_type.clone()),
                    };

                    // Track the function's return type for later lookup when it's called
                    // This is crucial for type inference when calling closures
                    compiler
                        .function_types
                        .insert(name.clone(), inferred_return_type.clone());

                    // Save this for later when we insert the variable
                    inferred_ast_type = Some(func_type);

                    // Compile the closure
                    let init_value = compiler.compile_expression(init_expr)?;
                    compiled_value = Some(init_value);

                    // For now, use simple function type - the actual closure compilation
                    // will handle the correct types internally
                    Type::Function(
                        compiler.context.i32_type().fn_type(
                            &param_types
                                .iter()
                                .map(|_| compiler.context.i32_type().into())
                                .collect::<Vec<_>>(),
                            false,
                        ),
                    )
                } else {
                    // Not a closure - try to infer the AST type first
                    // This is crucial for generic types like Result<T,E>
                    if let Ok(ast_type) = compiler.infer_expression_type(init_expr) {
                        // Save the inferred AST type for later
                        inferred_ast_type = Some(ast_type.clone());

                        // Track generic types if this is a Result, Option, Array, etc.
                        if let AstType::Generic {
                            name: type_name,
                            type_args,
                        } = &ast_type
                        {
                            if compiler.well_known.is_result(type_name) && type_args.len() == 2 {
                                compiler.track_generic_type(
                                    format!("{}_Result_Ok_Type", name),
                                    type_args[0].clone(),
                                );
                                compiler.track_generic_type(
                                    format!("{}_Result_Err_Type", name),
                                    type_args[1].clone(),
                                );
                                // Also track without variable name prefix for pattern matching
                                compiler.track_generic_type(
                                    "Result_Ok_Type".to_string(),
                                    type_args[0].clone(),
                                );
                                compiler.track_generic_type(
                                    "Result_Err_Type".to_string(),
                                    type_args[1].clone(),
                                );
                                compiler.generic_tracker.track_generic_type(&ast_type, name);
                            } else if compiler.well_known.is_option(type_name) && type_args.len() == 1 {
                                compiler.track_generic_type(
                                    format!("{}_Option_Some_Type", name),
                                    type_args[0].clone(),
                                );
                                // Also track without variable name prefix for pattern matching
                                compiler.track_generic_type(
                                    "Option_Some_Type".to_string(),
                                    type_args[0].clone(),
                                );
                                compiler.generic_tracker.track_generic_type(&ast_type, name);
                            } else if crate::type_context::is_key_value_collection(type_name) && type_args.len() >= 2 {
                                // Key-value collections
                                compiler.track_generic_type(
                                    format!("{}_{}_Key_Type", name, type_name),
                                    type_args[0].clone(),
                                );
                                compiler.track_generic_type(
                                    format!("{}_{}_Value_Type", name, type_name),
                                    type_args[1].clone(),
                                );
                                compiler.generic_tracker.track_generic_type(&ast_type, name);
                            } else if crate::type_context::is_single_element_collection(type_name) && !type_args.is_empty() {
                                // Single-element collections
                                compiler.track_generic_type(
                                    format!("{}_{}_Element_Type", name, type_name),
                                    type_args[0].clone(),
                                );
                                compiler.generic_tracker.track_generic_type(&ast_type, name);
                            }
                        }

                        // Now compile the expression
                        let init_value = compiler.compile_expression(init_expr)?;
                        compiled_value = Some(init_value);

                        // Convert the AST type to LLVM type
                        compiler.to_llvm_type(&ast_type)?
                    } else {
                        // Fall back to compiling and inferring from LLVM value
                        let init_value = compiler.compile_expression(init_expr)?;
                        // Save the compiled value to avoid recompiling
                        compiled_value = Some(init_value);

                        match init_value {
                            BasicValueEnum::IntValue(int_val) => {
                                let bit_width = int_val.get_type().get_bit_width();
                                if bit_width == 1 {
                                    // Boolean type
                                    Type::Basic(compiler.context.bool_type().into())
                                } else if bit_width <= 32 {
                                    Type::Basic(compiler.context.i32_type().into())
                                } else {
                                    Type::Basic(compiler.context.i64_type().into())
                                }
                            }
                            BasicValueEnum::FloatValue(_fv) => {
                                // Store the AST type as F64 to ensure proper loading later
                                inferred_ast_type = Some(AstType::F64);
                                // For now, assume all floats are f64
                                Type::Basic(compiler.context.f64_type().into())
                            }
                            BasicValueEnum::PointerValue(_) => {
                                // For pointers (including strings), use ptr type
                                // Store the inferred AST type as Ptr(U8) - the generic pointer type
                                inferred_ast_type = Some(AstType::ptr(AstType::U8));
                                Type::Basic(
                                    compiler
                                        .context
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .into(),
                                )
                            }
                            BasicValueEnum::StructValue(struct_val) => {
                                // For structs (including enums), use the struct type directly
                                let struct_type = struct_val.get_type();
                                Type::Struct(struct_type)
                            }
                            _ => Type::Basic(compiler.context.i64_type().into()), // Default to i64
                        }
                    }
                }
            } else {
                return Err(CompileError::TypeError(
                    "Cannot infer type without initializer".to_string(),
                    None,
                ));
            }
        }
    };

    // Extract BasicTypeEnum for build_alloca
    let basic_type = match llvm_type {
        Type::Basic(basic) => basic,
        Type::Struct(struct_type) => struct_type.into(),
        Type::Function(_) => compiler
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .into(),
        Type::Pointer(_) => compiler
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .into(),
        Type::Void => {
            return Err(CompileError::TypeError(
                format!(
                    "Cannot infer type for variable '{}' - expression has void type",
                    name
                ),
                None,
            ))
        }
    };

    let alloca = compiler
        .builder
        .build_alloca(basic_type, name)
        .map_err(CompileError::from)?;

    if let Some(init_expr) = initializer {
        // Use the saved value if we already compiled it for type inference
        let value = if let Some(saved_value) = compiled_value {
            saved_value
        } else {
            compiler.compile_expression(init_expr)?
        };

        // Handle function pointers specially
        if let Some(type_) = type_ {
            if matches!(type_, AstType::Function { .. }) {
                // For function pointers, we need to get the function and store its pointer
                if let Expression::Identifier(func_name) = init_expr {
                    if let Some(function) = compiler.module.get_function(func_name) {
                        // Store the function pointer
                        let func_ptr = function.as_global_value().as_pointer_value();
                        compiler
                            .builder
                            .build_store(alloca, func_ptr)
                            .map_err(CompileError::from)?;
                        compiler.variables.insert(
                            name.clone(),
                            crate::codegen::llvm::VariableInfo {
                                pointer: alloca,
                                ast_type: type_.clone(),
                                is_mutable: *is_mutable,
                                is_initialized: true,
                                definition_span: compiler.get_current_span(),
                            },
                        );
                        return Ok(());
                    } else {
                        return Err(CompileError::UndeclaredFunction(
                            func_name.clone(),
                            compiler.get_current_span(),
                        ));
                    }
                } else {
                    return Err(CompileError::TypeError(
                        "Function pointer initializer must be a function name".to_string(),
                        compiler.get_current_span(),
                    ));
                }
            } else if let AstType::Bool = type_ {
                // For booleans, store directly as i1
                compiler
                    .builder
                    .build_store(alloca, value)
                    .map_err(CompileError::from)?;
                compiler.variables.insert(
                    name.clone(),
                    crate::codegen::llvm::VariableInfo {
                        pointer: alloca,
                        ast_type: type_.clone(),
                        is_mutable: *is_mutable,
                        is_initialized: true,
                        definition_span: compiler.get_current_span(),
                    },
                );
                return Ok(());
            } else if type_.is_ptr_type() {
                // For pointers, if the initializer is AddressOf, use the pointer inside the alloca
                let ptr_value = match init_expr {
                    Expression::AddressOf(expr) => {
                        // AddressOf returns a pointer - compile it to get the pointer value
                        compiler.compile_expression(expr)?
                    }
                    _ => {
                        // Otherwise, compile normally
                        value
                    }
                };
                compiler
                    .builder
                    .build_store(alloca, ptr_value)
                    .map_err(CompileError::from)?;
                compiler.variables.insert(
                    name.clone(),
                    crate::codegen::llvm::VariableInfo {
                        pointer: alloca,
                        ast_type: type_.clone(),
                        is_mutable: *is_mutable,
                        is_initialized: true,
                        definition_span: compiler.get_current_span(),
                    },
                );
                return Ok(());
            }
        }

        // Store the value using coercing_store to handle type mismatches
        // (e.g., i64 literal into i32 alloca - this was causing memory corruption)
        compiler.coercing_store(value, alloca, basic_type, &format!("variable '{}'", name))?;

        // Determine the AST type to store
        let ast_type_to_store = if let Some(type_) = type_ {
            type_.clone()
        } else if let Some(inferred) = inferred_ast_type {
            inferred
        } else {
            // Fallback: infer from LLVM type
            match value {
                BasicValueEnum::IntValue(int_val) => {
                    let bit_width = int_val.get_type().get_bit_width();
                    if bit_width == 1 {
                        AstType::Bool
                    } else if bit_width <= 32 {
                        AstType::I32
                    } else {
                        AstType::I64
                    }
                }
                BasicValueEnum::FloatValue(_) => AstType::F64,
                BasicValueEnum::PointerValue(_) => AstType::ptr(AstType::Void),
                BasicValueEnum::StructValue(_) => {
                    // For structs, we need to get the type from the struct_types map
                    // This is a simplified version - in practice, you'd need to track struct types
                    AstType::Void // Placeholder
                }
                _ => AstType::I64,
            }
        };

        compiler.variables.insert(
            name.clone(),
            crate::codegen::llvm::VariableInfo {
                pointer: alloca,
                ast_type: ast_type_to_store,
                is_mutable: *is_mutable,
                is_initialized: true,
                definition_span: compiler.get_current_span(),
            },
        );
        Ok(())
    } else {
        // No initializer - initialize to zero/default
        let zero: BasicValueEnum = match llvm_type {
            Type::Basic(BasicTypeEnum::IntType(int_type)) => int_type.const_zero().into(),
            Type::Basic(BasicTypeEnum::FloatType(float_type)) => float_type.const_zero().into(),
            Type::Basic(BasicTypeEnum::PointerType(_)) => {
                compiler.context.i64_type().const_zero().into()
            }
            _ => compiler.context.i64_type().const_zero().into(),
        };
        compiler
            .builder
            .build_store(alloca, zero)
            .map_err(CompileError::from)?;

        if let Some(type_) = type_ {
            compiler.variables.insert(
                name.clone(),
                crate::codegen::llvm::VariableInfo {
                    pointer: alloca,
                    ast_type: type_.clone(),
                    is_mutable: *is_mutable,
                    is_initialized: false, // Forward declaration without initializer
                    definition_span: compiler.get_current_span(),
                },
            );
            Ok(())
        } else {
            // For inferred types without initializer, default to i64
            compiler.variables.insert(
                name.clone(),
                crate::codegen::llvm::VariableInfo {
                    pointer: alloca,
                    ast_type: AstType::I64,
                    is_mutable: *is_mutable,
                    is_initialized: false, // Forward declaration without initializer
                    definition_span: compiler.get_current_span(),
                },
            );
            Ok(())
        }
    }
}

pub fn compile_assignment<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statement: &Statement,
) -> Result<(), CompileError> {
    match statement {
        Statement::VariableAssignment { name, value, .. } => {
            // Get the variable info
            let var_info = compiler.variables.get(name).cloned().ok_or_else(|| {
                CompileError::UndeclaredVariable(name.clone(), compiler.get_current_span())
            })?;

            // Check if variable is mutable
            if !var_info.is_mutable {
                return Err(CompileError::TypeError(
                    format!("Cannot assign to immutable variable '{}'", name),
                    compiler.get_current_span(),
                ));
            }

            // Compile the value
            let compiled_value = compiler.compile_expression(value)?;

            // Store the value
            compiler
                .builder
                .build_store(var_info.pointer, compiled_value)?;

            // Mark as initialized if it wasn't already
            if let Some(var_info) = compiler.variables.get_mut(name) {
                var_info.is_initialized = true;
            }

            Ok(())
        }
        Statement::PointerAssignment { pointer, value, .. } => {
            if let Expression::ArrayIndex { array, index } = pointer {
                let element_ptr = compiler.compile_array_index_address(array, index)?;
                let val = compiler.compile_expression(value)?;
                compiler.builder.build_store(element_ptr, val)?;
                Ok(())
            } else if let Expression::PointerDereference(ptr_expr) = pointer {
                // ptr.val = value: store value at the address ptr points to
                if let Expression::Identifier(name) = &**ptr_expr {
                    if let Ok((alloca, ast_type)) = compiler.get_variable(name) {
                        if let Some(inner) = ast_type.ptr_inner() {
                            let ptr_type =
                                compiler.context.ptr_type(inkwell::AddressSpace::default());
                            let ptr_val = compiler.builder.build_load(
                                ptr_type,
                                alloca,
                                "load_ptr_for_store",
                            )?;
                            let val = compiler.compile_expression(value)?;
                            let inner_llvm_type = compiler.to_llvm_type(inner)?;
                            let expected_type = match inner_llvm_type {
                                crate::codegen::llvm::Type::Basic(ty) => ty,
                                crate::codegen::llvm::Type::Struct(st) => st.into(),
                                _ => {
                                    compiler
                                        .builder
                                        .build_store(ptr_val.into_pointer_value(), val)?;
                                    return Ok(());
                                }
                            };
                            compiler.coercing_store(
                                val,
                                ptr_val.into_pointer_value(),
                                expected_type,
                                &format!("pointer dereference of '{}'", name),
                            )?;
                            return Ok(());
                        }
                    }
                }
                let ptr_value = compiler.compile_expression(ptr_expr)?;
                let val = compiler.compile_expression(value)?;
                if let BasicValueEnum::PointerValue(ptr) = ptr_value {
                    compiler.builder.build_store(ptr, val)?;
                    Ok(())
                } else {
                    Err(CompileError::TypeError(
                        "Pointer assignment requires a pointer value".to_string(),
                        None,
                    ))
                }
            } else if let Expression::MemberAccess { object, member } = pointer {
                // ptr.val.field = value: store value at field within dereferenced struct
                if let Expression::PointerDereference(ptr_expr) = &**object {
                    let ptr_val = compiler.compile_expression(ptr_expr)?;
                    if let BasicValueEnum::PointerValue(struct_ptr) = ptr_val {
                        let ptr_type = compiler.infer_expression_type(ptr_expr)?;
                        let struct_name = if let Some(inner) = ptr_type.ptr_inner() {
                            match inner {
                                crate::ast::AstType::Struct { name, .. } => Some(name.clone()),
                                crate::ast::AstType::Generic { name, .. } => Some(name.clone()),
                                _ => None,
                            }
                        } else {
                            None
                        };

                        if let Some(struct_name) = struct_name {
                            if let Some(struct_info) = compiler.struct_types.get(&struct_name) {
                                if let Some((field_idx, _field_type)) =
                                    struct_info.fields.get(member)
                                {
                                    let field_ptr = compiler.builder.build_struct_gep(
                                        struct_info.llvm_type,
                                        struct_ptr,
                                        *field_idx as u32,
                                        &format!("{}_field_ptr", member),
                                    )?;
                                    let val = compiler.compile_expression(value)?;
                                    compiler.builder.build_store(field_ptr, val)?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                let ptr_value = compiler.compile_expression(pointer)?;
                let val = compiler.compile_expression(value)?;
                if let BasicValueEnum::PointerValue(ptr) = ptr_value {
                    compiler.builder.build_store(ptr, val)?;
                    Ok(())
                } else {
                    Err(CompileError::TypeError(
                        "Pointer assignment requires a pointer value".to_string(),
                        None,
                    ))
                }
            } else {
                let ptr_value = compiler.compile_expression(pointer)?;
                let val = compiler.compile_expression(value)?;
                if let BasicValueEnum::PointerValue(ptr) = ptr_value {
                    compiler.builder.build_store(ptr, val)?;
                    Ok(())
                } else {
                    Err(CompileError::TypeError(
                        "Pointer assignment requires a pointer value".to_string(),
                        None,
                    ))
                }
            }
        }
        _ => Err(CompileError::InternalError(
            "Expected assignment statement".to_string(),
            None,
        )),
    }
}

pub fn compile_forward_declaration<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statement: &Statement,
) -> Result<(), CompileError> {
    // Forward declarations are handled as part of VariableDeclaration
    compile_variable_declaration(compiler, statement)
}
