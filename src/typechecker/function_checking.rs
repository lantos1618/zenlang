//! Function type checking

use crate::ast::{AstType, Function};
use crate::error::Result;
use crate::typechecker::TypeChecker;

/// Type check a function definition
pub fn check_function(checker: &mut TypeChecker, function: &Function) -> Result<()> {
    checker.enter_scope();

    // Track current function name for variable collection into TypeContext
    let prev_function_name = checker.current_function_name.take();
    checker.current_function_name = Some(function.name.clone());

    // Set the expected return type for this function
    checker.set_function_return_type(Some(function.return_type.clone()));

    // Add function parameters to scope
    // All parameters are immutable (mutable params via :: syntax not yet supported)
    for (param_name, param_type) in &function.args {
        // Special handling for 'self' parameter in trait implementations
        let actual_type = if param_name == "self" {
            match param_type {
                AstType::Generic { name, .. } if name == "Self" || name.starts_with("Self_") => {
                    // Replace Self with the concrete implementing type
                    if let Some(impl_type) = &checker.current_impl_type {
                        // Look up the actual struct fields from the type store
                        let struct_fields = checker
                            .type_store
                            .borrow()
                            .get_struct(impl_type)
                            .map(|s| s.fields.clone());
                        if let Some(fields) = struct_fields {
                            AstType::Struct {
                                name: impl_type.clone(),
                                fields,
                            }
                        } else {
                            // Fallback if struct not found
                            AstType::Struct {
                                name: impl_type.clone(),
                                fields: vec![],
                            }
                        }
                    } else {
                        param_type.clone()
                    }
                }
                _ => param_type.clone(),
            }
        } else {
            param_type.clone()
        };

        checker.declare_variable(param_name, actual_type, false)?; // false = immutable
    }

    // Check function body
    for statement in &function.body {
        super::statement_checking::check_statement(checker, statement)?;
    }

    // Clear the expected return type
    checker.set_function_return_type(None);

    // Restore previous function name
    checker.current_function_name = prev_function_name;

    checker.exit_scope();
    Ok(())
}
