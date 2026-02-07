// Symbols Module for Zen LSP
// Handles document symbols and workspace symbol search

use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

use super::document_store::DocumentStore;

// ============================================================================
// PUBLIC HANDLER FUNCTIONS
// ============================================================================

/// Handle textDocument/documentSymbol requests
pub fn handle_document_symbols(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    log::debug!("[LSP SYMBOLS] Starting document symbols request");
    let params: DocumentSymbolParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(e) => {
            log::debug!("[LSP SYMBOLS] Failed to parse params: {:?}", e);
            return Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            };
        }
    };

    log::debug!("[LSP SYMBOLS] Waiting for store lock...");
    let lock_start = std::time::Instant::now();
    let store = match store.lock() {
        Ok(s) => {
            log::debug!("[LSP SYMBOLS] Got store lock in {:?}", lock_start.elapsed());
            s
        }
        Err(e) => {
            log::debug!("[LSP SYMBOLS] Failed to get store lock: {:?}", e);
            return Response {
                id: req.id,
                result: Some(
                    serde_json::to_value(Vec::<SymbolInformation>::new())
                        .unwrap_or(serde_json::Value::Null),
                ),
                error: None,
            };
        }
    };
    log::debug!(
        "[LSP SYMBOLS] Looking up document: {}",
        params.text_document.uri
    );
    if let Some(doc) = store.documents.get(&params.text_document.uri) {
        log::debug!(
            "[LSP SYMBOLS] Found document with {} symbols",
            doc.symbols.len()
        );
        let symbols: Vec<DocumentSymbol> = doc
            .symbols
            .values()
            .map(|sym| DocumentSymbol {
                name: sym.name.clone(),
                detail: sym.detail.clone(),
                kind: sym.kind,
                tags: None,
                #[allow(deprecated)]
                deprecated: None,
                range: sym.range,
                selection_range: sym.range,
                children: None,
            })
            .collect();

        log::debug!("[LSP SYMBOLS] Returning {} symbols", symbols.len());
        return crate::lsp::helpers::success_response_id(req.id, symbols);
    }

    log::debug!("[LSP SYMBOLS] Document not found in store");
    crate::lsp::helpers::null_response_id(req.id)
}

/// Handle workspace/symbol requests
pub fn handle_workspace_symbol(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: WorkspaceSymbolParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => {
            return crate::lsp::helpers::error_response_id(
                req.id,
                lsp_server::ErrorCode::InvalidParams,
                "Invalid parameters",
            )
        }
    };

    let store = match store.lock() {
        Ok(s) => s,
        Err(_) => {
            return crate::lsp::helpers::success_response_id(req.id, Vec::<WorkspaceSymbol>::new());
        }
    };
    // Optimized: lowercase query once instead of for every symbol
    let query_lower = params.query.to_lowercase();
    let mut symbols = Vec::with_capacity(100); // Pre-allocate for common case

    // Search in all open documents
    for (uri, doc) in &store.documents {
        for (name, symbol_info) in &doc.symbols {
            // Optimized: lowercase name once per symbol instead of in contains check
            if name.to_lowercase().contains(&query_lower) {
                symbols.push(SymbolInformation {
                    name: symbol_info.name.clone(),
                    kind: symbol_info.kind,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: symbol_info.range,
                    },
                    container_name: None,
                });
            }
        }
    }

    // Search in stdlib symbols
    for (name, symbol_info) in &store.stdlib_symbols {
        if name.to_lowercase().contains(&query_lower) {
            if let Some(def_uri) = &symbol_info.definition_uri {
                symbols.push(SymbolInformation {
                    name: symbol_info.name.clone(),
                    kind: symbol_info.kind,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    location: Location {
                        uri: def_uri.clone(),
                        range: symbol_info.range,
                    },
                    container_name: Some("stdlib".to_string()),
                });
            }
        }
    }

    // Search in workspace symbols (indexed from all files)
    for (name, symbol_info) in &store.workspace_symbols {
        if name.to_lowercase().contains(&query_lower) {
            if let Some(def_uri) = &symbol_info.definition_uri {
                symbols.push(SymbolInformation {
                    name: symbol_info.name.clone(),
                    kind: symbol_info.kind,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    location: Location {
                        uri: def_uri.clone(),
                        range: symbol_info.range,
                    },
                    container_name: Some("workspace".to_string()),
                });
            }
        }
    }

    // Limit results to avoid overwhelming the client
    symbols.truncate(100);

    crate::lsp::helpers::success_response_id(req.id, symbols)
}
