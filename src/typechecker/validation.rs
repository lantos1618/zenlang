use crate::ast::AstType;
use crate::stdlib_types::StdlibTypeRegistry;
use crate::well_known::well_known;

/// Check if a name looks like a type parameter (single uppercase letter or short uppercase name)
fn is_type_parameter_name(name: &str) -> bool {
    // Type parameters are typically single uppercase letters (T, U, V, K, V)
    // or short uppercase names (Self)
    if name.is_empty() {
        return false;
    }
    // Single uppercase letter is definitely a type parameter
    if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
        return true;
    }
    // "Self" is a special type parameter
    if name == "Self" || name.starts_with("Self_") {
        return true;
    }
    false
}

/// Check if two types are compatible (for assignment, parameter passing, etc.)
pub fn types_compatible(expected: &AstType, actual: &AstType) -> bool {
    // Exact match is always compatible
    if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
        return true;
    }

    // Generic type parameters (like T, U, etc.) are compatible with any type
    // This allows return type checking in generic functions before instantiation
    if let AstType::Generic { name, type_args } = expected {
        // A bare type parameter (no type_args) is compatible with any type
        if type_args.is_empty() && is_type_parameter_name(name) {
            return true;
        }
    }
    // Also check if actual is a type parameter (for symmetry)
    if let AstType::Generic { name, type_args } = actual {
        if type_args.is_empty() && is_type_parameter_name(name) {
            return true;
        }
    }

    // Check for numeric compatibility with implicit conversions
    if expected.is_numeric() && actual.is_numeric() {
        // Allow widening conversions (smaller to larger)
        if let (Some(expected_size), Some(actual_size)) = (expected.bit_size(), actual.bit_size()) {
            // Allow if actual fits in expected
            if actual_size <= expected_size {
                // Check sign compatibility
                if expected.is_signed_integer() && actual.is_unsigned_integer() {
                    // Unsigned to signed is OK if there's room
                    return actual_size < expected_size;
                }
                return true;
            }
        }
    }

    // Check for string type compatibility
    // StaticLiteral is internal only
    // StaticString can be coerced to String (requires allocator at runtime)
    // But String cannot be coerced back to StaticString
    match (expected, actual) {
        // StaticLiteral is internal - it should be compatible with StaticString
        (AstType::StaticString, AstType::StaticLiteral) => return true,
        (AstType::StaticLiteral, AstType::StaticString) => return true,

        // StaticString -> String struct is ok (will need allocator at runtime)
        (AstType::Struct { name, .. }, AstType::StaticString)
            if StdlibTypeRegistry::is_string_type(name) =>
        {
            return true
        }
        (AstType::Struct { name, .. }, AstType::StaticLiteral)
            if StdlibTypeRegistry::is_string_type(name) =>
        {
            return true
        } // Internal literal -> dynamic is ok

        // String struct -> StaticString is NOT ok (would lose allocator)
        (AstType::StaticString, AstType::Struct { name, .. })
            if StdlibTypeRegistry::is_string_type(name) =>
        {
            return false
        }
        (AstType::StaticLiteral, AstType::Struct { name, .. })
            if StdlibTypeRegistry::is_string_type(name) =>
        {
            return false
        }
        _ => {}
    }

    // Check for pointer compatibility
    if expected.is_ptr_type() && actual.is_ptr_type() {
        if let (Some(expected_inner), Some(actual_inner)) =
            (expected.ptr_inner(), actual.ptr_inner())
        {
            return types_compatible(expected_inner, actual_inner);
        }
    }
    // Allow slice/array to decay to pointer
    if expected.is_ptr_type() {
        if let Some(expected_inner) = expected.ptr_inner() {
            if let AstType::Slice(actual_inner) = actual {
                return types_compatible(expected_inner, actual_inner);
            }
            if let AstType::FixedArray { element_type, .. } = actual {
                return types_compatible(expected_inner, element_type);
            }
        }
    }
    match (expected, actual) {
        // Check struct compatibility
        (
            AstType::Struct {
                name: expected_name,
                ..
            },
            AstType::Struct {
                name: actual_name, ..
            },
        ) => expected_name == actual_name,
        // Check enum compatibility
        (
            AstType::Enum {
                name: expected_name,
                ..
            },
            AstType::Enum {
                name: actual_name, ..
            },
        ) => expected_name == actual_name,
        // Allow Generic type to match Enum type when name matches (for type declarations)
        (
            AstType::Generic {
                name: expected_name,
                type_args,
            },
            AstType::Enum {
                name: actual_name, ..
            },
        ) => expected_name == actual_name && type_args.is_empty(),
        // Allow Enum type to match Generic type when name matches (for type declarations)
        (
            AstType::Enum {
                name: expected_name,
                ..
            },
            AstType::Generic {
                name: actual_name,
                type_args,
            },
        ) => expected_name == actual_name && type_args.is_empty(),
        // Allow struct type to be assigned to enum if the struct is one of the enum's variants
        (
            AstType::Enum { variants, .. },
            AstType::Struct {
                name: struct_name, ..
            },
        ) => variants.iter().any(|v| v.name == *struct_name),
        // Allow struct type to be assigned to generic enum type
        (
            AstType::Generic {
                name: _enum_name,
                type_args,
            },
            AstType::Struct {
                name: _struct_name, ..
            },
        ) if type_args.is_empty() => {
            // Permissive: allow struct-to-generic-enum assignment without lookup.
            // Full verification would require enum registry access here.
            true
        }
        // Option and Result are now Generic types - handled in Generic match below
        // Check Option<T> compatibility using generic syntax
        (
            AstType::Generic {
                name: expected_name,
                type_args: expected_args,
            },
            AstType::Generic {
                name: actual_name,
                type_args: actual_args,
            },
        ) if well_known().is_option(expected_name) && well_known().is_option(actual_name) => {
            expected_args.len() == actual_args.len()
                && expected_args
                    .iter()
                    .zip(actual_args.iter())
                    .all(|(e, a)| types_compatible(e, a))
        }
        // Check Result<T,E> compatibility using generic syntax
        (
            AstType::Generic {
                name: expected_name,
                type_args: expected_args,
            },
            AstType::Generic {
                name: actual_name,
                type_args: actual_args,
            },
        ) if well_known().is_result(expected_name) && well_known().is_result(actual_name) => {
            expected_args.len() == actual_args.len()
                && expected_args
                    .iter()
                    .zip(actual_args.iter())
                    .all(|(e, a)| types_compatible(e, a))
        }
        // Check range compatibility
        (
            AstType::Range {
                start_type: expected_start,
                end_type: expected_end,
                ..
            },
            AstType::Range {
                start_type: actual_start,
                end_type: actual_end,
                ..
            },
        ) => {
            types_compatible(expected_start, actual_start)
                && types_compatible(expected_end, actual_end)
        }
        // Function and FunctionPointer compatibility
        (
            AstType::Function {
                args: expected_args,
                return_type: expected_ret,
            },
            AstType::FunctionPointer {
                param_types: actual_params,
                return_type: actual_ret,
            },
        ) => {
            expected_args.len() == actual_params.len()
                && expected_args
                    .iter()
                    .zip(actual_params.iter())
                    .all(|(e, a)| types_compatible(e, a))
                && types_compatible(expected_ret, actual_ret)
        }
        (
            AstType::FunctionPointer {
                param_types: expected_params,
                return_type: expected_ret,
            },
            AstType::Function {
                args: actual_args,
                return_type: actual_ret,
            },
        ) => {
            expected_params.len() == actual_args.len()
                && expected_params
                    .iter()
                    .zip(actual_args.iter())
                    .all(|(e, a)| types_compatible(e, a))
                && types_compatible(expected_ret, actual_ret)
        }
        // Void is only compatible with void
        (AstType::Void, AstType::Void) => true,
        // All other combinations are incompatible
        _ => false,
    }
}

