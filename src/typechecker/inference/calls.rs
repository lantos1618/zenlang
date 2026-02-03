//! Function and method call type inference

use super::casts::infer_cast_type;
use super::helpers::extract_type_name;
use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};
use crate::stdlib_types::{self, StdlibTypeRegistry};
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

                // Handle generic intrinsics: if return type is Generic<T> and type_args provided, substitute
                let return_type = result?;
                if !type_args.is_empty() {
                    if let AstType::Generic { name, .. } = &return_type {
                        // Single-letter generic names like "T" should be substituted
                        if name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase()) {
                            // Substitute with the first type argument
                            if !type_args.is_empty() {
                                return Ok(type_args[0].clone());
                            }
                        }
                    }
                }
                return Ok(return_type);
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

    // Debug: Check if it's a dotted name and if so, look up the full name
    if name.contains('.') {
        eprintln!(
            "DEBUG: Looking for function '{}', found: {}",
            name,
            checker.get_function_signatures().contains_key(name)
        );
    }

    match checker.get_variable_type(name) {
        Ok(AstType::FunctionPointer { return_type, .. }) => Ok(*return_type),
        // Also handle Function type (used by type aliases like CompletionFn)
        Ok(AstType::Function { return_type, .. }) => Ok(*return_type),
        // Handle type alias references (e.g., CompletionFn resolves to a function type)
        Ok(AstType::Generic {
            name: alias_name,
            type_args,
        }) if type_args.is_empty() => {
            // Check if this is a type alias that resolves to a function type
            if let Some(aliased_type) = checker.resolve_type_alias(&alias_name) {
                match aliased_type {
                    AstType::Function { return_type, .. } => return Ok(*return_type),
                    AstType::FunctionPointer { return_type, .. } => return Ok(*return_type),
                    _ => {}
                }
            }
            Err(CompileError::TypeError(
                format!("'{}' is not a function", name),
                checker.get_current_span(),
            ))
        }
        // Handle Struct type that might be a type alias name
        Ok(AstType::Struct {
            name: struct_name, ..
        }) => {
            // Check if this struct name is actually a type alias to a function type
            if let Some(aliased_type) = checker.resolve_type_alias(&struct_name) {
                match aliased_type {
                    AstType::Function { return_type, .. } => return Ok(*return_type),
                    AstType::FunctionPointer { return_type, .. } => return Ok(*return_type),
                    _ => {}
                }
            }
            Err(CompileError::TypeError(
                format!("'{}' is not a function", name),
                checker.get_current_span(),
            ))
        }
        Ok(_other) => Err(CompileError::not_a_function(
            name,
            checker.get_current_span(),
        )),
        Err(_) => Err(CompileError::unknown_function(
            name,
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
                // Handle generic intrinsics: if return type is Generic<T> and type_args provided, substitute
                if !type_args.is_empty() {
                    if let AstType::Generic {
                        name: type_name, ..
                    } = &return_type
                    {
                        // Single-letter generic names like "T" should be substituted
                        if type_name.len() == 1 && type_name.chars().all(|c| c.is_ascii_uppercase())
                        {
                            // Substitute with the first type argument
                            return Ok(type_args[0].clone());
                        }
                    }
                }
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

    // ========================================================================
    // ARCHITECTURE VIOLATION: Layer 3 stdlib types with hardcoded method inference
    // ========================================================================
    //
    // The code below provides special typechecker support for Layer 3 stdlib types:
    // HashMap, HashSet, Vec, and DynVec. This violates the three-layer architecture
    // described in docs/design/SEPARATION_OF_CONCERNS.md.
    //
    // These types should be pure stdlib implementations with no compiler awareness.
    // Instead, they currently get hardcoded method type inference because:
    // 1. The trait system is not yet implemented (Phase 5)
    // 2. Generic method return types can't be expressed without traits
    // 3. Methods like Vec<T>.get() -> T require trait-based type resolution
    //
    // TO FIX IN PHASE 5 (Trait System):
    // 1. Define traits: Collection<T>, Map<K,V>, Set<T>, etc.
    // 2. Implement trait methods in stdlib with proper type signatures
    // 3. Replace these hardcoded checks with trait method resolution
    // 4. Remove all string-based type identity checks for Layer 3 types
    //
    // See also: tech_debt_audit.md - "String-based type identity checks"
    // ========================================================================

    if let AstType::Generic { name, type_args } = &object_type {
        // Use centralized helpers from stdlib_types module to avoid duplicating string literals
        if stdlib_types::is_hashmap(name) {
            if let Some(return_type) = method_types::infer_hashmap_method_type(method, type_args) {
                return Ok(return_type);
            }
        } else if stdlib_types::is_hashset(name) {
            if let Some(return_type) = method_types::infer_hashset_method_type(method) {
                return Ok(return_type);
            }
        } else if checker.well_known.is_result(name) {
            // Result is Layer 2 - requires compiler support for pattern matching and .raise()
            if let Some(return_type) = method_types::infer_result_method_type(method, type_args) {
                return Ok(return_type);
            }
        } else if stdlib_types::is_vec_type(name) && !type_args.is_empty() {
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

    // Check if this is a function pointer field being called (manual vtable pattern)
    // e.g., self.allocate_fn(ctx, size) where allocate_fn is a field of type (RawPtr<u8>, usize) RawPtr<u8>
    if let AstType::Generic { name, .. } = effective_type {
        if let Some(struct_info) = checker.structs.get(name) {
            for (field_name, field_type) in &struct_info.fields {
                if field_name == method {
                    match field_type {
                        AstType::FunctionPointer { return_type, .. } => {
                            return Ok(*return_type.clone());
                        }
                        AstType::Function { return_type, .. } => {
                            return Ok(*return_type.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(AstType::Void)
}
