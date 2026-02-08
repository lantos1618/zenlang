//! Utilities for parsing qualified names.
//!
//! Zen uses two name separators:
//! - `::` for module paths (e.g., `std::io::File`)
//! - `.`  for method/member access (e.g., `Vec.len`, `io.print`)
//!
//! This module provides a single source of truth for splitting these names,
//! replacing 6+ ad-hoc implementations scattered across the codebase.

/// Split a `::` qualified name into (module, name).
/// Returns `None` if there's no `::` separator.
pub fn split_module_path(name: &str) -> Option<(&str, &str)> {
    name.split_once("::")
}

/// Split a `.` qualified name into (receiver, member).
/// Returns `None` if there's no `.` separator.
pub fn split_method_path(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

/// Get just the base name (before any `::` qualifier).
pub fn base_name(name: &str) -> &str {
    name.split("::").next().unwrap_or(name)
}

/// Get the leaf name (after the last `::` qualifier).
pub fn leaf_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

/// Strip generic type args from a name: `"Vec<i32>"` -> `"Vec"`.
pub fn strip_generics(name: &str) -> &str {
    match name.find('<') {
        Some(pos) => &name[..pos],
        None => name,
    }
}

/// Normalize a UFC method name by stripping generic type parameters from the type portion.
/// This ensures consistent method key generation regardless of how generics are written.
///
/// Examples:
/// - `"SafePtr<T>.is_valid"` → `"SafePtr.is_valid"`
/// - `"HashMap<K, V>.get"` → `"HashMap.get"`
/// - `"Foo<Bar<T>>.method"` → `"Foo.method"` (nested generics)
/// - `"plain_function"` → `"plain_function"` (no dot, no change)
/// - `"Point.distance"` → `"Point.distance"` (no generics, no change)
pub fn normalize_ufc_name(func_name: &str) -> String {
    // If there's no dot, it's not a UFC method - return as-is
    if !func_name.contains('.') {
        return func_name.to_string();
    }

    // Split on the first dot to get type_name and method_name
    if let Some((type_part, method_part)) = func_name.split_once('.') {
        // Strip generics from the type part
        let normalized_type = strip_generics(type_part);
        format!("{}.{}", normalized_type, method_part)
    } else {
        func_name.to_string()
    }
}

/// Construct a method key: `"TypeName.method"`.
/// This is the canonical format used across TypeContext, TypeStore, and codegen.
/// Always use this instead of ad-hoc `format!("{}.{}", ...)` calls.
#[inline]
pub fn method_key(type_name: &str, method_name: &str) -> String {
    format!("{}.{}", type_name, method_name)
}

/// Construct a scoped variable key: `"scope::var_name"`.
/// Used for per-function variable tracking in TypeStore and TypeContext.
#[inline]
pub fn scoped_var_key(scope: &str, var_name: &str) -> String {
    format!("{}::{}", scope, var_name)
}

/// Construct a stdlib function key: `"module::func_name"`.
/// Used for stdlib function signature lookup in TypeStore.
#[inline]
pub fn stdlib_func_key(module: &str, func_name: &str) -> String {
    format!("{}::{}", module, func_name)
}

/// Parse a generic type string into (base_name, type_args).
/// e.g. `"Vec<i32>"` -> `("Vec", ["i32"])`, `"i32"` -> `("i32", [])`.
pub fn parse_generic_type(type_str: &str) -> (String, Vec<String>) {
    let (name, type_args) = crate::parser::parse_generic_type_string(type_str);
    let args = type_args
        .iter()
        .map(crate::lsp::utils::format_type)
        .collect();
    (name, args)
}

/// Check if a name follows Zen's test naming convention.
/// Matches: `test_foo`, `foo_test`, `foo_test_bar`
pub fn is_test_name(name: &str) -> bool {
    name.starts_with("test_") || name.ends_with("_test") || name.contains("_test_")
}

/// Check if a file path looks like a test file.
/// Matches: `test_foo.zen`, `foo_test.zen`
pub fn is_test_file(file_name: &str) -> bool {
    let stem = file_name.strip_suffix(".zen").unwrap_or(file_name);
    is_test_name(stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_module_path() {
        assert_eq!(split_module_path("std::io"), Some(("std", "io")));
        assert_eq!(
            split_module_path("std::io::File"),
            Some(("std", "io::File"))
        );
        assert_eq!(split_module_path("File"), None);
    }

    #[test]
    fn test_split_method_path() {
        assert_eq!(split_method_path("Vec.len"), Some(("Vec", "len")));
        assert_eq!(split_method_path("io.print"), Some(("io", "print")));
        assert_eq!(split_method_path("print"), None);
    }

    #[test]
    fn test_base_name() {
        assert_eq!(base_name("std::io::File"), "std");
        assert_eq!(base_name("File"), "File");
    }

    #[test]
    fn test_leaf_name() {
        assert_eq!(leaf_name("std::io::File"), "File");
        assert_eq!(leaf_name("File"), "File");
    }

    #[test]
    fn test_method_key() {
        assert_eq!(method_key("Vec", "len"), "Vec.len");
        assert_eq!(method_key("HashMap", "new"), "HashMap.new");
    }

    #[test]
    fn test_scoped_var_key() {
        assert_eq!(scoped_var_key("main", "x"), "main::x");
        assert_eq!(
            scoped_var_key("MyStruct.method", "self"),
            "MyStruct.method::self"
        );
    }

    #[test]
    fn test_stdlib_func_key() {
        assert_eq!(stdlib_func_key("math", "sqrt"), "math::sqrt");
        assert_eq!(stdlib_func_key("io", "print"), "io::print");
    }

    #[test]
    fn test_method_key_roundtrip() {
        let key = method_key("Vec", "push");
        assert_eq!(split_method_path(&key), Some(("Vec", "push")));
    }

    #[test]
    fn test_strip_generics() {
        assert_eq!(strip_generics("Vec<i32>"), "Vec");
        assert_eq!(strip_generics("HashMap<String, i32>"), "HashMap");
        assert_eq!(strip_generics("i32"), "i32");
    }

    #[test]
    fn test_normalize_ufc_name() {
        // Generic types with methods
        assert_eq!(
            normalize_ufc_name("SafePtr<T>.is_valid"),
            "SafePtr.is_valid"
        );
        assert_eq!(normalize_ufc_name("HashMap<K, V>.get"), "HashMap.get");
        assert_eq!(normalize_ufc_name("Vec<i32>.push"), "Vec.push");

        // Nested generics
        assert_eq!(normalize_ufc_name("Foo<Bar<T>>.method"), "Foo.method");
        assert_eq!(
            normalize_ufc_name("Result<Vec<String>, Error>.unwrap"),
            "Result.unwrap"
        );

        // Non-generic types with methods
        assert_eq!(normalize_ufc_name("Point.distance"), "Point.distance");
        assert_eq!(normalize_ufc_name("String.len"), "String.len");

        // Plain functions (no dot)
        assert_eq!(normalize_ufc_name("plain_function"), "plain_function");
        assert_eq!(normalize_ufc_name("add"), "add");

        // Edge cases
        assert_eq!(normalize_ufc_name("T.method"), "T.method");
        assert_eq!(normalize_ufc_name("Option<T>.unwrap"), "Option.unwrap");
    }
}
