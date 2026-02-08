//! Build system for Zen projects.
//!
//! Implements `build.zen` discovery and declarative parsing to create a `PackageMap`
//! that tells the module system where packages come from.
//!
//! Design:
//! - `build.zen` in project root declares packages: `std = @builtin.import_std()`
//! - Individual files reference packages by name: `{ println } = std.io`
//! - `@` prefix reserved for `@builtin` raw intrinsics (only in compiler.zen)
//! - If no `build.zen` exists, an implicit PackageMap with `"std" => Stdlib` is used

use crate::error::CompileError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where a package's source code comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    /// The Zen standard library (resolved via stdlib search paths).
    Stdlib,
    /// A local directory relative to the project root.
    Local(PathBuf),
    /// A remote package (future: git URL + version).
    Remote {
        url: String,
        version: Option<String>,
    },
}

/// Maps package names to their source locations.
#[derive(Debug, Clone)]
pub struct PackageMap {
    pub packages: HashMap<String, PackageSource>,
}

impl Default for PackageMap {
    fn default() -> Self {
        // By default, `std` maps to stdlib — this is the implicit config
        // when no build.zen is present.
        let mut packages = HashMap::new();
        packages.insert("std".to_string(), PackageSource::Stdlib);
        PackageMap { packages }
    }
}

impl PackageMap {
    /// Look up which package a module path belongs to.
    /// Given "std.io.io", returns Some(("std", PackageSource::Stdlib, "io.io")).
    pub fn resolve<'a>(
        &'a self,
        module_path: &'a str,
    ) -> Option<(&'a str, &'a PackageSource, &'a str)> {
        for (name, source) in &self.packages {
            if module_path == name.as_str() {
                return Some((name.as_str(), source, ""));
            }
            if let Some(rest) = module_path.strip_prefix(&format!("{}.", name)) {
                return Some((name.as_str(), source, rest));
            }
        }
        None
    }
}

/// Build configuration parsed from `build.zen`.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub packages: PackageMap,
    pub project_root: PathBuf,
    /// Executable targets declared in build.zen (name → root source path).
    pub executables: Vec<ExecutableTarget>,
}

/// A declared executable target.
#[derive(Debug, Clone)]
pub struct ExecutableTarget {
    pub name: String,
    pub root: String,
}

impl BuildConfig {
    /// Walk up from `start_path` looking for `build.zen`.
    /// Returns `Ok(None)` if no build.zen is found (single-file mode).
    /// Returns `Err` if build.zen exists but fails to parse.
    pub fn discover(start_path: &Path) -> Result<Option<BuildConfig>, CompileError> {
        let start = if start_path.is_file() {
            start_path.parent()
        } else {
            Some(start_path)
        };
        let Some(start) = start else {
            return Ok(None);
        };

        let mut dir = start.to_path_buf();
        loop {
            let build_zen = dir.join("build.zen");
            if build_zen.exists() {
                return Self::parse_build_file(&build_zen, &dir).map(Some);
            }
            if !dir.pop() {
                break;
            }
        }
        Ok(None)
    }

    /// Parse a build.zen file declaratively.
    ///
    /// We scan top-level assignments for patterns like:
    /// - `std = @builtin.import_std()`   → PackageSource::Stdlib
    /// - `http = @builtin.import("url")` → PackageSource::Remote { url }
    /// - `utils = @builtin.import("./lib")` → PackageSource::Local(path)
    ///
    /// This is NOT executing build.zen — just reading declarations.
    fn parse_build_file(path: &Path, project_root: &Path) -> Result<BuildConfig, CompileError> {
        let source = std::fs::read_to_string(path).map_err(|e| {
            CompileError::BuildError(format!("Failed to read {}: {}", path.display(), e), None)
        })?;
        let mut packages = HashMap::new();
        let mut executables = Vec::new();

        // Use the Zen lexer+parser to parse the file
        let lexer = crate::lexer::Lexer::new(&source);
        let mut parser = crate::parser::Parser::new(lexer);
        let program = parser.parse_program().map_err(|e| {
            CompileError::BuildError(format!("Failed to parse {}: {}", path.display(), e), None)
        })?;

        // Scan top-level declarations for package imports
        for decl in &program.declarations {
            match decl {
                // Handle: name = @builtin.import_std() or name = @builtin.import("url")
                crate::ast::Declaration::Function(_func) => {
                    // Functions with empty body that are actually assignments
                    // won't appear here — they're parsed as functions
                }
                crate::ast::Declaration::Constant { name, value, .. } => {
                    // Handle: std = @builtin.import_std()
                    if let Some(source) = extract_package_source(value, project_root) {
                        packages.insert(name.clone(), source);
                    }
                }
                _ => {}
            }
        }

        // Also scan top-level statements for variable declarations
        for stmt in &program.statements {
            if let crate::ast::Statement::VariableDeclaration {
                name,
                initializer: Some(init),
                ..
            } = stmt
            {
                if let Some(source) = extract_package_source(init, project_root) {
                    packages.insert(name.clone(), source);
                }
            }
        }

        // Also look for simple `name = expr` patterns parsed as functions
        // with body that is a single expression
        for decl in &program.declarations {
            if let crate::ast::Declaration::Function(func) = decl {
                // Check if the function body contains b.add_executable calls
                // For now, just scan function bodies for executable declarations
                scan_for_executables(&func.body, &mut executables);
            }
        }

        // If no explicit package declarations found, this build.zen might use
        // the old @std style — still return a config with default std mapping
        if packages.is_empty() {
            packages.insert("std".to_string(), PackageSource::Stdlib);
        }

        Ok(BuildConfig {
            packages: PackageMap { packages },
            project_root: project_root.to_path_buf(),
            executables,
        })
    }

