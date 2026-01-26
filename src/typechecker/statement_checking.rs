//! Statement type checking

use super::validation::types_compatible;
use crate::ast::{AstType, LoopKind, Statement};
use crate::error::{CompileError, Result};
use crate::typechecker::TypeChecker;

/// Type check a statement
pub fn check_statement(checker: &mut TypeChecker, statement: &Statement) -> Result<()> {
    match statement {
        Statement::VariableDeclaration {
            name,
            type_,
            initializer,
            is_mutable,
            span,
            ..
        } => {
            checker.set_current_span(span.clone());
            if let Some(init_expr) = initializer {
                // Check if variable already exists (forward declaration case)
                if checker.variable_exists(name) {
                    // This might be initialization of a forward-declared variable
                    let var_info = checker.get_variable_info(name)?;

                    // Allow initialization of forward-declared variables with = operator
                    // This works for both immutable (x: i32 then x = 10) and mutable (w:: i32 then w = 20)
                    if !var_info.is_initialized {
                        // This is initialization of a forward-declared variable
                        // The = operator can be used to initialize both immutable and mutable forward declarations
                        let inferred_type = checker.infer_expression_type(init_expr)?;
                        if !checker.types_compatible(&var_info.type_, &inferred_type) {
                            return Err(CompileError::TypeError(
                                    format!(
                                        "Type mismatch: variable '{}' declared as {:?} but initialized with {:?}",
                                        name, var_info.type_, inferred_type
                                    ),
                                    span.clone()
                                ));
                        }
                        checker.mark_variable_initialized(name)?;
                        return Ok(());
                    } else if var_info.is_initialized && var_info.is_mutable {
                        // This is a reassignment to an existing mutable variable
                        // (e.g., w = 25 after w:: i32 and w = 20)
                        let inferred_type = checker.infer_expression_type(init_expr)?;
                        if !checker.types_compatible(&var_info.type_, &inferred_type) {
                            return Err(CompileError::TypeError(
                                    format!(
                                        "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                                        inferred_type, name, var_info.type_
                                    ),
                                    span.clone()
                                ));
                        }
                        // Reassignment is allowed for mutable variables
                        return Ok(());
                    } else {
                        // Immutable variable already initialized - cannot reassign
                        return Err(CompileError::TypeError(
                            format!("Cannot reassign immutable variable '{}'", name),
                            span.clone(),
                        ));
                    }
                }

                // New variable declaration with initializer
                let inferred_type = checker.infer_expression_type(init_expr)?;

                if let Some(declared_type) = type_ {
                    // Check that the initializer type matches the declared type
                    if !checker.types_compatible(declared_type, &inferred_type) {
                        return Err(CompileError::TypeError(
                                format!(
                                    "Type mismatch: variable '{}' declared as {:?} but initialized with {:?}",
                                    name, declared_type, inferred_type
                                ),
                                span.clone()
                            ));
                    }
                    checker.declare_variable_with_init_and_span(
                        name,
                        declared_type.clone(),
                        *is_mutable,
                        true,
                        span.clone(),
                    )?;
                } else {
                    // Inferred type from initializer
                    checker.declare_variable_with_init_and_span(
                        name,
                        inferred_type,
                        *is_mutable,
                        true,
                        span.clone(),
                    )?;
                }
            } else if let Some(declared_type) = type_ {
                // Forward declaration without initializer
                checker.declare_variable_with_init_and_span(
                    name,
                    declared_type.clone(),
                    *is_mutable,
                    false,
                    span.clone(),
                )?;
            } else {
                return Err(CompileError::TypeError(
                    format!(
                        "Cannot infer type for variable '{}' without initializer",
                        name
                    ),
                    span.clone(),
                ));
            }
        }
        Statement::VariableAssignment { name, value, span } => {
            checker.set_current_span(span.clone());
            // Check if variable exists in CURRENT scope (not outer scopes)
            // This allows local variables to shadow imports/outer variables
            if !checker.variable_exists_in_current_scope(name) {
                // Check if it exists in an outer scope
                if checker.variable_exists(name) {
                    let var_info = checker.get_variable_info(name)?;
                    // If it's uninitialized in outer scope, this initializes it
                    if !var_info.is_initialized {
                        let value_type = checker.infer_expression_type(value)?;
                        if !checker.types_compatible(&var_info.type_, &value_type) {
                            return Err(CompileError::TypeError(
                                format!(
                                    "Type mismatch in initial assignment to '{}': expected {:?}, got {:?}",
                                    name, var_info.type_, value_type
                                ),
                                checker.get_current_span(),
                            ));
                        }
                        checker.mark_variable_initialized(name)?;
                    } else {
                        // Variable exists in outer scope and is initialized
                        // Create a new local variable that shadows it
                        let value_type = checker.infer_expression_type(value)?;
                        checker.declare_variable_with_init_and_span(
                            name,
                            value_type.clone(),
                            false,
                            true,
                            span.clone(),
                        )?;
                    }
                } else {
                    // Variable doesn't exist anywhere - create new immutable local
                    let value_type = checker.infer_expression_type(value)?;
                    checker.declare_variable_with_init_and_span(
                        name,
                        value_type.clone(),
                        false,
                        true,
                        span.clone(),
                    )?;
                }
            } else {
                // Variable exists in current scope - this is a reassignment
                let var_info = checker.get_variable_info(name)?;

                if !var_info.is_initialized {
                    // First assignment to forward-declared variable
                    let value_type = checker.infer_expression_type(value)?;
                    if !checker.types_compatible(&var_info.type_, &value_type) {
                        return Err(CompileError::TypeError(
                            format!(
                                "Type mismatch in initial assignment to '{}': expected {:?}, got {:?}",
                                name, var_info.type_, value_type
                            ),
                            checker.get_current_span(),
                        ));
                    }
                    checker.mark_variable_initialized(name)?;
                } else {
                    // Reassignment to existing variable in current scope
                    if !var_info.is_mutable {
                        return Err(CompileError::TypeError(
                            format!("Cannot reassign to immutable variable '{}'", name),
                            checker.get_current_span(),
                        ));
                    }

                    let value_type = checker.infer_expression_type(value)?;

                    if !checker.types_compatible(&var_info.type_, &value_type) {
                        return Err(CompileError::TypeError(
                            format!(
                                "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                                value_type, name, var_info.type_
                            ),
                            checker.get_current_span(),
                        ));
                    }
                }
            }
        }
        Statement::Return { expr, span } => {
            checker.set_current_span(span.clone());
            let return_type = checker.infer_expression_type(expr)?;

            // Check that return type matches expected function return type
            if let Some(expected) = checker.get_function_return_type() {
                if !types_compatible(&return_type, expected) {
                    return Err(CompileError::TypeError(
                        format!(
                            "Return type mismatch: expected {:?}, got {:?}",
                            expected, return_type
                        ),
                        span.clone(),
                    ));
                }
            }
        }
        Statement::Expression { expr, span } => {
            checker.set_current_span(span.clone());
            checker.infer_expression_type(expr)?;
        }
        Statement::Loop { kind, body, .. } => {
            checker.enter_scope();

            // Handle loop-specific variables
            match kind {
                LoopKind::Infinite => {
                    // No special handling needed
                }
                LoopKind::Condition(expr) => {
                    // Type check the condition
                    let cond_type = checker.infer_expression_type(expr)?;
                    // Condition should be boolean or integer (truthy)
                    if !matches!(cond_type, AstType::Bool | AstType::I32 | AstType::I64) {
                        return Err(CompileError::TypeError(
                            format!(
                                "Loop condition must be boolean or integer, got {:?}",
                                cond_type
                            ),
                            checker.get_current_span(),
                        ));
                    }
                }
            }

            // Check loop body with the variable in scope
            for stmt in body {
                checker.check_statement(stmt)?;
            }
            checker.exit_scope();
        }
        Statement::ComptimeBlock { statements, .. } => {
            checker.enter_scope();
            for stmt in statements {
                checker.check_statement(stmt)?;
            }
            checker.exit_scope();
        }
        Statement::PointerAssignment {
            pointer,
            value,
            span,
        } => {
            checker.set_current_span(span.clone());
            // For array indexing like arr[i] = value
            // The pointer expression should be a pointer type
            let pointer_type = checker.infer_expression_type(pointer)?;
            let value_type = checker.infer_expression_type(value)?;

            // Type check that value is compatible with the pointed-to type
            if let Some(inner_type) = pointer_type.ptr_inner() {
                if !types_compatible(inner_type, &value_type) {
                    return Err(CompileError::TypeError(
                        format!(
                            "Pointer assignment type mismatch: pointer points to {:?}, but value is {:?}",
                            inner_type, value_type
                        ),
                        span.clone(),
                    ));
                }
            }
            // If pointer is an array index, check element type compatibility
            else if let AstType::FixedArray { element_type, .. } = &pointer_type {
                if !types_compatible(element_type, &value_type) {
                    return Err(CompileError::TypeError(
                        format!(
                            "Array assignment type mismatch: array elements are {:?}, but value is {:?}",
                            element_type, value_type
                        ),
                        span.clone(),
                    ));
                }
            }
        }
        Statement::DestructuringImport { names, .. } => {
            // Handle destructuring imports: { io, math } = @std
            // Register each imported module as a variable with StdModule type
            for name in names {
                checker.declare_variable_with_init(name, AstType::StdModule, false, true)?;
            }
        }
        _ => {}
    }
    Ok(())
}
