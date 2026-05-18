use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::Program;
use crate::error::{CompileError, FileTable, Span};
use crate::lexer;
use crate::parser;
use crate::resolver::SymbolTable;

mod graph_loading;
mod import_resolution;
mod root_prefix;

use import_resolution::find_stdlib_root;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportBinding {
    pub local_name: String,
    pub source_module: ModuleId,
    pub source_symbol: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub info: ModuleInfo,
    pub program: Program,
    pub imports: Vec<ImportBinding>,
    pub symbols: SymbolTable,
}

#[derive(Debug, Clone)]
pub struct ResolvedModuleGraph {
    pub entry: ModuleId,
    modules: HashMap<ModuleId, ResolvedModule>,
    paths: HashMap<String, ModuleId>,
}

impl ResolvedModuleGraph {
    pub fn module(&self, id: ModuleId) -> Option<&ResolvedModule> {
        self.modules.get(&id)
    }

    pub fn module_by_path(&self, canonical_path: &str) -> Option<&ResolvedModule> {
        let id = self.paths.get(canonical_path)?;
        self.module(*id)
    }

    pub fn modules(&self) -> &HashMap<ModuleId, ResolvedModule> {
        &self.modules
    }
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

    pub fn modules(&self) -> &HashMap<String, Program> {
        &self.modules
    }

    pub fn module_info(&self, canonical_path: &str) -> Option<&ModuleInfo> {
        self.module_infos.get(canonical_path)
    }

    pub fn module_infos(&self) -> &HashMap<String, ModuleInfo> {
        &self.module_infos
    }

    fn register_module_info(&mut self, key: &str, canonical: &Path) -> ModuleInfo {
        if let Some(info) = self.module_infos.get(key) {
            return info.clone();
        }

        let id = ModuleId(self.next_module_id);
        self.next_module_id += 1;
        let info = ModuleInfo {
            id,
            package_id: self.package_id_for(canonical),
            canonical_path: key.to_string(),
        };
        self.module_infos.insert(key.to_string(), info.clone());
        info
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

#[cfg(test)]
mod tests;
