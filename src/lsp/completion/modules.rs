//! Module path completions
//!
//! Handles completions for module paths like @std.io, @std.collections

use lsp_types::*;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::stdlib_resolver::StdlibResolver;

/// Get module path completions (e.g., @std. -> io, types, collections)
pub fn get_module_path_completions(base: &str, store: &DocumentStore) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    if base == "@std" {
        let available_modules = store.stdlib_resolver.list_modules();

        for module_name in available_modules {
            completions.push(CompletionItem {
                label: format!("{}.{}", base, module_name),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(format!("Import from {} module", module_name)),
                documentation: Some(Documentation::String(format!(
                    "Standard library module: {}",
                    module_name
                ))),
                ..Default::default()
            });
        }
    } else if base.starts_with("@std.") {
        let submodule = base.strip_prefix("@std.").unwrap_or("");

        let submodule_dir = store
            .stdlib_resolver
            .stdlib_root
            .join(submodule.replace('.', "/"));

        if submodule_dir.exists() && submodule_dir.is_dir() {
            let mut submodules = Vec::new();
            StdlibResolver::scan_directory(&submodule_dir, &mut submodules, submodule);

            for submod in submodules {
                completions.push(CompletionItem {
                    label: format!("{}.{}", base, submod),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(format!("Nested module: {}", submod)),
                    documentation: None,
                    ..Default::default()
                });
            }
        }
    }

    completions
}
