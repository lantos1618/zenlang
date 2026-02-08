use super::utils::{find_all_symbol_occurrences, find_symbol_at_position};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{null_response, success_response, with_document};
use lsp_server::{Request, Response};
use lsp_types::*;

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

pub fn handle_document_highlight(
    req: Request,
    store: &std::sync::Arc<std::sync::RwLock<DocumentStore>>,
) -> Response {
    with_document::<DocumentHighlightParams, _>(&req, store, |doc, params, _store| {
        let position = params.text_document_position_params.position;
        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            let highlights = find_symbol_occurrences(&doc.content, &symbol_name);
            return success_response(&req, highlights);
        }
        null_response(&req)
    })
}
