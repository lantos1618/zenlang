//! Standard Library Discovery Module
//!
//! This module provides centralized stdlib path discovery logic used across
//! the compiler, LSP server, and type system.
//!
//! The discovery strategy is:
//! 1. Check `ZEN_STDLIB_PATH` environment variable (runtime override)
//! 2. Try paths relative to the executable (for installed binaries)
//! 3. Try relative paths from CWD: `./stdlib`, `../stdlib`, `../../stdlib`
//! 4. Fall back to compile-time embedded path (from CARGO_MANIFEST_DIR)
//!
//! This ensures the compiler works both during development and when installed.

use std::path::{Path, PathBuf};

/// Compile-time embedded stdlib path - this is where stdlib was when the compiler was built.
/// This serves as the final fallback and makes development builds "just work".
const EMBEDDED_STDLIB_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/stdlib");

/// Find the stdlib root directory using the standard discovery strategy.
///
/// This function checks:
/// 1. `ZEN_STDLIB_PATH` environment variable
/// 2. Paths relative to the executable location (for installed compiler)
/// 3. Common relative paths from CWD (`./stdlib`, `../stdlib`, `../../stdlib`)
/// 4. Returns `None` if nothing found
///
/// # Returns
///
/// - `Some(PathBuf)` if a valid stdlib directory is found
/// - `None` if no stdlib directory exists at any of the checked locations
///
/// # Example
///
/// ```ignore
/// use zen::stdlib_discovery::find_stdlib_root;
///
/// if let Some(stdlib_path) = find_stdlib_root() {
///     println!("Found stdlib at: {}", stdlib_path.display());
/// } else {
///     eprintln!("No stdlib found");
/// }
/// ```
pub fn find_stdlib_root() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(path) = std::env::var("ZEN_STDLIB_PATH") {
        let p = PathBuf::from(path);
        if p.exists() && p.is_dir() {
            return Some(p);
        }
    }

    // Try paths relative to executable location
    // This handles cases where the compiler is run from outside the project directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check sibling stdlib (target/release/zen -> target/release/../stdlib doesn't work)
            // Check parent/stdlib (target/release/zen -> target/stdlib - unlikely)
            // Check parent/parent/stdlib (target/release/zen -> stdlib) - most likely for dev
            let exe_candidates = [
                exe_dir.join("stdlib"),       // exe_dir/stdlib
                exe_dir.join("../stdlib"),    // exe_dir/../stdlib
                exe_dir.join("../../stdlib"), // exe_dir/../../stdlib (for target/release/zen -> stdlib)
            ];
            for candidate in exe_candidates {
                if let Ok(canonical) = candidate.canonicalize() {
                    if canonical.is_dir() {
                        return Some(canonical);
                    }
                }
            }
        }
    }

    // Try relative paths from CWD (for running from project directory)
    let cwd_candidates = [
        PathBuf::from("./stdlib"),
        PathBuf::from("../stdlib"),
        PathBuf::from("../../stdlib"),
    ];

    for candidate in cwd_candidates {
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }
    }

    // Final fallback: compile-time embedded path
    // This makes development builds "just work" regardless of CWD
    let embedded = PathBuf::from(EMBEDDED_STDLIB_PATH);
    if embedded.exists() && embedded.is_dir() {
        return Some(embedded);
    }

    None
}

/// Find the stdlib root directory, always returning a valid path.
///
/// This function guarantees a return value. If no stdlib is found through
/// the normal discovery process, it falls back to the compile-time embedded path.
/// If even that doesn't exist, it prints a warning and returns the embedded path anyway.
///
/// # Returns
///
/// Always returns a `PathBuf` pointing to the stdlib directory.
///
/// # Example
///
/// ```ignore
/// use zen::stdlib_discovery::find_stdlib_root_or_default;
///
/// let stdlib_path = find_stdlib_root_or_default();
/// println!("Using stdlib at: {}", stdlib_path.display());
/// ```
pub fn find_stdlib_root_or_default() -> PathBuf {
    find_stdlib_root().unwrap_or_else(|| {
        // This should rarely happen since find_stdlib_root() now checks the embedded path
        // But if it does, provide helpful information
        let embedded = PathBuf::from(EMBEDDED_STDLIB_PATH);
        eprintln!(
            "Warning: Zen stdlib not found. Checked:\n\
             - ZEN_STDLIB_PATH environment variable\n\
             - Paths relative to executable\n\
             - ./stdlib, ../stdlib, ../../stdlib\n\
             - Compile-time path: {}\n\
             Set ZEN_STDLIB_PATH to override.",
            EMBEDDED_STDLIB_PATH
        );
        embedded
    })
}

