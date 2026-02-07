//! Scope management for type checker
//! Handles variable scoping, declaration, and lookup

use super::{TypeChecker, VariableInfo};
use crate::ast::AstType;
use crate::error::{CompileError, Result, Span};
use std::collections::HashMap;

/// Enter a new scope
pub fn enter_scope(checker: &mut TypeChecker) {
    checker.scopes.push(HashMap::new());
}

/// Exit the current scope
pub fn exit_scope(checker: &mut TypeChecker) {
    checker.scopes.pop();
}

/// Declare a variable in the current scope
pub fn declare_variable(
    checker: &mut TypeChecker,
    name: &str,
    type_: AstType,
    is_mutable: bool,
    span: Option<Span>,
) -> Result<()> {
    // Only check current scope for duplicates - shadowing from outer scopes is allowed
    if variable_exists_in_current_scope(checker, name) {
        return Err(CompileError::TypeError(
            format!("Variable '{}' already declared in this scope", name),
            span,
        ));
    }

    let info = VariableInfo {
        type_,
        is_mutable,
        is_initialized: false,
    };

    if let Some(scope) = checker.scopes.last_mut() {
        scope.insert(name.to_string(), info);
    } else {
        return Err(CompileError::InternalError(
            "No active scope".to_string(),
            span,
        ));
    }

    Ok(())
}

/// Declare a variable with initialization
pub fn declare_variable_with_init(
    checker: &mut TypeChecker,
    name: &str,
    type_: AstType,
    is_mutable: bool,
    is_initialized: bool,
    span: Option<Span>,
) -> Result<()> {
    // Only check current scope for duplicates - shadowing from outer scopes is allowed
    if variable_exists_in_current_scope(checker, name) {
        return Err(CompileError::TypeError(
            format!("Variable '{}' already declared in this scope", name),
            span,
        ));
    }

    let info = VariableInfo {
        type_,
        is_mutable,
        is_initialized,
    };

    if let Some(scope) = checker.scopes.last_mut() {
        scope.insert(name.to_string(), info);
    } else {
        return Err(CompileError::InternalError(
            "No active scope".to_string(),
            span,
        ));
    }

    Ok(())
}

/// Mark a variable as initialized
pub fn mark_variable_initialized(checker: &mut TypeChecker, name: &str) -> Result<()> {
    let span = checker.get_current_span();
    for scope in checker.scopes.iter_mut().rev() {
        if let Some(var_info) = scope.get_mut(name) {
            var_info.is_initialized = true;
            return Ok(());
        }
    }

    Err(CompileError::TypeError(
        format!("Variable '{}' not found", name),
        span,
    ))
}

/// Get the type of a variable
/// This version includes special handling for generics and enums
pub fn get_variable_type(checker: &TypeChecker, name: &str) -> Result<AstType> {
    for scope in checker.scopes.iter().rev() {
        if let Some(var_info) = scope.get(name) {
            // Handle generic placeholders - convert to appropriate defaults
            if let AstType::Generic {
                name: type_name,
                type_args,
            } = &var_info.type_
            {
                if type_name == "T" && type_args.is_empty() {
                    return Ok(AstType::I32);
                } else if type_name == "E" && type_args.is_empty() {
                    // Error types default to String
                    return Ok(crate::ast::resolve_string_struct_type());
                }
            }
            return Ok(var_info.type_.clone());
        }
    }

    if checker.type_store.borrow().has_enum(name) {
        // Return a special type to indicate this is an enum type constructor
        return Ok(AstType::EnumType {
            name: name.to_string(),
        });
    }

    Err(CompileError::TypeError(
        format!("Undefined variable: {}", name),
        checker.get_current_span(),
    ))
}

/// Get variable info (type, mutability, initialization status)
pub fn get_variable_info(checker: &TypeChecker, name: &str) -> Result<VariableInfo> {
    for scope in checker.scopes.iter().rev() {
        if let Some(var_info) = scope.get(name) {
            return Ok(var_info.clone());
        }
    }

    Err(CompileError::TypeError(
        format!("Variable '{}' not found", name),
        checker.get_current_span(),
    ))
}

/// Check if a variable exists in any scope
pub fn variable_exists(checker: &TypeChecker, name: &str) -> bool {
    checker
        .scopes
        .iter()
        .rev()
        .any(|scope| scope.contains_key(name))
}

/// Check if a variable exists in the current (innermost) scope only
pub fn variable_exists_in_current_scope(checker: &TypeChecker, name: &str) -> bool {
    checker
        .scopes
        .last()
        .map(|scope| scope.contains_key(name))
        .unwrap_or(false)
}
