// Type definition handler

use super::utils::find_symbol_at_position;
use crate::lsp::document_store::DocumentStore;
use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

/// Extract type name from a detail string like "name: Type" or "val: Result<f64, E>"
fn extract_type_name(detail: &str) -> Option<String> {
    if let Some(colon_pos) = detail.find(':') {
        let type_part = detail[colon_pos + 1..].trim();
        let type_name = if let Some(generic_start) = type_part.find('<') {
            &type_part[..generic_start]
        } else if let Some(space_pos) = type_part.find(' ') {
            &type_part[..space_pos]
        } else {
            type_part
        };
        return Some(type_name.trim().to_string());
    }
    None
}

/// Handle textDocument/typeDefinition requests
pub fn handle_type_definition(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: GotoDefinitionParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => {
            return Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            };
        }
    };

    let store = match store.lock() {
        Ok(s) => s,
        Err(_) => {
            return Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            };
        }
    };

    if let Some(doc) = store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        let position = params.text_document_position_params.position;

        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            // For type definition, we want to find the definition of the type, not the variable
            // First, check if it's a variable and get its type
            if let Some(symbol_info) = doc.symbols.get(&symbol_name) {
                // If it's a variable/parameter, extract the type name from its detail
                if let Some(detail) = &symbol_info.detail {
                    if let Some(type_name) = extract_type_name(detail) {
                        // Now find the definition of this type
                        if let Some(type_symbol) = store.resolve_symbol(doc, &type_name) {
                            let uri = type_symbol
                                .definition_uri
                                .as_ref()
                                .unwrap_or(&params.text_document_position_params.text_document.uri);

                            let location = Location {
                                uri: uri.clone(),
                                range: type_symbol.range,
                            };

                            return Response {
                                id: req.id,
                                result: Some(
                                    serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                        .unwrap_or(Value::Null),
                                ),
                                error: None,
                            };
                        }
                    }
                }
            }

            // If the symbol itself is a type, just use handle_definition logic
            if let Some(symbol_info) = store.resolve_symbol(doc, &symbol_name) {
                let uri = symbol_info
                    .definition_uri
                    .as_ref()
                    .unwrap_or(&params.text_document_position_params.text_document.uri);

                let location = Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                };

                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }
        }
    }

    Response {
        id: req.id,
        result: Some(Value::Null),
        error: None,
    }
}
