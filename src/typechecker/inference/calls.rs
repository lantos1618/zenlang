//! Function and method call type inference

use super::casts::infer_cast_type;
use super::helpers::extract_type_name;
use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result, Span};
use crate::stdlib_types::{self, StdlibTypeRegistry};
use crate::typechecker::intrinsics;
use crate::typechecker::method_types;
use crate::typechecker::TypeChecker;

/// Substitute generic type params in a return type based on the caller's concrete type args.
/// E.g., calling `Ptr<u8>.from_addr(...)` where `from_addr` returns `Ptr<T>` → substitutes T→u8.
fn substitute_generic_type_params(caller_type_name: &str, return_type: &AstType) -> AstType {
    let (_, caller_type_args) = crate::parser::parse_generic_type_string(caller_type_name);
    if caller_type_args.is_empty() {
        return return_type.clone();
    }
    substitute_type_args_recursive(return_type, &caller_type_args, 0)
}

const MAX_SUBSTITUTION_DEPTH: usize = 64;

fn substitute_type_args_recursive(
    ty: &AstType,
    concrete_args: &[AstType],
    depth: usize,
) -> AstType {
    if depth > MAX_SUBSTITUTION_DEPTH {
        return ty.clone();
    }
    match ty {
        AstType::Generic { name, type_args } => {
            // Single-letter generic param (T, U, V, etc.) — substitute with concrete arg
            if name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase()) {
                let index = (name.chars().next().unwrap() as usize).saturating_sub('T' as usize);
                if index < concrete_args.len() {
                    return concrete_args[index].clone();
                }
            }
            // Recurse into type args (e.g., Ptr<T> → Ptr<u8>)
            AstType::Generic {
                name: name.clone(),
                type_args: type_args
                    .iter()
                    .map(|t| substitute_type_args_recursive(t, concrete_args, depth + 1))
                    .collect(),
            }
        }
        _ => ty.clone(),
    }
}

/// Infer the return type of a function call
pub fn infer_function_call_type(
    checker: &mut TypeChecker,
    name: &str,
    type_args: &[AstType],
    args: &[Expression],
    call_span: Option<Span>,
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
                                call_span.clone().or_else(|| checker.get_current_span()),
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
                if checker.type_store.borrow().has_struct(module)
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

    if name == "not" {
        return Ok(AstType::Bool);
    }

    if name == "cast" || name == "builtin.cast" {
        return infer_cast_type(
            args,
            call_span.clone().or_else(|| checker.get_current_span()),
        );
    }

    // Handle generic types with explicit type_args from AST
    if !type_args.is_empty()
        && (checker.type_store.borrow().has_struct(name)
            || checker.get_stdlib_struct(name).is_some())
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
                call_span.clone().or_else(|| checker.get_current_span()),
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
                call_span.clone().or_else(|| checker.get_current_span()),
            ))
        }
        Ok(_other) => Err(CompileError::not_a_function(
            name,
            call_span.clone().or_else(|| checker.get_current_span()),
        )),
        Err(_) => Err(CompileError::unknown_function(
            name,
            call_span.or_else(|| checker.get_current_span()),
        )),
    }
}

