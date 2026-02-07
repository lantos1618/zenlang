use crate::ast::{AstType, Expression, Pattern as AstPattern, PatternArm, Statement};
use crate::name_utils;
use crate::stdlib_types::{stdlib_types, StdlibTypeRegistry};
use crate::well_known::well_known;
use std::collections::{HashMap, HashSet};

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
        // Allow Generic type to match Struct type when name matches (for return types)
        (
            AstType::Generic {
                name: expected_name,
                type_args,
            },
            AstType::Struct {
                name: actual_name, ..
            },
        ) => expected_name == actual_name && type_args.is_empty(),
        // Allow Struct type to match Generic type when name matches
        (
            AstType::Struct {
                name: expected_name,
                ..
            },
            AstType::Generic {
                name: actual_name,
                type_args,
            },
        ) => expected_name == actual_name && type_args.is_empty(),
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

#[derive(Debug, Clone, PartialEq)]
pub struct AllocatorViolation {
    /// Full call name (e.g. "Vec<i32>.new") — used by LSP to locate in source text
    pub call_name: String,
    /// Base type name (e.g. "Vec") — used in diagnostic messages
    pub type_name: String,
}

pub fn check_allocator_violations(statements: &[Statement]) -> Vec<AllocatorViolation> {
    let mut violations = Vec::new();
    collect_allocator_violations_in_stmts(statements, &mut violations);
    violations
}

fn collect_allocator_violations_in_stmts(
    statements: &[Statement],
    violations: &mut Vec<AllocatorViolation>,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression { expr, .. } | Statement::Return { expr, .. } => {
                collect_allocator_violations_in_expr(expr, violations);
            }
            Statement::VariableDeclaration {
                initializer: Some(expr),
                ..
            }
            | Statement::VariableAssignment { value: expr, .. } => {
                collect_allocator_violations_in_expr(expr, violations);
            }
            _ => {}
        }
    }
}

fn collect_allocator_violations_in_expr(
    expr: &Expression,
    violations: &mut Vec<AllocatorViolation>,
) {
    match expr {
        Expression::FunctionCall { name, args, .. } => {
            let base_name = name_utils::strip_generics(name);

            let requires_alloc = stdlib_types().requires_allocator(base_name);

            if requires_alloc && (args.is_empty() || !has_allocator_arg(args)) {
                violations.push(AllocatorViolation {
                    call_name: name.clone(),
                    type_name: base_name.to_string(),
                });
            }
            for arg in args {
                collect_allocator_violations_in_expr(arg, violations);
            }
        }
        Expression::MethodCall { object, args, .. } => {
            collect_allocator_violations_in_expr(object, violations);
            for arg in args {
                collect_allocator_violations_in_expr(arg, violations);
            }
        }
        Expression::Block(stmts) => {
            collect_allocator_violations_in_stmts(stmts, violations);
        }
        Expression::Conditional { scrutinee, arms } => {
            collect_allocator_violations_in_expr(scrutinee, violations);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_allocator_violations_in_expr(guard, violations);
                }
                collect_allocator_violations_in_expr(&arm.body, violations);
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            collect_allocator_violations_in_expr(left, violations);
            collect_allocator_violations_in_expr(right, violations);
        }
        _ => {}
    }
}

