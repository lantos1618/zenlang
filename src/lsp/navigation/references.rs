// References handler

use super::scope::determine_symbol_scope;
use super::utils::{
    find_function_range, find_symbol_at_position, is_in_string_or_comment, is_word_boundary_char,
};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::SymbolScope;
use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

/// Find all references to a symbol within a function
fn find_local_references(
    content: &str,
    symbol_name: &str,
    function_name: &str,
) -> Option<Vec<Range>> {
    let func_range = find_function_range(content, function_name)?;
    let mut references = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line_num in func_range.start.line..=func_range.end.line {
        if line_num as usize >= lines.len() {
            break;
        }

        let line = lines[line_num as usize];
        let mut start_col = 0;

        while let Some(col) = line[start_col..].find(symbol_name) {
            let actual_col = start_col + col;
            let before_ok =
                actual_col == 0 || is_word_boundary_char(line, actual_col.saturating_sub(1));
            let after_ok = actual_col + symbol_name.len() >= line.len()
                || is_word_boundary_char(line, actual_col + symbol_name.len());

            if before_ok && after_ok && !is_in_string_or_comment(line, actual_col) {
                references.push(Range {
                    start: Position {
                        line: line_num,
                        character: actual_col as u32,
                    },
                    end: Position {
                        line: line_num,
                        character: (actual_col + symbol_name.len()) as u32,
                    },
                });
            }
            start_col = actual_col + 1;
        }
    }
    Some(references)
}

/// Find all references to a symbol in a document
fn find_references_in_document(content: &str, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let mut start_col = 0;
        while let Some(col) = line[start_col..].find(symbol_name) {
            let actual_col = start_col + col;
            let before_ok =
                actual_col == 0 || is_word_boundary_char(line, actual_col.saturating_sub(1));
            let after_ok = actual_col + symbol_name.len() >= line.len()
                || is_word_boundary_char(line, actual_col + symbol_name.len());

            if before_ok && after_ok && !is_in_string_or_comment(line, actual_col) {
                references.push(Range {
                    start: Position {
                        line: line_num as u32,
                        character: actual_col as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: (actual_col + symbol_name.len()) as u32,
                    },
                });
            }
            start_col = actual_col + 1;
        }
    }
    references
}

/// Handle textDocument/references requests
pub fn handle_references(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: ReferenceParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => {
            return Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            };
        }
    };

    let store_lock = match store.lock() {
        Ok(s) => s,
        Err(_) => {
            return Response {
                id: req.id,
                result: Some(serde_json::to_value(Vec::<Location>::new()).unwrap_or(Value::Null)),
                error: None,
            };
        }
    };
    let mut locations = Vec::new();

    if let Some(doc) = store_lock
        .documents
        .get(&params.text_document_position.text_document.uri)
    {
        let position = params.text_document_position.position;

        // Find the symbol at the cursor position
        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            log::debug!("[LSP] Find references for symbol: '{}'", symbol_name);

            // Determine the scope of the symbol
            let symbol_scope = determine_symbol_scope(doc, &symbol_name, position);
            log::debug!("[LSP] Symbol scope: {:?}", symbol_scope);

            match symbol_scope {
                SymbolScope::Local { ref function_name } => {
                    // Local variable - only find references within the function
                    let current_uri = params.text_document_position.text_document.uri.clone();
                    if let Some(refs) =
                        find_local_references(&doc.content, &symbol_name, function_name)
                    {
                        for range in refs {
                            locations.push(Location {
                                uri: current_uri.clone(),
                                range,
                            });
                        }
                    }
                }
                SymbolScope::ModuleLevel | SymbolScope::Unknown => {
                    // Module-level symbol - search across documents (limited for performance)
                    for (uri, search_doc) in store_lock
                        .documents
                        .iter()
                        .take(crate::lsp::search_limits::REFERENCES_SEARCH)
                    {
                        let refs = find_references_in_document(&search_doc.content, &symbol_name);
                        for range in refs {
                            locations.push(Location {
                                uri: uri.clone(),
                                range,
                            });
                        }
                    }
                }
            }

            // Include the definition if requested
            if params.context.include_declaration {
                if let Some(symbol_info) = doc.symbols.get(&symbol_name) {
                    locations.push(Location {
                        uri: params.text_document_position.text_document.uri.clone(),
                        range: symbol_info.range,
                    });
                }
            }

            log::debug!("[LSP] Found {} reference(s)", locations.len());
        }
    }

    Response {
        id: req.id,
        result: Some(serde_json::to_value(locations).unwrap_or(Value::Null)),
        error: None,
    }
}
