// Document highlight handler

use super::utils::{find_symbol_at_position, is_word_boundary_char};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{null_response, success_response, try_lock, try_parse_params};
use lsp_server::{Request, Response};
use lsp_types::*;

/// Find all occurrences of a symbol in a document for highlighting
fn find_symbol_occurrences(content: &str, symbol_name: &str) -> Vec<DocumentHighlight> {
    let mut highlights = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut start_pos = 0;
        while let Some(pos) = line[start_pos..].find(symbol_name) {
            let actual_pos = start_pos + pos;
            let is_word_start =
                actual_pos == 0 || is_word_boundary_char(line, actual_pos.saturating_sub(1));
            let end_pos = actual_pos + symbol_name.len();
            let is_word_end = end_pos >= line.len() || is_word_boundary_char(line, end_pos);

            if is_word_start && is_word_end {
                highlights.push(DocumentHighlight {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_pos as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end_pos as u32,
                        },
                    },
                    kind: Some(DocumentHighlightKind::TEXT),
                });
            }
            start_pos = actual_pos + 1;
        }
    }
    highlights
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