/// Infer the return type of a method call.
///
/// Resolution is split into two phases:
///   Phase 1 - Static calls: object is a type/module name (e.g., `String.len`, `compiler.alloc`)
///   Phase 2 - Instance calls: object is a value, resolve method on its inferred type
///
/// Each phase uses a well-defined pipeline of resolution strategies (see helpers below).
/// If no strategy matches, a TypeError is returned — never a silent Void.
pub fn infer_method_call_type(
    checker: &mut TypeChecker,
    object: &Expression,
    method: &str,
    type_args: &[AstType],
    call_span: Option<Span>,
) -> Result<AstType> {
    // === Phase 1: Static/module calls (object is a type name or module) ===
    if let Expression::Identifier(name) = object {
        if let Some(result) = try_resolve_static_call(checker, name, method, type_args) {
            return result;
        }
    }

    // === Phase 2: Infer object type, then resolve method on it ===
    let object_type = checker.infer_expression_type(object)?;
    let effective_type = object_type
        .ptr_inner()
        .cloned()
        .unwrap_or_else(|| object_type.clone());

    if let Some(return_type) =
        try_resolve_instance_method(checker, &object_type, &effective_type, method)
    {
        return Ok(return_type);
    }

    // === Phase 3: StdModule fallback ===
    // Module functions (io.println, math.sqrt, etc.) should resolve in Phase 1 via
    // get_stdlib_function_type() with alias resolution (e.g., "io" → "@std.io").
    // If we reach here, the module was accessed indirectly (e.g., via a variable alias
    // like `let m = io; m.println("hello")`), so we can't resolve the module path.
    // Return Void as a graceful fallback — this is the ONE intentional Void return.
    if effective_type == AstType::StdModule {
        return Ok(AstType::Void);
    }

    // === Phase 4: No resolution strategy matched — error, never silent Void ===
    let type_desc = extract_type_name(&effective_type)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{:?}", effective_type));
    Err(CompileError::TypeError(
        format!("Method '{}' not found on type '{}'", method, type_desc),
        call_span.or_else(|| checker.get_current_span()),
    ))
}

/// Phase 1 helper: resolve calls where the object is a type name or module identifier.
///
/// Covers: compiler intrinsics, stdlib methods/functions, user-defined UFC static calls,
/// and constructors (Type.new()).
///
/// Returns `Some(Ok(...))` on match, `Some(Err(...))` on type error, `None` to continue
/// to Phase 2 (instance method resolution).
fn try_resolve_static_call(
    checker: &TypeChecker,
    name: &str,
    method: &str,
    type_args: &[AstType],
) -> Option<Result<AstType>> {
    // 1. Compiler intrinsics (compiler.*, @builtin.*)
    if name == "compiler" || name == "builtin" || name == "@builtin" {
        if let Some(return_type) = crate::intrinsics::get_intrinsic_return_type(method) {
            // Substitute generic type params (e.g., compiler.alloc<MyStruct>())
            if !type_args.is_empty() {
                if let AstType::Generic {
                    name: type_name, ..
                } = &return_type
                {
                    if type_name.len() == 1 && type_name.chars().all(|c| c.is_ascii_uppercase()) {
                        return Some(Ok(type_args[0].clone()));
                    }
                }
            }
            return Some(Ok(return_type));
        }
    }

    // 2. Stdlib methods (Type::method in stdlib_methods registry)
    if let Some(return_type) = checker.get_stdlib_method_type(name, method) {
        return Some(Ok(return_type.clone()));
    }

    // 3. Stdlib functions (module::function in stdlib_functions registry)
    if let Some(return_type) = checker.get_stdlib_function_type(name, method) {
        return Some(Ok(return_type.clone()));
    }

    // 4. User-defined UFC methods (Type.method in function signatures)
    //    Uses find_ufc_method to handle generic names (e.g., "SafePtr<T>.is_valid")
    if let Some(func_sig) = checker.find_ufc_method(name, method) {
        let return_type = substitute_generic_type_params(name, &func_sig.return_type);
        return Some(Ok(return_type));
    }

    // 5. Constructors with explicit type args (e.g., HashMap.new<i32, String>())
    if method == "new" && !type_args.is_empty() {
        return Some(Ok(AstType::Generic {
            name: name.to_string(),
            type_args: type_args.to_vec(),
        }));
    }

    // 6. Constructor fallback: known types with .new()
    if method == "new" {
        if let Some(return_type) = checker.get_stdlib_method_type(name, "new") {
            return Some(Ok(return_type.clone()));
        }
        if checker.type_store.borrow().has_struct(name) || checker.get_stdlib_struct(name).is_some()
        {
            return Some(Ok(AstType::Generic {
                name: name.to_string(),
                type_args: vec![],
            }));
        }
    }

    None // Not a static call — fall through to instance method resolution
}

