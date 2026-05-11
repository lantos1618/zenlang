use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};
use crate::lexer;
use crate::parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub id: ModuleId,
    pub package_id: PackageId,
    pub canonical_path: String,
}

/// Resolved modules by canonical path.
pub struct ModuleSystem {
    /// All loaded modules keyed by canonical path.
    modules: HashMap<String, Program>,
    /// Module graph records keyed by canonical path.
    module_infos: HashMap<String, ModuleInfo>,
    /// Next module ID to assign.
    next_module_id: u32,
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
            module_infos: HashMap::new(),
            next_module_id: 0,
            stdlib_root: find_stdlib_root(),
            loading: HashSet::new(),
        }
    }

    pub fn with_stdlib_root(stdlib_root: PathBuf) -> Self {
        Self {
            modules: HashMap::new(),
            module_infos: HashMap::new(),
            next_module_id: 0,
            stdlib_root: Some(stdlib_root),
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
        self.register_module_info(&key, &canonical);
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
                if module_path.len() == 1 {
                    // Root prelude imports are still handled by codegen/runtime.
                    continue;
                }

                let Some(file_path) = self.resolve_stdlib_file_path(&module_path[1..])? else {
                    return Err(vec![CompileError::Resolution(
                        format!("cannot find stdlib module '{}'", module_path.join(".")),
                        Some(span),
                    )]);
                };

                let dep_program = self.load_with_imports(&file_path, files)?;
                self.collect_imported_declarations(
                    &dep_program,
                    &names,
                    &module_path.join("."),
                    span,
                    &mut imported_decls,
                )?;
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

            let mut requested_names = HashSet::new();
            for name in &names {
                if !requested_names.insert(name.as_str()) {
                    return Err(vec![CompileError::Resolution(
                        format!(
                            "duplicate import '{}' from module '{}'",
                            name,
                            module_path.join(".")
                        ),
                        Some(span),
                    )]);
                }
            }

            let dep_program = self.load_with_imports(&file_path, files)?;
            self.collect_imported_declarations(
                &dep_program,
                &names,
                &module_path.join("."),
                span,
                &mut imported_decls,
            )?;
        }

        // Append imported declarations to the program
        program.declarations.extend(imported_decls);
        Ok(())
    }

    fn collect_imported_declarations(
        &self,
        dep_program: &Program,
        names: &[String],
        module_name: &str,
        import_span: Span,
        imported_decls: &mut Vec<Declaration>,
    ) -> Result<(), Vec<CompileError>> {
        for name in names {
            let mut found_private = false;
            let mut found_public = false;

            for decl in &dep_program.declarations {
                if decl.name() == Some(name.as_str()) {
                    if decl.is_public() {
                        found_public = true;
                        imported_decls.push(decl.clone());
                    } else {
                        found_private = true;
                    }
                }

                // Importing a public type also brings its public methods into scope.
                if let Declaration::Method {
                    type_name, public, ..
                } = decl
                {
                    if type_name == name && *public {
                        imported_decls.push(decl.clone());
                    }
                }
            }

            if !found_public {
                if found_private {
                    return Err(vec![CompileError::Resolution(
                        format!(
                            "symbol '{}' in module '{}' is not exported",
                            name, module_name
                        ),
                        Some(import_span),
                    )]);
                }
                return Err(vec![CompileError::Resolution(
                    format!("module '{}' does not export '{}'", module_name, name),
                    Some(import_span),
                )]);
            }
        }

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
        if sub_path.is_empty() {
            // `{ io } = std` — load the stdlib prelude or nothing
            return Ok(Program {
                declarations: Vec::new(),
                file_id: 0,
            });
        }

        if let Some(file_path) = self.resolve_stdlib_file_path(sub_path)? {
            return self.load_file(&file_path, files);
        }

        // Return empty for now — many stdlib modules are stubs
        Ok(Program {
            declarations: Vec::new(),
            file_id: 0,
        })
    }

    fn resolve_stdlib_file_path(
        &self,
        sub_path: &[String],
    ) -> Result<Option<PathBuf>, Vec<CompileError>> {
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
            return Ok(None);
        }

        let mut dir = root;
        for seg in sub_path {
            dir.push(seg);
        }

        let file_path = dir.with_extension("zen");
        if file_path.exists() {
            return Ok(Some(file_path));
        }

        let mod_path = dir.join("mod.zen");
        if mod_path.exists() {
            return Ok(Some(mod_path));
        }

        let parent_file = dir.with_extension("zen");
        if parent_file.exists() {
            return Ok(Some(parent_file));
        }

        Ok(None)
    }

    pub fn modules(&self) -> &HashMap<String, Program> {
        &self.modules
    }

    pub fn module_info(&self, canonical_path: &str) -> Option<&ModuleInfo> {
        self.module_infos.get(canonical_path)
    }

    pub fn module_infos(&self) -> &HashMap<String, ModuleInfo> {
        &self.module_infos
    }

    fn register_module_info(&mut self, key: &str, canonical: &Path) {
        if self.module_infos.contains_key(key) {
            return;
        }

        let id = ModuleId(self.next_module_id);
        self.next_module_id += 1;
        self.module_infos.insert(
            key.to_string(),
            ModuleInfo {
                id,
                package_id: self.package_id_for(canonical),
                canonical_path: key.to_string(),
            },
        );
    }

    fn package_id_for(&self, canonical: &Path) -> PackageId {
        let is_stdlib = self
            .stdlib_root
            .as_ref()
            .and_then(|root| root.canonicalize().ok())
            .is_some_and(|root| canonical.starts_with(root));

        if is_stdlib {
            PackageId(1)
        } else {
            PackageId(0)
        }
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
            "pub add = (a: i32, b: i32) i32 {\n    return a + b\n}\n",
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
    fn stdlib_submodule_import_loads_through_module_system() {
        let tmp = setup_temp_dir();
        let stdlib = tmp.path().join("stdlib");
        fs::create_dir(&stdlib).unwrap();
        fs::write(
            &stdlib.join("math.zen"),
            "pub answer = () i32 { return 42 }\n",
        )
        .unwrap();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ answer } = std.math\n\nmain = () i32 { return answer() }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::with_stdlib_root(stdlib.clone());

        let program = ms.load_with_imports(&main_path, &mut files).unwrap();
        let func_names: Vec<&str> = program
            .declarations
            .iter()
            .filter_map(|d| d.name())
            .collect();

        assert!(func_names.contains(&"main"));
        assert!(func_names.contains(&"answer"));
        assert_eq!(
            files.file_count(),
            2,
            "main and stdlib module should both be loaded"
        );

        let std_key = stdlib
            .join("math.zen")
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        let std_info = ms.module_info(&std_key).expect("stdlib module info");
        assert_eq!(std_info.package_id.0, 1, "stdlib modules use package 1");
    }

    #[test]
    fn cached_module_not_reloaded() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .unwrap();

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
    fn loaded_modules_have_stable_ids_and_package_ids() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .unwrap();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ add } = math\n\nmain = () i32 { return add(1, 2) }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();
        ms.load_with_imports(&main_path, &mut files).unwrap();

        let main_key = main_path.canonicalize().unwrap().display().to_string();
        let math_key = math_path.canonicalize().unwrap().display().to_string();
        let main_info = ms.module_info(&main_key).expect("main module info");
        let math_info = ms.module_info(&math_key).expect("math module info");

        assert_ne!(main_info.id, math_info.id);
        assert_eq!(main_info.package_id, math_info.package_id);
        assert_eq!(main_info.package_id.0, 0, "local modules use package 0");
        assert_eq!(main_info.canonical_path, main_key);
    }

    #[test]
    fn transitive_imports() {
        let tmp = setup_temp_dir();

        // c.zen has a helper
        let c_path = tmp.path().join("c.zen");
        fs::write(&c_path, "pub helper = () i32 { return 42 }\n").unwrap();

        // b.zen imports from c
        let b_path = tmp.path().join("b.zen");
        fs::write(
            &b_path,
            "{ helper } = c\n\npub wrapper = () i32 { return helper() }\n",
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
        fs::write(&math_path, "pub square = (x: i32) i32 { return x * x }\n").unwrap();

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

    #[test]
    fn private_import_is_rejected() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(&math_path, "add = (a: i32, b: i32) i32 { return a + b }\n").unwrap();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ add } = math\n\nmain = () i32 { return add(1, 2) }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let result = ms.load_with_imports(&main_path, &mut files);
        assert!(result.is_err(), "private import should be rejected");
        let msg = format!("{}", result.unwrap_err()[0]);
        assert!(
            msg.contains("not exported"),
            "error should mention export visibility, got: {msg}"
        );
    }

    #[test]
    fn missing_imported_symbol_is_rejected() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .unwrap();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ subtract } = math\n\nmain = () i32 { return 0 }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let result = ms.load_with_imports(&main_path, &mut files);
        assert!(
            result.is_err(),
            "missing imported symbol should be rejected"
        );
        let msg = format!("{}", result.unwrap_err()[0]);
        assert!(
            msg.contains("does not export"),
            "error should mention missing export, got: {msg}"
        );
    }

    #[test]
    fn duplicate_imported_symbol_is_rejected() {
        let tmp = setup_temp_dir();

        let math_path = tmp.path().join("math.zen");
        fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .unwrap();

        let main_path = tmp.path().join("main.zen");
        fs::write(
            &main_path,
            "{ add, add } = math\n\nmain = () i32 { return 0 }\n",
        )
        .unwrap();

        let mut files = FileTable::new();
        let mut ms = ModuleSystem::new();

        let result = ms.load_with_imports(&main_path, &mut files);
        assert!(result.is_err(), "duplicate import should be rejected");
        let msg = format!("{}", result.unwrap_err()[0]);
        assert!(
            msg.contains("duplicate import"),
            "error should mention duplicate import, got: {msg}"
        );
    }
}
