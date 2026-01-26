//! Function type checking

use crate::ast::{AstType, Function};
use crate::error::Result;
use crate::typechecker::TypeChecker;

/// Type check a function definition
pub fn check_function(checker: &mut TypeChecker, function: &Function) -> Result<()> {
    checker.enter_scope();

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
                        // Look up the actual struct fields from the registry
                        if let Some(struct_info) = checker.structs.get(impl_type) {
                            AstType::Struct {
                                name: impl_type.clone(),
                                fields: struct_info.fields.clone(),
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

    checker.exit_scope();
    Ok(())
}