/// Phase 2 helper: resolve a method call on an instance value.
///
/// Tries resolution strategies in a defined order:
///   1. Free function with first-param match (UFCS)
///   2. Stdlib methods (by extracted type name)
///   3. User-defined UFC methods (Type.method in function signatures)
///   4. String methods (hardcoded)
///   5. Special "loop" method
///   6. Generic collection methods (HashMap, Vec, etc. — architecture violation, Phase 5 fix)
///   7. Pointer methods
///   8. Trait/behavior methods
///   9. Function pointer fields (vtable pattern)
///
/// Returns `Some(return_type)` on match, `None` if no strategy matched.
fn try_resolve_instance_method(
    checker: &TypeChecker,
    object_type: &AstType,
    effective_type: &AstType,
    method: &str,
) -> Option<AstType> {
    // Strategy 1: Free function with first-param type match (UFCS)
    //   e.g., `draw(canvas, x, y)` callable as `canvas.draw(x, y)`
    if let Some(func_type) = checker.get_function_signatures().get(method) {
        if !func_type.params.is_empty() {
            let (_, first_param_type) = &func_type.params[0];
            if first_param_type == effective_type
                || first_param_type == object_type
                || types_match_by_name(first_param_type, effective_type)
                || types_match_by_name(first_param_type, object_type)
            {
                return Some(func_type.return_type.clone());
            }
        }
    }

    // Strategy 2 & 3: Type-name based lookups (stdlib + user-defined UFC)
    //   Extracts the type name from Struct, Generic, or Enum, then looks up:
    //   - stdlib_methods["TypeName::method"]
    //   - functions["TypeName.method"]
    if let Some(type_name) = extract_type_name(effective_type) {
        // 2. Stdlib methods
        if let Some(return_type) = checker.get_stdlib_method_type(type_name, method) {
            // Apply generic substitution: if the receiver has type args (e.g., Vec<Entry<K,V>>),
            // substitute them into the return type (e.g., Option<T> -> Option<Entry<K,V>>).
            let substituted = if let AstType::Generic { type_args, .. } = effective_type {
                if !type_args.is_empty() {
                    substitute_type_args_recursive(&return_type, type_args, 0)
                } else {
                    return_type.clone()
                }
            } else {
                return_type.clone()
            };
            return Some(substituted);
        }
        // 3. User-defined UFC methods (handles generic names like "SafePtr<T>.is_valid")
        if let Some(func_sig) = checker.find_ufc_method(type_name, method) {
            let return_type = if let AstType::Generic { type_args, .. } = effective_type {
                if !type_args.is_empty() {
                    substitute_type_args_recursive(&func_sig.return_type, type_args, 0)
                } else {
                    func_sig.return_type.clone()
                }
            } else {
                func_sig.return_type.clone()
            };
            return Some(return_type);
        }
    }

    // Strategy 4: String methods (hardcoded in method_types)
    let is_string_struct = matches!(
        effective_type,
        AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name)
    );
    if is_string_struct
        || *effective_type == AstType::StaticString
        || *effective_type == AstType::StaticLiteral
    {
        if let Some(return_type) = method_types::infer_string_method_type(method, is_string_struct)
        {
            return Some(return_type);
        }
    }

    // Strategy 5: Special "loop" method (always returns Void)
    if method == "loop" {
        return Some(AstType::Void);
    }

    // Strategy 6: Generic collection methods (architecture violation — Phase 5 trait fix)
    //   HashMap, HashSet, Vec, DynVec, Result have hardcoded method inference because
    //   the trait system is not yet implemented. See docs/design/SEPARATION_OF_CONCERNS.md.
    if let AstType::Generic {
        name,
        type_args: obj_type_args,
    } = object_type
    {
        if stdlib_types::is_hashmap(name) {
            if let Some(return_type) =
                method_types::infer_hashmap_method_type(method, obj_type_args)
            {
                return Some(return_type);
            }
        } else if stdlib_types::is_hashset(name) {
            if let Some(return_type) = method_types::infer_hashset_method_type(method) {
                return Some(return_type);
            }
        } else if checker.well_known.is_result(name) {
            if let Some(return_type) = method_types::infer_result_method_type(method, obj_type_args)
            {
                return Some(return_type);
            }
        } else if stdlib_types::is_vec_type(name) && !obj_type_args.is_empty() {
            if let Some(return_type) =
                method_types::infer_vec_method_type(method, &obj_type_args[0])
            {
                return Some(return_type);
            }
        }
    }

    // Strategy 7: Pointer methods (Ptr<T>, MutPtr<T>, RawPtr<T>)
    if let Some(inner) = object_type.ptr_inner() {
        if let Some(return_type) = method_types::infer_pointer_method_type(method, inner) {
            return Some(return_type);
        }
    }

    // Strategy 8: Trait/behavior methods
    if let Some(type_name) = extract_type_name(effective_type) {
        if let Some(method_info) = checker.resolve_trait_method(type_name, method) {
            return Some(method_info.return_type);
        }
    }

    // Strategy 9: Function pointer fields (vtable pattern)
    //   e.g., self.allocate_fn(ctx, size) where allocate_fn is a field of type (RawPtr<u8>, usize) RawPtr<u8>
    //   Uses extract_type_name to handle BOTH Struct and Generic type variants.
    if let Some(return_type) = try_resolve_fn_ptr_field(checker, effective_type, method) {
        return Some(return_type);
    }

    // Strategy 10: Generic type parameter method calls (deferred to monomorphization)
    //   e.g., behavior.on_start() where behavior is type B (a generic param)
    //   Single-letter uppercase names are type parameters — accept any method call on them.
    if let AstType::Generic { name, type_args } = effective_type {
        if type_args.is_empty() && name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase()) {
            return Some(AstType::Void);
        }
    }

    None
}

