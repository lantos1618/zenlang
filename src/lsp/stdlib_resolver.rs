// Stdlib Module Resolver for LSP
// Handles resolution of @std module paths to actual files

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolve @std module paths to file paths
/// Supports:
/// - @std.io -> stdlib/io/io.zen or stdlib/io/mod.zen
/// - @std.collections.hashmap -> stdlib/collections/hashmap.zen
/// - @std.collections -> stdlib/collections/mod.zen
#[derive(Clone)]
pub struct StdlibResolver {
    pub stdlib_root: PathBuf,
    module_cache: HashMap<String, PathBuf>,
}

impl StdlibResolver {
    pub fn new(workspace_root: Option<&Path>) -> Self {
        // Find stdlib directory
        let stdlib_root = Self::find_stdlib_root(workspace_root);

        Self {
            stdlib_root,
            module_cache: HashMap::new(),
        }
    }

    fn find_stdlib_root(workspace_root: Option<&Path>) -> PathBuf {
        // Check environment variable first
        if let Ok(path) = std::env::var("ZEN_STDLIB_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() && p.is_dir() {
                return p;
            }
        }

        // Try workspace-relative and common locations
        let candidates = vec![
            workspace_root.map(|p| p.join("stdlib")),
            Some(PathBuf::from("./stdlib")),
            Some(PathBuf::from("../stdlib")),
            Some(PathBuf::from("../../stdlib")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() && candidate.is_dir() {
                return candidate;
            }
        }

        // Default fallback
        PathBuf::from("./stdlib")
    }

    /// Resolve a module path like "@std.io" or "@std.collections.hashmap" to a file path
    pub fn resolve_module_path(&self, module_path: &str) -> Option<PathBuf> {
        // Check cache first
        if let Some(cached) = self.module_cache.get(module_path) {
            if cached.exists() {
                return Some(cached.clone());
            }
        }

        // Remove @std prefix
        let path = if let Some(stripped) = module_path.strip_prefix("@std.") {
            stripped
        } else if module_path == "@std" {
            return None; // @std itself doesn't resolve to a file
        } else {
            module_path
        };

        let parts: Vec<&str> = path.split('.').collect();

        // Try different resolution strategies
        self.try_resolve_path(&parts)
    }

    fn try_resolve_path(&self, parts: &[&str]) -> Option<PathBuf> {
        if parts.is_empty() {
            return None;
        }

        // Strategy 1: Single module like "io" -> stdlib/io/io.zen or stdlib/io/mod.zen
        if parts.len() == 1 {
            let module_name = parts[0];
            let module_dir = self.stdlib_root.join(module_name);

            // Try module_name.zen in the directory
            let file_path = module_dir.join(format!("{}.zen", module_name));
            if file_path.exists() {
                return Some(file_path);
            }

            // Try mod.zen in the directory
            let mod_path = module_dir.join("mod.zen");
            if mod_path.exists() {
                return Some(mod_path);
            }

            // Try as a single file stdlib/module_name.zen
            let single_file = self.stdlib_root.join(format!("{}.zen", module_name));
            if single_file.exists() {
                return Some(single_file);
            }
        }

        // Strategy 2: Nested path like "collections.hashmap" -> stdlib/collections/hashmap.zen
        if parts.len() >= 2 {
            let folder = parts[0];
            let file = parts[parts.len() - 1];

            // Try stdlib/folder/file.zen
            let file_path = self.stdlib_root.join(folder).join(format!("{}.zen", file));
            if file_path.exists() {
                return Some(file_path);
            }

            // Try stdlib/folder/mod.zen (for folder-level modules)
            if parts.len() == 2 {
                let mod_path = self.stdlib_root.join(folder).join("mod.zen");
                if mod_path.exists() {
                    return Some(mod_path);
                }
            }
        }

        // Strategy 3: Deeply nested paths
        // Build path progressively: stdlib/part1/part2/.../partN.zen
        let mut current_path = self.stdlib_root.clone();
        for (i, part) in parts.iter().enumerate() {
            current_path = current_path.join(part);

            // If this is the last part, try as a file
            if i == parts.len() - 1 {
                let file_path = current_path.with_extension("zen");
                if file_path.exists() {
                    return Some(file_path);
                }
            }
        }

        // Strategy 4: Try mod.zen at the deepest level
        let mod_path = current_path.join("mod.zen");
        if mod_path.exists() {
            return Some(mod_path);
        }

        None
    }

    /// Get all available modules in stdlib (for completion)
    pub fn list_modules(&self) -> Vec<String> {
        let mut modules = Vec::new();

        if !self.stdlib_root.exists() {
            return modules;
        }

        Self::scan_directory(&self.stdlib_root, &mut modules, "");

        modules
    }

    /// Scan a directory for .zen files and subdirectories (public for use in completion)
    pub fn scan_directory(dir: &Path, modules: &mut Vec<String>, prefix: &str) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("zen")) {
                    // Found a .zen file
                    if name == "mod" {
                        // mod.zen - add the directory as a module
                        if let Some(dir_name) = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                        {
                            let module_path = if prefix.is_empty() {
                                dir_name.to_string()
                            } else {
                                format!("{}.{}", prefix, dir_name)
                            };
                            modules.push(module_path);
                        }
                    } else {
                        // Regular .zen file
                        let module_path = if prefix.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}.{}", prefix, name)
                        };
                        modules.push(module_path);
                    }
                } else if path.is_dir() {
                    // Recursively scan subdirectories
                    let new_prefix = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}.{}", prefix, name)
                    };
                    Self::scan_directory(&path, modules, &new_prefix);
                }
            }
        }
    }

    /// Convert a file path to a module path (for go-to-definition)
    pub fn path_to_module_path(&self, file_path: &Path) -> Option<String> {
        // Check if the path is within stdlib_root
        if let Ok(relative) = file_path.strip_prefix(&self.stdlib_root) {
            let mut parts = Vec::new();

            for component in relative.components() {
                if let std::path::Component::Normal(name) = component {
                    if let Some(name_str) = name.to_str() {
                        // Remove .zen extension if present
                        let name = name_str.strip_suffix(".zen").unwrap_or(name_str);

                        // Skip "mod" files - they represent the directory
                        if name != "mod" {
                            parts.push(name.to_string());
                        }
                    }
                }
            }

            if !parts.is_empty() {
                return Some(format!("@std.{}", parts.join(".")));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::lsp::stdlib_resolver::StdlibResolver;

    #[test]
    fn test_resolve_simple_module() {
        let _resolver = StdlibResolver::new(None);
        // This test would need actual stdlib files to work
        // Just testing the logic structure
    }
}
