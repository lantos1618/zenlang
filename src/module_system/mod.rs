use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};
use crate::lexer;
use crate::parser;

/// Resolved modules by canonical path.
pub struct ModuleSystem {
    /// All loaded modules keyed by canonical path.
    modules: HashMap<String, Program>,
    /// Stdlib root directory.
    stdlib_root: Option<PathBuf>,
    /// Files currently being loaded (for circular import detection).
    loading: HashSet<PathBuf>,
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleSystem {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            stdlib_root: find_stdlib_root(),
            loading: HashSet::new(),
        }
    }

    /// Load and parse a file, returning its Program.
    pub fn load_file(
        &mut self,
        path: &Path,
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        let source = std::fs::read_to_string(path).map_err(|e| {
            vec![CompileError::Internal(format!(
                "cannot read {}: {}",
                path.display(),
                e
            ))]
        })?;

        let file_id = files.add_file(path.display().to_string(), source.clone());
        let tokens = lexer::tokenize(&source, file_id).map_err(|e| vec![e])?;
        let program = parser::parse(tokens, file_id)?;
        Ok(program)
    }

    /// Load a file and recursively resolve all its imports.
    ///
    /// This is the main entry point for multi-file compilation. It:
    /// 1. Loads and parses the entry file
    /// 2. Walks its Import declarations
    /// 3. Loads dependencies (relative file imports)
    /// 4. Merges imported declarations into the program
    /// 5. Detects circular imports
    pub fn load_with_imports(
        &mut self,
        path: &Path,
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        let canonical = path.canonicalize().map_err(|e| {
            vec![CompileError::Internal(format!(
                "cannot resolve path {}: {}",
                path.display(),
                e
            ))]
        })?;

        // Circular import detection
        if self.loading.contains(&canonical) {
            return Err(vec![CompileError::Resolution(
                format!("circular import detected: {}", canonical.display()),
                None,
            )]);
        }

        // If already loaded, return cached version
        let key = canonical.display().to_string();
        if let Some(prog) = self.modules.get(&key) {
            return Ok(prog.clone());
        }

        self.loading.insert(canonical.clone());

        let mut program = self.load_file(path, files)?;
        let base_dir = canonical
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        self.resolve_imports(&mut program, &base_dir, files)?;

        self.loading.remove(&canonical);
        self.modules.insert(key, program.clone());
        Ok(program)
    }

    /// Walk Import declarations in `program` and load their dependencies.
    ///
    /// Import routing:
    /// - `{ io } = std` or `{ io } = std.io` — stdlib, skip (handled by codegen)
    /// - `{ cast } = @builtin` — intrinsic, skip
    /// - `{ add } = math` — resolve `math.zen` relative to `base_dir`
    /// - `{ Foo } = utils.math` — resolve `utils/math.zen` relative to `base_dir`
    fn resolve_imports(
        &mut self,
        program: &mut Program,
        base_dir: &Path,
        files: &mut FileTable,
    ) -> Result<(), Vec<CompileError>> {
        // Collect imports first to avoid borrow conflict
        let imports: Vec<(Vec<String>, Vec<String>, Span)> = program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Import {
                    names,
                    module_path,
                    span,
                } => Some((names.clone(), module_path.clone(), *span)),
                _ => None,
            })
            .collect();

        let mut imported_decls: Vec<Declaration> = Vec::new();

        for (names, module_path, span) in imports {
            if module_path.is_empty() {
                return Err(vec![CompileError::Resolution(
                    "empty import path".into(),
                    Some(span),
                )]);
            }

            let first = &module_path[0];

            if first == "std" || first == "@std" {
                // Stdlib imports — handled by codegen/runtime, nothing to load
                continue;
            }

            if first == "@builtin" {
                // Builtin intrinsics — nothing to load
                continue;
            }

            // Local file import: resolve module_path segments as a file path
            // e.g. ["math"] → math.zen, ["utils", "math"] → utils/math.zen
            let rel_path: PathBuf = module_path.iter().collect();
            let mut file_path = base_dir.join(&rel_path);
            if file_path.extension().is_none() {
                file_path.set_extension("zen");
            }

            if !file_path.exists() {
                return Err(vec![CompileError::Resolution(
                    format!(
                        "cannot find imported module '{}' (looked for {})",
                        module_path.join("."),
                        file_path.display()
                    ),
                    Some(span),
                )]);
            }

            let dep_program = self.load_with_imports(&file_path, files)?;

            // Merge only the named imports
            let name_set: HashSet<&str> = names.iter().map(|n| n.as_str()).collect();
            for decl in &dep_program.declarations {
                if let Some(decl_name) = decl.name() {
                    if name_set.contains(decl_name) {
                        imported_decls.push(decl.clone());
                    }
                }
                // Also include methods on imported types
                if let Declaration::Method { type_name, .. } = decl {
                    if name_set.contains(type_name.as_str()) {
                        imported_decls.push(decl.clone());
                    }
                }
            }
        }

        // Append imported declarations to the program
        program.declarations.extend(imported_decls);
        Ok(())
    }

    /// Resolve an import module path to a filesystem path and load it.
    /// e.g. ["std"] or ["std", "io"] or ["@std", "io"]
    pub fn resolve_import(
        &mut self,
        module_path: &[String],
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        if module_path.is_empty() {
            return Err(vec![CompileError::Resolution(
                "empty module path".into(),
                None,
            )]);
        }

        let first = &module_path[0];

        // @std or std prefix → look in stdlib
        if first == "@std" || first == "std" {
            return self.resolve_stdlib_import(&module_path[1..], files);
        }

        // @builtin → return empty program (intrinsics)
        if first == "@builtin" {
            return Ok(Program {
                declarations: Vec::new(),
                file_id: 0,
            });
        }

        Err(vec![CompileError::Resolution(
            format!("unknown module: {}", module_path.join(".")),
            None,
        )])
    }

    fn resolve_stdlib_import(
        &mut self,
        sub_path: &[String],
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        let root = match &self.stdlib_root {
            Some(r) => r.clone(),
            None => {
                return Err(vec![CompileError::Resolution(
                    "stdlib not found".into(),
                    None,
                )]);
            }
        };

        if sub_path.is_empty() {
            // `{ io } = std` — load the stdlib prelude or nothing
            return Ok(Program {
                declarations: Vec::new(),
                file_id: 0,
            });
        }

        // Build path: stdlib/<sub>/<sub>.zen or stdlib/<sub>/mod.zen
        let mut dir = root.clone();
        for seg in sub_path {
            dir.push(seg);
        }

        // Try <dir>/<last>.zen
        let _last = sub_path.last().unwrap();
        let file_path = dir.with_extension("zen");
        if file_path.exists() {
            return self.load_file(&file_path, files);
        }

        // Try <dir>/mod.zen
        let mod_path = dir.join("mod.zen");
        if mod_path.exists() {
            return self.load_file(&mod_path, files);
        }

        // Try <dir>.zen (parent dir)
        let parent_file = dir.with_extension("zen");
        if parent_file.exists() {
            return self.load_file(&parent_file, files);
        }

        // Return empty for now — many stdlib modules are stubs
        Ok(Program {
            declarations: Vec::new(),
            file_id: 0,
        })
    }

    pub fn modules(&self) -> &HashMap<String, Program> {
        &self.modules
    }
}

