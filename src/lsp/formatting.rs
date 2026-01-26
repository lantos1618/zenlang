use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::formatting::{
    fix_indentation, format_braces, format_enum_variants, normalize_variable_declarations,
    remove_trailing_whitespace,
};

use super::document_store::DocumentStore;
use super::helpers::{null_response, success_response, try_lock, try_parse_params_with_error};

pub fn handle_formatting(req: Request, store: Arc<Mutex<DocumentStore>>) -> Response {
    let params: DocumentFormattingParams = match try_parse_params_with_error(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, Vec::<TextEdit>::new()),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(doc) => doc,
        None => return null_response(&req),
    };

    // Format the document
    let formatted = format_document(&doc.content);

    // Create a single edit that replaces the entire document
    let line_count = doc.content.lines().count();
    let last_line_len = doc.content.lines().last().map(|l| l.len()).unwrap_or(0);

    let edit = TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end: Position::new(line_count as u32, last_line_len as u32),
        },
        new_text: formatted,
    };

    Response {
        id: req.id,
        result: Some(serde_json::to_value(vec![edit]).unwrap_or(Value::Null)),
        error: None,
    }
}

fn format_document(content: &str) -> String {
    // First, normalize variable declarations (fix syntax like `i: i32 ::= 0` → `i :: i32 = 0`)
    let normalized = normalize_variable_declarations(content);

    // Fix trailing whitespace
    let no_trailing = remove_trailing_whitespace(&normalized);

    // Fix indentation (tabs to spaces)
    let fixed_indent = fix_indentation(&no_trailing);

    // Then do brace-based indentation
    let braced = format_braces(&fixed_indent);

    // Finally format enum variants (must be last to preserve enum indentation)
    format_enum_variants(&braced)
}
