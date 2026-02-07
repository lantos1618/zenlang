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
    fn test_strip_generics() {
        assert_eq!(strip_generics("Vec<i32>"), "Vec");
        assert_eq!(strip_generics("HashMap<String, i32>"), "HashMap");
        assert_eq!(strip_generics("i32"), "i32");
    }
}