/// Validate that imports are not inside comptime blocks
pub fn validate_import_not_in_comptime(stmt: &crate::ast::Statement) -> Result<(), String> {
    use crate::ast::Statement;

    // Check if this is a ModuleImport statement
    if let Statement::ModuleImport { alias, module_path } = stmt {
        return Err(format!(
            "Import statement '{}' for module '{}' cannot be inside a comptime block. \
            Imports must be at module level.",
            alias, module_path
        ));
    }

    // Also check for variable declarations that look like imports
    if let Statement::VariableDeclaration {
        name,
        initializer: Some(expr),
        ..
    } = stmt
    {
        if contains_import_expression(expr) {
            return Err(format!(
                "Import-like statement '{}' cannot be inside a comptime block. \
                Imports must be at module level.",
                name
            ));
        }
    }

    // Check for nested comptime blocks that might contain imports
    if let Statement::ComptimeBlock {
        statements: nested_stmts,
        ..
    } = stmt
    {
        for nested_stmt in nested_stmts {
            validate_import_not_in_comptime(nested_stmt)?;
        }
    }

    Ok(())
}

/// Check if an expression contains import-related patterns
fn contains_import_expression(expr: &crate::ast::Expression) -> bool {
    match expr {
        crate::ast::Expression::Identifier(id) if id.starts_with("@std") => true,
        crate::ast::Expression::MemberAccess { object, .. } => {
            if let crate::ast::Expression::Identifier(id) = &**object {
                id.starts_with("@std") || id == "build"
            } else {
                contains_import_expression(object)
            }
        }
        crate::ast::Expression::FunctionCall { name, .. } if name.contains("import") => true,
        _ => false,
    }
}