/// Find stdlib root relative to a specific base path.
///
/// This function is useful for workspace-aware contexts (like LSP) where
/// stdlib might be located relative to the workspace root.
///
/// The discovery strategy is:
/// 1. `ZEN_STDLIB_PATH` environment variable
/// 2. `<base>/stdlib` if base is provided
/// 3. Common relative paths from current directory
/// 4. Returns `None` if nothing found
///
/// # Arguments
///
/// * `base` - Optional base path to search from (e.g., workspace root)
///
/// # Returns
///
/// - `Some(PathBuf)` if a valid stdlib directory is found
/// - `None` if no stdlib directory exists
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use zen::stdlib_discovery::find_stdlib_root_from;
///
/// let workspace = Path::new("/path/to/workspace");
/// if let Some(stdlib_path) = find_stdlib_root_from(Some(workspace)) {
///     println!("Found stdlib at: {}", stdlib_path.display());
/// }
/// ```
pub fn find_stdlib_root_from(base: Option<&Path>) -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(path) = std::env::var("ZEN_STDLIB_PATH") {
        let p = PathBuf::from(path);
        if p.exists() && p.is_dir() {
            return Some(p);
        }
    }

    // If base path provided, try base/stdlib first
    if let Some(base_path) = base {
        let candidate = base_path.join("stdlib");
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }
    }

    // Try common relative paths from CWD
    let cwd_candidates = [
        PathBuf::from("./stdlib"),
        PathBuf::from("../stdlib"),
        PathBuf::from("../../stdlib"),
    ];

    for candidate in cwd_candidates {
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }
    }

    // Final fallback: compile-time embedded path
    let embedded = PathBuf::from(EMBEDDED_STDLIB_PATH);
    if embedded.exists() && embedded.is_dir() {
        return Some(embedded);
    }

    None
}

/// Find stdlib root relative to a base path, with a default fallback.
///
/// Same as `find_stdlib_root_from` but guarantees a return value.
/// Falls back to the compile-time embedded path if no stdlib is found.
///
/// # Arguments
///
/// * `base` - Optional base path to search from
///
/// # Returns
///
/// Always returns a `PathBuf` pointing to the stdlib directory.
pub fn find_stdlib_root_from_or_default(base: Option<&Path>) -> PathBuf {
    find_stdlib_root_from(base).unwrap_or_else(|| {
        let embedded = PathBuf::from(EMBEDDED_STDLIB_PATH);
        eprintln!(
            "Warning: Zen stdlib not found. Using compile-time path: {}\n\
             Set ZEN_STDLIB_PATH environment variable to override.",
            EMBEDDED_STDLIB_PATH
        );
        embedded
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_stdlib_root_respects_env_var() {
        // Note: This test would need actual setup to work properly
        // Just checking the logic compiles
        let _ = find_stdlib_root();
    }

    #[test]
    fn test_find_stdlib_root_or_default_always_returns() {
        let result = find_stdlib_root_or_default();
        assert!(!result.as_os_str().is_empty());
    }

    #[test]
    fn test_find_stdlib_root_from_with_base() {
        let base = Path::new("/some/workspace");
        let _ = find_stdlib_root_from(Some(base));
    }

    #[test]
    fn test_find_stdlib_root_from_or_default_always_returns() {
        let result = find_stdlib_root_from_or_default(None);
        assert!(!result.as_os_str().is_empty());
    }
}
