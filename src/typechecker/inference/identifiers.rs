//! Identifier type inference

use crate::ast::AstType;
use crate::error::{CompileError, Result};
use crate::typechecker::TypeChecker;

/// Infer the type of an identifier
pub fn infer_identifier_type(checker: &mut TypeChecker, name: &str) -> Result<AstType> {
    if let Ok(var_type) = checker.get_variable_type(name) {
        return Ok(var_type);
    }

    // Built-in modules are always available without explicit import
    if crate::intrinsics::is_builtin_module(name) {
        return Ok(AstType::StdModule);
    }

    if let Some(sig) = checker.get_function_signatures().get(name) {
        return Ok(AstType::FunctionPointer {
            param_types: sig.params.iter().map(|(_, t)| t.clone()).collect(),
            return_type: Box::new(sig.return_type.clone()),
        });
    }

    if name == "Array" {
        return Ok(AstType::Generic {
            name: "Array".to_string(),
            type_args: vec![],
        });
    }

    // Check if name is a struct type - used for static method calls like MyStruct.new()
    // Return a Struct type with empty fields (fields will be filled in during codegen)
    if checker.structs.contains_key(name) {
        return Ok(AstType::Struct {
            name: name.to_string(),
            fields: vec![],
        });
    }

    // If name contains generic syntax (e.g., "Vec<i32>"), parse it as a type
    // This handles cases where identifiers come in as strings with generic syntax
    if name.contains('<') {
        match crate::parser::parse_type_from_string(name) {
            Ok(parsed_type) => {
                // Handle parsed generic types
                match &parsed_type {
                    AstType::Generic {
                        name: base_name,
                        type_args,
                    } => {
                        let wk = &checker.well_known;
                        // Handle pointer types specially
                        if wk.is_immutable_ptr(base_name) {
                            let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                            return Ok(AstType::ptr(inner));
                        } else if wk.is_mutable_ptr(base_name) {
                            let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                            return Ok(AstType::mut_ptr(inner));
                        } else if wk.is_raw_ptr(base_name) {
                            let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                            return Ok(AstType::raw_ptr(inner));
                        }
                        // Return the parsed generic type as-is
                        return Ok(parsed_type);
                    }
                    // If it parsed to something else, return it
                    _ => return Ok(parsed_type),
                }
            }
            Err(_) => {
                // If parsing fails, fall through to error below
            }
        }
    }

    Err(CompileError::UndeclaredVariable(
        name.to_string(),
        checker.get_current_span(),
    ))
}
