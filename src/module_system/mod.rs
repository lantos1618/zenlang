use std::collections::HashMap;
use std::path::Path;

use crate::ast::Program;
use crate::error::{CompileError, FileTable, Span};
use crate::lexer;
use crate::parser;
use crate::resolver::SymbolTable;

mod graph_loading;
mod stdlib_paths;

pub use graph_loading::load_module_graph;
use stdlib_paths::find_stdlib_root;

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

    pub(crate) fn sorted_modules(&self) -> Vec<&ResolvedModule> {
        let mut modules = self.modules.values().collect::<Vec<_>>();
        modules.sort_by_key(|module| module.info.id.0);
        modules
    }
}

fn load_file(path: &Path, files: &mut FileTable) -> Result<Program, Vec<CompileError>> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        vec![CompileError::Internal(format!(
            "cannot read {}: {}",
            path.display(),
            e
        ))]
    })?;

    let file_id = files.add_file(path.display().to_string(), &source);
    let tokens = lexer::tokenize(&source, file_id).map_err(|e| vec![e])?;
    parser::parse(tokens, file_id)
}

fn package_id_for(canonical: &Path) -> PackageId {
    let is_stdlib = find_stdlib_root()
        .and_then(|root| root.canonicalize().ok())
        .is_some_and(|root| canonical.starts_with(root));

    if is_stdlib {
        PackageId(1)
    } else {
        PackageId(0)
    }
}