    /// Create a default BuildConfig for single-file mode (no build.zen).
    pub fn default_config() -> BuildConfig {
        BuildConfig {
            packages: PackageMap::default(),
            project_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            executables: Vec::new(),
        }
    }
}

/// Extract a PackageSource from an expression like `@builtin.import_std()`
/// or `@builtin.import("url")`.
fn extract_package_source(
    expr: &crate::ast::Expression,
    project_root: &Path,
) -> Option<PackageSource> {
    use crate::ast::Expression;

    match expr {
        Expression::FunctionCall { name, args, .. } => {
            // Handle @builtin.import_std() → Stdlib
            if name == "builtin.import_std" || name == "import_std" {
                return Some(PackageSource::Stdlib);
            }

            // Handle @builtin.import("url") → Remote or Local
            if name == "builtin.import" || name == "import" {
                if let Some(Expression::String(url)) = args.first() {
                    if url.starts_with("./") || url.starts_with("../") {
                        return Some(PackageSource::Local(project_root.join(url)));
                    }
                    return Some(PackageSource::Remote {
                        url: url.clone(),
                        version: None,
                    });
                }
            }

            None
        }
        // Handle member access: @builtin.import_std() parsed as MemberAccess + call
        Expression::MemberAccess { .. } => {
            // This handles cases where @builtin.import_std is parsed as
            // a member access on @builtin
            None
        }
        _ => None,
    }
}

/// Scan function body statements for executable target declarations.
fn scan_for_executables(stmts: &[crate::ast::Statement], executables: &mut Vec<ExecutableTarget>) {
    for stmt in stmts {
        if let crate::ast::Statement::Expression { expr, .. } = stmt {
            scan_expr_for_executables(expr, executables);
        }
        if let crate::ast::Statement::VariableDeclaration {
            initializer: Some(init),
            ..
        } = stmt
        {
            scan_expr_for_executables(init, executables);
        }
    }
}

fn scan_expr_for_executables(
    expr: &crate::ast::Expression,
    executables: &mut Vec<ExecutableTarget>,
) {
    use crate::ast::Expression;

    if let Expression::FunctionCall { name, args, .. } = expr {
        // Look for b.add_executable({ name: "myapp", root: "src/main.zen" })
        // or add_executable(b, { name: ..., root: ... })
        if name.contains("add_executable") {
            for arg in args {
                if let Expression::StructLiteral { fields, .. } = arg {
                    let mut exe_name = None;
                    let mut exe_root = None;
                    for (field_name, field_val) in fields {
                        if field_name == "name" {
                            if let Expression::String(s) = field_val {
                                exe_name = Some(s.clone());
                            }
                        }
                        if field_name == "root" || field_name == "root_source_file" {
                            if let Expression::String(s) = field_val {
                                exe_root = Some(s.clone());
                            }
                        }
                    }
                    if let (Some(name), Some(root)) = (exe_name, exe_root) {
                        executables.push(ExecutableTarget { name, root });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_package_map() {
        let map = PackageMap::default();
        assert_eq!(map.packages.get("std"), Some(&PackageSource::Stdlib));
    }

    #[test]
    fn test_package_map_resolve() {
        let map = PackageMap::default();

        // "std" alone
        let (name, source, rest) = map.resolve("std").unwrap();
        assert_eq!(name, "std");
        assert_eq!(source, &PackageSource::Stdlib);
        assert_eq!(rest, "");

        // "std.io"
        let (name, source, rest) = map.resolve("std.io").unwrap();
        assert_eq!(name, "std");
        assert_eq!(source, &PackageSource::Stdlib);
        assert_eq!(rest, "io");

        // "std.core.result"
        let (name, source, rest) = map.resolve("std.core.result").unwrap();
        assert_eq!(name, "std");
        assert_eq!(source, &PackageSource::Stdlib);
        assert_eq!(rest, "core.result");

        // Unknown package
        assert!(map.resolve("http.server").is_none());
    }

    #[test]
    fn test_default_config() {
        let config = BuildConfig::default_config();
        assert!(config.packages.packages.contains_key("std"));
        assert!(config.executables.is_empty());
    }

    #[test]
    fn test_package_map_with_multiple_packages() {
        let mut packages = HashMap::new();
        packages.insert("std".to_string(), PackageSource::Stdlib);
        packages.insert(
            "http".to_string(),
            PackageSource::Remote {
                url: "github.com/someone/zen-http".to_string(),
                version: None,
            },
        );
        packages.insert(
            "utils".to_string(),
            PackageSource::Local(PathBuf::from("./lib")),
        );
        let map = PackageMap { packages };

        assert!(map.resolve("std.io").is_some());
        assert!(map.resolve("http.server").is_some());
        assert!(map.resolve("utils.helpers").is_some());
        assert!(map.resolve("unknown.thing").is_none());
    }
}
