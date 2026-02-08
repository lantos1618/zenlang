pub mod resolver;

use crate::ast::{Declaration, Program};
use crate::build_system::{PackageMap, PackageSource};
use crate::error::CompileError;
use crate::parser::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

/// Maximum number of cached modules before eviction
const MAX_CACHED_MODULES: usize = 200;

/// Cached module entry with content hash for invalidation
#[derive(Clone)]
struct CachedModule {
    program: Program,
    content_hash: u64,
    /// Insertion order for oldest-first eviction
    insertion_order: u64,
}

/// Fast hash for content comparison (FNV-1a)
fn hash_content(content: &str) -> u64 {
    const FNV_PRIME: u64 = 0x00000100000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;

    let mut hash = FNV_OFFSET;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Module system for Zen language
pub struct ModuleSystem {
    /// Map from module paths to their cached AST with content hash
    modules: HashMap<String, CachedModule>,
    /// Search paths for modules
    search_paths: Vec<PathBuf>,
    /// Current working directory
    #[allow(dead_code)] // Stored for future use
    cwd: PathBuf,
    /// Monotonic counter for insertion ordering (used for eviction)
    insertion_counter: u64,
    /// Stack of modules currently being loaded (for circular import detection)
    loading_stack: Vec<String>,
    /// Package map from build.zen (maps package names to sources)
    package_map: Option<PackageMap>,
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleSystem {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|e| {
            // current_dir() can fail if the directory was deleted or permissions changed
            // Fallback to "." but warn the user as this may cause module resolution issues
            eprintln!(
                "Warning: Could not determine current directory ({}), using '.' as fallback",
                e
            );
            PathBuf::from(".")
        });

        let mut search_paths = vec![cwd.clone(), cwd.join("lib"), cwd.join("modules")];

        // Add standard library path - check multiple locations
        // First check if we have a local stdlib directory
        let stdlib_path = cwd.join("stdlib");
        if stdlib_path.exists() {
            search_paths.push(stdlib_path);
        }

        // Also check parent directory (for when running from target/debug)
        let parent_stdlib = cwd.parent().and_then(|p| {
            let path = p.join("stdlib");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        });
        if let Some(path) = parent_stdlib {
            search_paths.push(path);
        }

        // Add standard library path if ZEN_HOME is set
        if let Ok(zen_home) = std::env::var("ZEN_HOME") {
            let zen_path = PathBuf::from(zen_home);
            search_paths.push(zen_path.join("stdlib"));
            search_paths.push(zen_path.join("std"));
            search_paths.push(zen_path.join("lib"));
        }

        // Also check relative to the executable (important for LSP which may run from different cwd)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check exe_dir/../../stdlib (for target/debug/zen -> zenlang/stdlib)
                if let Some(exe_parent) = exe_dir.parent() {
                    if let Some(project_root) = exe_parent.parent() {
                        let exe_stdlib = project_root.join("stdlib");
                        if exe_stdlib.exists() && !search_paths.contains(&exe_stdlib) {
                            search_paths.push(exe_stdlib);
                        }
                    }
                }
            }
        }

        ModuleSystem {
            modules: HashMap::new(),
            search_paths,
            cwd,
            insertion_counter: 0,
            loading_stack: Vec::new(),
            package_map: None,
        }
    }

    /// Set the package map (from build.zen discovery)
    pub fn set_package_map(&mut self, package_map: PackageMap) {
        self.package_map = Some(package_map);
    }

    /// Add a search path for modules
    #[allow(dead_code)] // Used in tests, public API for future use
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Resolve and load a module, using cached version if content unchanged
    pub fn load_module(&mut self, module_path: &str) -> Result<&Program, CompileError> {
        // Check for circular imports: if this module is already being loaded
        // up the call stack, we have a cycle
        if self.loading_stack.contains(&module_path.to_string()) {
            return Err(CompileError::CyclicDependency(
                format!("circular import detected: {}", module_path),
                None,
            ));
        }

        // Check PackageMap: resolve package-prefixed paths like "std.io"
        // This also handles paths like "http.server" from build.zen
        if let Some(ref package_map) = self.package_map.clone() {
            if let Some((_pkg_name, source, rest)) = package_map.resolve(module_path) {
                match source {
                    PackageSource::Stdlib => {
                        // Rewrite to @std form for existing resolution logic
                        let std_path = if rest.is_empty() {
                            "@std".to_string()
                        } else {
                            format!("@std.{}", rest)
                        };
                        // Avoid infinite recursion: only redirect if the path actually changed
                        if std_path != module_path {
                            return self.load_module(&std_path);
                        }
                    }
                    PackageSource::Local(base_path) => {
                        // Resolve local package: rest becomes a path relative to base_path
                        if !rest.is_empty() {
                            let relative = rest.replace('.', "/") + ".zen";
                            let file_path = base_path.join(&relative);
                            if file_path.exists() {
                                return self.load_file_module(module_path, &file_path);
                            }
                            // Try folder-name-as-index pattern
                            let parts: Vec<&str> = rest.split('.').collect();
                            if let Some(last) = parts.last() {
                                let dir_path: PathBuf =
                                    parts.iter().fold(base_path.clone(), |p, part| p.join(part));
                                let index_path = dir_path.join(format!("{}.zen", last));
                                if index_path.exists() {
                                    return self.load_file_module(module_path, &index_path);
                                }
                            }
                        }
                    }
                    PackageSource::Remote { .. } => {
                        // Future: resolve remote packages
                        // For now, return an error
                        return Err(CompileError::ImportError(
                            format!("Remote packages not yet supported: {}", module_path),
                            None,
                        ));
                    }
                }
            }
        }

        // Handle @std and std. modules - try to load actual stdlib files
        if module_path.starts_with("@std") || module_path.starts_with("std.") {
            let path_str = module_path
                .trim_start_matches("@std.")
                .trim_start_matches("@std")
                .trim_start_matches("std.");

            if !path_str.is_empty()
                && !path_str
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '_')
            {
                return Err(CompileError::FileNotFound(
                    format!("Invalid module path: {}", module_path),
                    Some("Module paths may only contain alphanumeric characters, dots, and underscores".to_string()),
                ));
            }

            if path_str.contains("..") {
                return Err(CompileError::FileNotFound(
                    format!("Invalid module path: {}", module_path),
                    Some("Module paths must not contain '..'".to_string()),
                ));
            }

            let path_parts: Vec<&str> = if path_str.is_empty() {
                vec![]
            } else {
                path_str.split('.').collect()
            };

            if path_parts.is_empty() {
                // @std itself - return empty program (cached if already present)
                if !self.modules.contains_key(module_path) {
                    self.insert_cached(
                        module_path.to_string(),
                        Program {
                            declarations: Vec::new(),
                            statements: Vec::new(),
                        },
                        0,
                    );
                }
                return Ok(&self.modules[module_path].program);
            }

            // @std.compiler is a built-in compiler module, not a file
            if path_parts.len() == 1 && path_parts[0] == "compiler" {
                if !self.modules.contains_key(module_path) {
                    self.insert_cached(
                        module_path.to_string(),
                        Program {
                            declarations: Vec::new(),
                            statements: Vec::new(),
                        },
                        0,
                    );
                }
                return Ok(&self.modules[module_path].program);
            }

            if let Some(file_to_load) = self.find_stdlib_file(&path_parts) {
                let source = std::fs::read_to_string(&file_to_load).map_err(|e| {
                    CompileError::FileNotFound(
                        file_to_load.display().to_string(),
                        Some(e.to_string()),
                    )
                })?;

                let new_hash = hash_content(&source);

                let cache_valid = self
                    .modules
                    .get(module_path)
                    .map(|c| c.content_hash == new_hash)
                    .unwrap_or(false);

                if !cache_valid {
                    let lexer = crate::lexer::Lexer::new(&source);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program().map_err(|e| {
                        CompileError::ParseError(
                            format!("Failed to parse stdlib module {}: {:?}", module_path, e),
                            None,
                        )
                    })?;

                    self.loading_stack.push(module_path.to_string());
                    let result: Result<(), CompileError> = (|| {
                        for decl in &program.declarations {
                            if let Declaration::ModuleImport {
                                alias: _,
                                module_path: import_path,
                                ..
                            } = decl
                            {
                                self.load_module(import_path)?;
                            }
                        }
                        Ok(())
                    })();
                    self.loading_stack.pop();
                    result?;

                    self.insert_cached(module_path.to_string(), program, new_hash);
                }
                return Ok(&self.modules[module_path].program);
            }

            // Module file not found on disk - cache empty program
            if !self.modules.contains_key(module_path) {
                self.insert_cached(
                    module_path.to_string(),
                    Program {
                        declarations: Vec::new(),
                        statements: Vec::new(),
                    },
                    0,
                );
            }
            return Ok(&self.modules[module_path].program);
        }

        // Try to find the module file
        let file_path = self.resolve_module_path(module_path)?;

        // Read and parse the module
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            CompileError::FileNotFound(file_path.display().to_string(), Some(e.to_string()))
        })?;

        let new_hash = hash_content(&source);

        let cache_valid = self
            .modules
            .get(module_path)
            .map(|c| c.content_hash == new_hash)
            .unwrap_or(false);

        if !cache_valid {
            let lexer = crate::lexer::Lexer::new(&source);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().map_err(|e| {
                CompileError::ParseError(
                    format!("Failed to parse module {}: {:?}", module_path, e),
                    None,
                )
            })?;

            let processed_program = program.clone();
            self.loading_stack.push(module_path.to_string());
            let result: Result<(), CompileError> = (|| {
                for decl in &program.declarations {
                    if let Declaration::ModuleImport {
                        alias: _,
                        module_path: import_path,
                        ..
                    } = decl
                    {
                        self.load_module(import_path)?;
                    }
                }
                Ok(())
            })();
            self.loading_stack.pop();
            result?;

            self.insert_cached(module_path.to_string(), processed_program, new_hash);
        }
        Ok(&self.modules[module_path].program)
    }

    /// Load a module from a specific file path (used by PackageMap resolution)
    fn load_file_module(
        &mut self,
        module_path: &str,
        file_path: &PathBuf,
    ) -> Result<&Program, CompileError> {
        let source = std::fs::read_to_string(file_path).map_err(|e| {
            CompileError::FileNotFound(file_path.display().to_string(), Some(e.to_string()))
        })?;

        let new_hash = hash_content(&source);
        let cache_valid = self
            .modules
            .get(module_path)
            .map(|c| c.content_hash == new_hash)
            .unwrap_or(false);

        if !cache_valid {
            let lexer = crate::lexer::Lexer::new(&source);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().map_err(|e| {
                CompileError::ParseError(
                    format!("Failed to parse module {}: {:?}", module_path, e),
                    None,
                )
            })?;

            self.loading_stack.push(module_path.to_string());
            let result: Result<(), CompileError> = (|| {
                for decl in &program.declarations {
                    if let Declaration::ModuleImport {
                        alias: _,
                        module_path: import_path,
                        ..
                    } = decl
                    {
                        self.load_module(import_path)?;
                    }
                }
                Ok(())
            })();
            self.loading_stack.pop();
            result?;

            self.insert_cached(module_path.to_string(), program, new_hash);
        }
        Ok(&self.modules[module_path].program)
    }

    /// Resolve a module path to a file path
    fn resolve_module_path(&self, module_path: &str) -> Result<PathBuf, CompileError> {
        // Convert module path (e.g., "std.io") to file path (e.g., "std/io.zen")
        let relative_path = module_path.replace('.', "/") + ".zen";

        // Try each search path
        for search_path in &self.search_paths {
            let full_path = search_path.join(&relative_path);
            if full_path.exists() {
                return Ok(full_path);
            }

            // Folder-name-as-index: foo.bar -> foo/bar/bar.zen
            let parts: Vec<&str> = module_path.split('.').collect();
            if let Some(last_part) = parts.last() {
                let dir_path = search_path.join(module_path.replace('.', "/"));
                let index_path = dir_path.join(format!("{}.zen", last_part));
                if index_path.exists() {
                    return Ok(index_path);
                }
            }

            // Also try as a directory with mod.zen (legacy fallback)
            let mod_path = search_path
                .join(module_path.replace('.', "/"))
                .join("mod.zen");
            if mod_path.exists() {
                return Ok(mod_path);
            }
        }

        Err(CompileError::FileNotFound(
            format!("Module '{}' not found in search paths", module_path),
            None,
        ))
    }

    pub fn get_modules(&self) -> HashMap<String, Program> {
        self.modules
            .iter()
            .map(|(k, v)| (k.clone(), v.program.clone()))
            .collect()
    }

    pub fn merge_programs(&self, main_program: Program) -> Program {
        let mut merged = main_program;

        for cached in self.modules.values() {
            for decl in &cached.program.declarations {
                if !matches!(decl, Declaration::ModuleImport { .. }) {
                    merged.declarations.push(decl.clone());
                }
            }
        }

        merged
    }

    fn insert_cached(&mut self, key: String, program: Program, content_hash: u64) {
        self.evict_if_needed();
        let order = self.insertion_counter;
        self.insertion_counter += 1;
        self.modules.insert(
            key,
            CachedModule {
                program,
                content_hash,
                insertion_order: order,
            },
        );
    }

    fn evict_if_needed(&mut self) {
        if self.modules.len() < MAX_CACHED_MODULES {
            return;
        }
        // Evict oldest entry by insertion_order
        if let Some(oldest_key) = self
            .modules
            .iter()
            .min_by_key(|(_, v)| v.insertion_order)
            .map(|(k, _)| k.clone())
        {
            self.modules.remove(&oldest_key);
        }
    }

    pub fn clear_cache(&mut self) {
        self.modules.clear();
    }

    fn find_stdlib_file(&self, path_parts: &[&str]) -> Option<PathBuf> {
        for search_path in &self.search_paths {
            let path_str = search_path.to_string_lossy();
            if !path_str.ends_with("stdlib") && !path_str.contains("stdlib") {
                continue;
            }

            let mut file_path = search_path.clone();
            for part in path_parts {
                file_path = file_path.join(part);
            }
            file_path.set_extension("zen");
            if file_path.exists() {
                return Some(file_path);
            }

            // Folder-name-as-index pattern: concurrency/async/async.zen
            // Works for any depth: @std.compiler -> compiler/compiler.zen
            //                      @std.concurrency.async -> concurrency/async/async.zen
            if let Some(last_part) = path_parts.last() {
                let mut dir_path = search_path.clone();
                for part in path_parts {
                    dir_path = dir_path.join(part);
                }
                let index_path = dir_path.join(format!("{}.zen", last_part));
                if index_path.exists() {
                    return Some(index_path);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::module_system::ModuleSystem;
    use std::path::PathBuf;

    #[test]
    fn test_module_system_creation() {
        let ms = ModuleSystem::new();
        assert!(ms.search_paths.len() >= 3);
        assert!(ms.modules.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut ms = ModuleSystem::new();
        let initial_len = ms.search_paths.len();
        ms.add_search_path(PathBuf::from("/custom/path"));
        assert_eq!(ms.search_paths.len(), initial_len + 1);
    }
}