/// Strategy 9 helper: check if `method` is a function pointer field on the struct.
///
/// Handles both `AstType::Struct { name }` and `AstType::Generic { name }` — the old code
/// only checked Generic, silently missing Struct types.
/// Check if two types refer to the same named type, even when represented
/// differently (e.g., Generic { name: "Point" } vs Struct { name: "Point" }).
/// The parser creates Generic for type annotations while the typechecker
/// creates Struct for struct literal inference — this bridges that gap.
fn types_match_by_name(a: &AstType, b: &AstType) -> bool {
    let name_a = match a {
        AstType::Struct { name, .. }
        | AstType::Generic { name, .. }
        | AstType::Enum { name, .. } => Some(name.as_str()),
        _ => None,
    };
    let name_b = match b {
        AstType::Struct { name, .. }
        | AstType::Generic { name, .. }
        | AstType::Enum { name, .. } => Some(name.as_str()),
        _ => None,
    };
    name_a.is_some() && name_a == name_b
}

fn try_resolve_fn_ptr_field(
    checker: &TypeChecker,
    effective_type: &AstType,
    method: &str,
) -> Option<AstType> {
    let type_name = extract_type_name(effective_type)?;
    let type_store = checker.type_store.borrow();
    let struct_info = type_store.get_struct(type_name)?;
    let fields: Vec<(String, AstType)> = struct_info.fields.clone();
    drop(type_store);
    for (field_name, field_type) in &fields {
        if field_name == method {
            match field_type {
                AstType::FunctionPointer { return_type, .. } => {
                    return Some(*return_type.clone());
                }
                AstType::Function { return_type, .. } => {
                    return Some(*return_type.clone());
                }
                _ => {}
            }
        }
    }
    None
}
