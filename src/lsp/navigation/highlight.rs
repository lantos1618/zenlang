// Document highlight handler

use super::utils::{find_all_symbol_occurrences, find_symbol_at_position};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{null_response, success_response, try_lock, try_parse_params};
use lsp_server::{Request, Response};
use lsp_types::*;

/// Find all occurrences of a symbol in a document for highlighting
fn find_symbol_occurrences(content: &str, symbol_name: &str) -> Vec<DocumentHighlight> {
    let occurrences = find_all_symbol_occurrences(content, symbol_name);
    occurrences
        .into_iter()
        .map(|(line_num, col, _)| DocumentHighlight {
            range: Range {
                start: Position {
                    line: line_num,
                    character: col as u32,
                },
                end: Position {
                    line: line_num,
                    character: (col + symbol_name.len()) as u32,
                },
            },
            kind: Some(DocumentHighlightKind::TEXT),
        })
        .collect()
}

/// Handle textDocument/documentHighlight requests
pub fn handle_document_highlight(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: DocumentHighlightParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    if let Some(doc) = store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        let position = params.text_document_position_params.position;
        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            let highlights = find_symbol_occurrences(&doc.content, &symbol_name);
            return success_response(&req, highlights);
        }
    }

    null_response(&req)
}
