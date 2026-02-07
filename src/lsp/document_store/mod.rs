// Main DocumentStore struct and public API
mod builtin_registration;
mod document_lifecycle;
mod parsing;
mod reference_tracking;
mod symbol_extraction;
mod symbol_search;
mod utilities;
mod variable_extraction;

use super::indexing::{find_stdlib_path, index_stdlib_directory, index_workspace_files_recursive};
use super::stdlib_resolver::StdlibResolver;
use super::types::{AnalysisJob, Document, SymbolInfo};
use lsp_types::*;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub struct DocumentStore {
    pub documents: HashMap<Url, Document>,
    pub stdlib_symbols: HashMap<String, SymbolInfo>,
    pub workspace_symbols: HashMap<String, SymbolInfo>, // Indexed workspace symbols
    pub workspace_root: Option<Url>,
    pub analysis_sender: Option<Sender<AnalysisJob>>,
    pub stdlib_resolver: StdlibResolver,
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentStore {
    pub fn new() -> Self {
        let workspace_root_path = None::<&std::path::Path>;
        let stdlib_resolver = StdlibResolver::new(workspace_root_path);

        let mut store = Self {
            documents: HashMap::new(),
            stdlib_symbols: HashMap::new(),
            workspace_symbols: HashMap::new(),
            workspace_root: None,
            analysis_sender: None,
            stdlib_resolver,
        };

        store.register_builtin_types();
        store
    }

    /// Resolve a symbol by name: document → stdlib → workspace.
    /// Use for type/definition lookups where stdlib is authoritative.
    pub fn resolve_symbol<'a>(&'a self, doc: &'a Document, name: &str) -> Option<&'a SymbolInfo> {
        doc.symbols
            .get(name)
            .or_else(|| self.stdlib_symbols.get(name))
            .or_else(|| self.workspace_symbols.get(name))
    }

    /// Resolve a symbol by name: local → workspace → stdlib.
    /// Use for hover/inference where local definitions shadow stdlib.
    pub fn resolve_symbol_local_first<'a>(
        &'a self,
        local_symbols: &'a HashMap<String, SymbolInfo>,
        name: &str,
    ) -> Option<&'a SymbolInfo> {
        local_symbols
            .get(name)
            .or_else(|| self.workspace_symbols.get(name))
            .or_else(|| self.stdlib_symbols.get(name))
    }

    pub fn index_stdlib_deferred(&mut self) {
        self.index_stdlib();
    }

    fn index_stdlib(&mut self) {
        if let Some(stdlib_path) = find_stdlib_path() {
            index_stdlib_directory(&stdlib_path, &mut self.stdlib_symbols);
            log::debug!(
                "[LSP] Indexed {} stdlib symbols from: {}",
                self.stdlib_symbols.len(),
                stdlib_path.display()
            );
        }
    }

    pub fn set_analysis_sender(&mut self, sender: Sender<AnalysisJob>) {
        self.analysis_sender = Some(sender);
    }

    pub fn set_workspace_root(&mut self, root_uri: Url) {
        self.workspace_root = Some(root_uri.clone());

        // Update stdlib resolver with workspace root
        if let Ok(workspace_path) = root_uri.to_file_path() {
            self.stdlib_resolver = StdlibResolver::new(Some(&workspace_path));
        }

        // Note: Workspace indexing is now done asynchronously after initialization
        // to avoid blocking the main thread and holding locks for extended periods
    }

    pub fn index_workspace(&mut self, root_uri: &Url) {
        if let Ok(root_path) = root_uri.to_file_path() {
            log::debug!("[LSP] Indexing workspace: {}", root_path.display());
            let start = Instant::now();

            let count = self.index_workspace_directory(&root_path);

            let duration = start.elapsed();
            log::debug!(
                "[LSP] Indexed {} symbols from workspace in {:?}",
                count,
                duration
            );
        }
    }

    // Static method for background workspace indexing (doesn't hold locks)
    pub fn index_workspace_files(root_path: &std::path::Path) -> HashMap<String, SymbolInfo> {
        let mut workspace_symbols = HashMap::new();
        index_workspace_files_recursive(root_path, &mut workspace_symbols);
        workspace_symbols
    }

    fn index_workspace_directory(&mut self, path: &std::path::Path) -> usize {
        use std::fs;

        let mut symbol_count = 0;

        // Skip common directories we don't want to index
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if dir_name == "target"
                || dir_name == "node_modules"
                || dir_name == ".git"
                || dir_name == "tests"
                || dir_name.starts_with('.')
            {
                return 0;
            }
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                if entry_path.is_file() && entry_path.extension().is_some_and(|e| e == "zen") {
                    // Skip test files
                    if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        if crate::name_utils::is_test_file(file_name) {
                            continue;
                        }
                    }

                    if let Ok(content) = fs::read_to_string(&entry_path) {
                        let symbols = self.extract_symbols(&content);

                        // Convert path to URI for workspace symbols
                        if let Ok(uri) = Url::from_file_path(&entry_path) {
                            for (name, mut symbol) in symbols {
                                symbol.definition_uri = Some(uri.clone());
                                // Only add if not already in stdlib (stdlib takes priority)
                                if !self.stdlib_symbols.contains_key(&name) {
                                    self.workspace_symbols.insert(name, symbol);
                                    symbol_count += 1;
                                }
                            }
                        }
                    }
                } else if entry_path.is_dir() {
                    // Recursively index subdirectories
                    symbol_count += self.index_workspace_directory(&entry_path);
                }
            }
        }

        symbol_count
    }
}
