//! Function and method call type inference

use super::casts::infer_cast_type;
use super::helpers::extract_type_name;
use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};
use crate::stdlib_types::StdlibTypeRegistry;
use crate::typechecker::intrinsics;
use crate::typechecker::method_types;
use crate::typechecker::TypeChecker;

/// Infer the return type of a function call
pub fn infer_function_call_type(
    checker: &mut TypeChecker,
    name: &str,
    type_args: &[AstType],
    args: &[Expression],
) -> Result<AstType> {
    if name.contains('.') {
        let parts: Vec<&str> = name.splitn(2, '.').collect();
        if parts.len() == 2 {
            let module = parts[0];
            let func = parts[1];

            if let Some(result) = intrinsics::check_compiler_intrinsic(module, func, args.len()) {
                if module == "compiler" && func == "inline_c" && args.len() == 1 {
                    let arg_type = checker.infer_expression_type(&args[0])?;
                    match arg_type {
                        AstType::StaticString | AstType::StaticLiteral => {}
                        _ => {
                            return Err(CompileError::TypeError(
                                "compiler.inline_c() requires a string literal argument"
                                    .to_string(),
                                checker.get_current_span(),
                            ))
                        }
                    }
                }
                return result;
            }

            if let Some(return_type) = checker.get_stdlib_function_type(module, func) {
                return Ok(return_type.clone());
            }

            // Handle generic constructors like HashMap.new<K, V> or Vec.new<T>
            if func == "new" && !type_args.is_empty() {
                // If we have explicit type args, return a generic type with those args
                // Check if it's a known struct type
                if checker.structs.contains_key(module)
                    || checker.get_stdlib_struct(module).is_some()
                {
                    return Ok(AstType::Generic {
                        name: module.to_string(),
                        type_args: type_args.to_vec(),
                    });
                }
            }
        }
    }

    if name == "cast" {
        return infer_cast_type(args, checker.get_current_span());
    }

    // Handle generic types with explicit type_args from AST
    if !type_args.is_empty()
        && (checker.structs.contains_key(name) || checker.get_stdlib_struct(name).is_some())
    {
        return Ok(AstType::Generic {
            name: name.to_string(),
            type_args: type_args.to_vec(),
        });
    }

    if let Some(sig) = checker.get_function_signatures().get(name) {
        return Ok(sig.return_type.clone());
    }

    match checker.get_variable_type(name) {
        Ok(AstType::FunctionPointer { return_type, .. }) => Ok(*return_type),
        Ok(_) => Err(CompileError::TypeError(
            format!("'{}' is not a function", name),
            checker.get_current_span(),
        )),
        Err(_) => Err(CompileError::TypeError(
            format!("Unknown function: {}", name),
            checker.get_current_span(),
        )),
    }
}

/// Infer the return type of a method call
pub fn infer_method_call_type(
    checker: &mut TypeChecker,
    object: &Expression,
    method: &str,
    type_args: &[AstType],
) -> Result<AstType> {
    if let Expression::Identifier(name) = object {
        // Check for compiler intrinsics first (compiler.* or @builtin.*)
        if let Some(return_type) = crate::intrinsics::get_intrinsic_return_type(method) {
            // For compiler/builtin modules, use the intrinsic's return type directly
            if name == "compiler" || name == "builtin" || name == "@builtin" {
                return Ok(return_type);
            }
        }

        // Check for methods (Type.method style like String.len)
        if let Some(return_type) = checker.get_stdlib_method_type(name, method) {
            return Ok(return_type.clone());
        }

        // Check for module functions (module.function style like gpa.default_gpa)
        if let Some(return_type) = checker.get_stdlib_function_type(name, method) {
            return Ok(return_type.clone());
        }

        // Check for user-defined attached methods (like MyStruct.new)
        let full_method_name = format!("{}.{}", name, method);
        if let Some(func_sig) = checker.get_function_signatures().get(&full_method_name) {
            return Ok(func_sig.return_type.clone());
        }

        // Handle constructors with type args (e.g., HashMap.new<i32, String>())
        if method == "new" && !type_args.is_empty() {
            return Ok(AstType::Generic {
                name: name.to_string(),
                type_args: type_args.to_vec(),
            });
        }

        // Handle constructors - check if type has a .new() method in stdlib
        if method == "new" {
            // First check if stdlib defines a return type for Type.new()
            if let Some(return_type) = checker.get_stdlib_method_type(name, "new") {
                return Ok(return_type.clone());
            }
            // If type is known but no explicit return type, return generic with empty type args
            // Type args will be inferred from usage context
            if checker.structs.contains_key(name) || checker.get_stdlib_struct(name).is_some() {
                return Ok(AstType::Generic {
                    name: name.to_string(),
                    type_args: vec![],
                });
            }
        }
    }

    let object_type = checker.infer_expression_type(object)?;

    let dereferenced_type = object_type.ptr_inner().cloned();

    let effective_type = dereferenced_type.as_ref().unwrap_or(&object_type);

    if let Some(func_type) = checker.get_function_signatures().get(method) {
        if !func_type.params.is_empty() {
            let (_, first_param_type) = &func_type.params[0];
            if first_param_type == effective_type || first_param_type == &object_type {
                return Ok(func_type.return_type.clone());
            }
        }
    }

    if let Some(type_name) = extract_type_name(effective_type) {
        if let Some(return_type) = checker.get_stdlib_method_type(type_name, method) {
            return Ok(return_type.clone());
        }
    }

    let is_string_struct = matches!(effective_type, AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name));
    if is_string_struct
        || *effective_type == AstType::StaticString
        || *effective_type == AstType::StaticLiteral
    {
        if let Some(return_type) = method_types::infer_string_method_type(method, is_string_struct)
        {
            return Ok(return_type);
        }
    }

    if method == "loop" {
        return Ok(AstType::Void);
    }

    if let AstType::Generic { name, type_args } = &object_type {
        if name == "HashMap" {
            if let Some(return_type) = method_types::infer_hashmap_method_type(method, type_args) {
                return Ok(return_type);
            }
        } else if name == "HashSet" {
            if let Some(return_type) = method_types::infer_hashset_method_type(method) {
                return Ok(return_type);
            }
        } else if checker.well_known.is_result(name) {
            if let Some(return_type) = method_types::infer_result_method_type(method, type_args) {
                return Ok(return_type);
            }
        } else if matches!(name.as_str(), "Vec" | "DynVec") && !type_args.is_empty() {
            if let Some(return_type) = method_types::infer_vec_method_type(method, &type_args[0]) {
                return Ok(return_type);
            }
        }
    }

    // Vec methods - Vec<T> is now Generic { name: "Vec", type_args: [T] }
    if let AstType::Generic { name, type_args } = &object_type {
        if name == "Vec" && !type_args.is_empty() {
            if let Some(return_type) = method_types::infer_vec_method_type(method, &type_args[0]) {
                return Ok(return_type);
            }
        }
    }

    // Pointer methods - check for Ptr<T>, MutPtr<T>, RawPtr<T> methods
    if let Some(inner) = object_type.ptr_inner() {
        if let Some(return_type) = method_types::infer_pointer_method_type(method, inner) {
            return Ok(return_type);
        }
    }

    // Try trait/behavior method resolution
    if let Some(type_name) = extract_type_name(effective_type) {
        if let Some(method_info) = checker.resolve_trait_method(type_name, method) {
            return Ok(method_info.return_type);
        }
    }

    Ok(AstType::Void)
}
