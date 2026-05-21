use std::collections::HashMap;

use crate::ast::Program;
use crate::error::Span;
use crate::resolver::SymbolTable;

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
    pub(super) modules: HashMap<ModuleId, ResolvedModule>,
    pub(super) paths: HashMap<String, ModuleId>,
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