/// Find the stdlib root by looking for a `stdlib/` directory.
fn find_stdlib_root() -> Option<PathBuf> {
    // Try relative to current exe
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Check alongside the binary
            let stdlib = dir.join("stdlib");
            if stdlib.is_dir() {
                return Some(stdlib);
            }
            // Check one level up (target/debug -> project root)
            if let Some(parent) = dir.parent() {
                if let Some(grandparent) = parent.parent() {
                    let stdlib = grandparent.join("stdlib");
                    if stdlib.is_dir() {
                        return Some(stdlib);
                    }
                }
            }
        }
    }

    // Try current working directory
    let cwd = std::env::current_dir().ok()?;
    let stdlib = cwd.join("stdlib");
    if stdlib.is_dir() {
        return Some(stdlib);
    }

    None
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn load_file_with_relative_import() {
        let tmp = setup_temp_dir();

        // Create math.zen with an add function
        let math_path = tmp.path().join("math.zen");
        fs::write(
            &math_path,
            "add = (a: i32, b: i32) i32 {\n    return a + b\n}\n",
        )
        .unwrap();

        // Create main.zen that imports from math (parser syntax: { add } = math)
        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ add } = math\n\nmain = () i32 {\n    return add(1, 2)\n}\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let program = ms.load_with_imports(&main_path, &mut files).unwrap();

        // Should have the import decl + main func + the merged add func
        let func_names: Vec<&str> = program
            .declarations
            .iter()
            .filter_map(|d| d.name())
            .collect();
        assert!(func_names.contains(&"main"), "should contain main");
        assert!(func_names.contains(&"add"), "should contain imported add");
    }

    #[test]
    fn circular_import_detected() {
        let tmp = setup_temp_dir();

        // a.zen imports from b
        let a_path = tmp.path().join("a.zen");
        fs::write(&a_path, "{ bar } = b\n\nfoo = () i32 { return 1 }\n").unwrap();

        // b.zen imports from a (circular!)
        let b_path = tmp.path().join("b.zen");
        fs::write(&b_path, "{ foo } = a\n\nbar = () i32 { return 2 }\n").unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let result = ms.load_with_imports(&a_path, &mut files);
        assert!(result.is_err(), "circular import should be an error");
        let errs = result.unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(
            msg.contains("circular import"),
            "error should mention circular import, got: {}",
            msg
        );
    }

    #[test]
    fn missing_import_file_error() {
        let tmp = setup_temp_dir();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ Foo } = nonexistent\n\nmain = () i32 { return 0 }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let result = ms.load_with_imports(&main_path, &mut files);
        assert!(result.is_err(), "missing import file should be an error");
        let errs = result.unwrap_err();
        let msg = format!("{}", errs[0]);
        assert!(
            msg.contains("cannot find imported module"),
            "error should mention missing file, got: {}",
            msg
        );
    }

    #[test]
    fn std_imports_are_skipped() {
        let tmp = setup_temp_dir();

        let main_path = tmp.path().join("main.zen");
        fs::write(&main_path, "{ io } = std\n\nmain = () i32 { return 0 }\n").unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        // std imports should not cause errors — they're handled by codegen
        let program = ms.load_with_imports(&main_path, &mut files).unwrap();
        assert!(program
            .declarations
            .iter()
            .any(|d| d.name() == Some("main")));
    }

    #[test]
    fn cached_module_not_reloaded() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(&math_path, "add = (a: i32, b: i32) i32 { return a + b }\n").unwrap();

        // Two files both import math
        let a_path = tmp.path().join("a.zen");
        fs::write(
            &a_path,
            "{ add } = math\n\nfoo = () i32 { return add(1, 2) }\n",
        )
        .unwrap();

        let b_path = tmp.path().join("b.zen");
        fs::write(
            &b_path,
            "{ add } = math\n\nbar = () i32 { return add(3, 4) }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        ms.load_with_imports(&a_path, &mut files).unwrap();
        ms.load_with_imports(&b_path, &mut files).unwrap();

        // math.zen canonical path should appear exactly once in modules cache
        let math_canonical = math_path.canonicalize().unwrap();
        let math_key = math_canonical.display().to_string();
        assert!(
            ms.modules().contains_key(&math_key),
            "math.zen should be cached by canonical path"
        );
        // File table should only have math.zen registered once (not re-read)
        // a.zen(id=0), math.zen(id=1), b.zen(id=2) — math not loaded twice
        assert_eq!(files.file_count(), 3, "should have 3 files: a, math, b");
    }

    #[test]
    fn transitive_imports() {
        let tmp = setup_temp_dir();

        // c.zen has a helper
        let c_path = tmp.path().join("c.zen");
        fs::write(&c_path, "helper = () i32 { return 42 }\n").unwrap();

        // b.zen imports from c
        let b_path = tmp.path().join("b.zen");
        fs::write(
            &b_path,
            "{ helper } = c\n\nwrapper = () i32 { return helper() }\n",
        )
        .unwrap();

        // a.zen imports from b
        let a_path = tmp.path().join("a.zen");
        fs::write(
            &a_path,
            "{ wrapper } = b\n\nmain = () i32 { return wrapper() }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let program = ms.load_with_imports(&a_path, &mut files).unwrap();
        let func_names: Vec<&str> = program
            .declarations
            .iter()
            .filter_map(|d| d.name())
            .collect();
        assert!(func_names.contains(&"main"));
        assert!(func_names.contains(&"wrapper"));
    }

    #[test]
    fn dotted_path_resolves_to_subdir() {
        let tmp = setup_temp_dir();

        // Create utils/math.zen
        let utils_dir = tmp.path().join("utils");
        fs::create_dir(&utils_dir).unwrap();
        let math_path = utils_dir.join("math.zen");
        fs::write(&math_path, "square = (x: i32) i32 { return x * x }\n").unwrap();

        // Create main.zen that imports from utils.math
        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ square } = utils.math\n\nmain = () i32 { return square(5) }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let program = ms.load_with_imports(&main_path, &mut files).unwrap();
        let func_names: Vec<&str> = program
            .declarations
            .iter()
            .filter_map(|d| d.name())
            .collect();
        assert!(func_names.contains(&"main"));
        assert!(func_names.contains(&"square"));
    }
}
