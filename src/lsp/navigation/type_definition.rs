use super::utils::find_symbol_at_position;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{null_response, with_document, HasDocumentUri};
use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

pub fn handle_type_definition(
    req: Request,
    store: &std::sync::Arc<std::sync::RwLock<DocumentStore>>,
) -> Response {
    with_document::<GotoDefinitionParams, _>(&req, store, |doc, params, store_guard| {
        let position = params.text_document_position_params.position;

        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            if let Some(symbol_info) = doc.symbols.get(&symbol_name) {
                // Use SymbolInfo.type_info with AstType::base_name() instead of parsing detail strings
                let type_name = symbol_info
                    .type_info
                    .as_ref()
                    .and_then(|t| t.base_name().map(|s| s.to_string()));
                if let Some(type_name) = type_name {
                    if let Some(type_symbol) = store_guard.resolve_symbol(doc, &type_name) {
                        let uri = type_symbol
                            .definition_uri
                            .as_ref()
                            .unwrap_or(params.document_uri());

                        let location = Location {
                            uri: uri.clone(),
                            range: type_symbol.range,
                        };

                        return Response {
                            id: req.id.clone(),
                            result: Some(
                                serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                    .unwrap_or(Value::Null),
                            ),
                            error: None,
                        };
                    }
                }
            }

            if let Some(symbol_info) = store_guard.resolve_symbol(doc, &symbol_name) {
                let uri = symbol_info
                    .definition_uri
                    .as_ref()
                    .unwrap_or(params.document_uri());

                let location = Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                };

                return Response {
                    id: req.id.clone(),
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }
        }

        null_response(&req)
    })
}