fn has_allocator_arg(args: &[Expression]) -> bool {
    for arg in args {
        match arg {
            Expression::FunctionCall { name, .. } => {
                if name.contains("allocator") || name == "get_default_allocator" {
                    return true;
                }
            }
            Expression::Identifier(name) => {
                if name.contains("alloc") || name.ends_with("_allocator") || name == "allocator" {
                    return true;
                }
            }
            Expression::MethodCall { object, method, .. } => {
                if method.contains("allocator") || method == "get_allocator" {
                    return true;
                }
                if has_allocator_arg(&[(**object).clone()]) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternExhaustivenessViolation {
    pub enum_type: String,
    pub missing_variants: Vec<String>,
    /// Retained for LSP position lookup — the compiler itself doesn't use this.
    pub scrutinee: Expression,
}

/// Pure pattern exhaustiveness check.  `enum_registry` maps enum names to
/// their variant names; well-known types (Option, Result) are built-in.
pub fn check_pattern_exhaustiveness(
    statements: &[Statement],
    enum_registry: &HashMap<String, Vec<String>>,
    infer_type: &impl Fn(&Expression) -> Option<String>,
) -> Vec<PatternExhaustivenessViolation> {
    let mut violations = Vec::new();
    collect_exhaustiveness_in_stmts(statements, enum_registry, infer_type, &mut violations, 0);
    violations
}

const MAX_PATTERN_DEPTH: usize = 50;

fn collect_exhaustiveness_in_stmts(
    statements: &[Statement],
    enum_registry: &HashMap<String, Vec<String>>,
    infer_type: &impl Fn(&Expression) -> Option<String>,
    violations: &mut Vec<PatternExhaustivenessViolation>,
    depth: usize,
) {
    if depth > MAX_PATTERN_DEPTH {
        return;
    }
    for stmt in statements {
        match stmt {
            Statement::Expression { expr, .. } | Statement::Return { expr, .. } => {
                collect_exhaustiveness_in_expr(expr, enum_registry, infer_type, violations, depth);
            }
            Statement::VariableDeclaration {
                initializer: Some(expr),
                ..
            }
            | Statement::VariableAssignment { value: expr, .. } => {
                collect_exhaustiveness_in_expr(expr, enum_registry, infer_type, violations, depth);
            }
            _ => {}
        }
    }
}

fn collect_exhaustiveness_in_expr(
    expr: &Expression,
    enum_registry: &HashMap<String, Vec<String>>,
    infer_type: &impl Fn(&Expression) -> Option<String>,
    violations: &mut Vec<PatternExhaustivenessViolation>,
    depth: usize,
) {
    match expr {
        Expression::PatternMatch { scrutinee, arms } => {
            if let Some(scrutinee_type) = infer_type(scrutinee) {
                let missing = find_missing_variants_pure(&scrutinee_type, arms, enum_registry);
                if !missing.is_empty() {
                    violations.push(PatternExhaustivenessViolation {
                        enum_type: scrutinee_type,
                        missing_variants: missing,
                        scrutinee: (**scrutinee).clone(),
                    });
                }
            }
            collect_exhaustiveness_in_expr(scrutinee, enum_registry, infer_type, violations, depth);
            for arm in arms {
                collect_exhaustiveness_in_expr(
                    &arm.body,
                    enum_registry,
                    infer_type,
                    violations,
                    depth,
                );
            }
        }
        Expression::Block(stmts) => {
            collect_exhaustiveness_in_stmts(
                stmts,
                enum_registry,
                infer_type,
                violations,
                depth + 1,
            );
        }
        Expression::Conditional { scrutinee, arms } => {
            collect_exhaustiveness_in_expr(scrutinee, enum_registry, infer_type, violations, depth);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_exhaustiveness_in_expr(
                        guard,
                        enum_registry,
                        infer_type,
                        violations,
                        depth,
                    );
                }
                collect_exhaustiveness_in_expr(
                    &arm.body,
                    enum_registry,
                    infer_type,
                    violations,
                    depth,
                );
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            collect_exhaustiveness_in_expr(left, enum_registry, infer_type, violations, depth);
            collect_exhaustiveness_in_expr(right, enum_registry, infer_type, violations, depth);
        }
        _ => {}
    }
}

pub fn find_missing_variants_pure(
    scrutinee_type: &str,
    arms: &[PatternArm],
    enum_registry: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let wk = well_known();

    let base_type = name_utils::strip_generics(scrutinee_type).to_string();
    let known_variants: Vec<String> = if wk.is_option(&base_type) {
        vec![wk.some_name().to_string(), wk.none_name().to_string()]
    } else if wk.is_result(&base_type) {
        vec![wk.ok_name().to_string(), wk.err_name().to_string()]
    } else {
        let enum_name = name_utils::base_name(&base_type).trim().to_string();
        match enum_registry.get(&enum_name) {
            Some(variants) => variants.clone(),
            None => return Vec::new(),
        }
    };
    let mut covered = HashSet::new();
    let mut has_wildcard = false;

    for arm in arms {
        collect_covered_variants(&arm.pattern, &mut covered, &mut has_wildcard);
    }

    if has_wildcard {
        return Vec::new();
    }

    known_variants
        .into_iter()
        .filter(|v| !covered.contains(v))
        .collect()
}

fn collect_covered_variants(
    pattern: &AstPattern,
    covered: &mut HashSet<String>,
    has_wildcard: &mut bool,
) {
    match pattern {
        AstPattern::EnumVariant { variant, .. } => {
            covered.insert(variant.clone());
        }
        AstPattern::EnumLiteral { variant, .. } => {
            covered.insert(variant.clone());
        }
        AstPattern::Wildcard => {
            *has_wildcard = true;
        }
        AstPattern::Or(pats) => {
            for p in pats {
                collect_covered_variants(p, covered, has_wildcard);
            }
        }
        AstPattern::Guard { pattern, .. } => {
            collect_covered_variants(pattern, covered, has_wildcard);
        }
        _ => {}
    }
}
