// LSP Response Helpers
// Consolidates common response patterns to reduce duplication across LSP handlers

use lsp_server::{ErrorCode, Request, Response, ResponseError};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use super::document_store::DocumentStore;
use super::types::Document;

pub trait HasDocumentUri {
    fn document_uri(&self) -> &lsp_types::Url;
}

// text_document_position_params.text_document.uri
impl HasDocumentUri for lsp_types::HoverParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position_params.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::SignatureHelpParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position_params.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::GotoDefinitionParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position_params.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::DocumentHighlightParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position_params.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::CallHierarchyPrepareParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position_params.text_document.uri
    }
}

// text_document_position.text_document.uri
impl HasDocumentUri for lsp_types::ReferenceParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::RenameParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document_position.text_document.uri
    }
}

// text_document.uri
impl HasDocumentUri for lsp_types::InlayHintParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::CodeLensParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::DocumentFormattingParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::SemanticTokensParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::CodeActionParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::DocumentSymbolParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

impl HasDocumentUri for lsp_types::TextDocumentPositionParams {
    fn document_uri(&self) -> &lsp_types::Url {
        &self.text_document.uri
    }
}

pub fn with_document<P, F>(req: &Request, store: &Arc<Mutex<DocumentStore>>, f: F) -> Response
where
    P: DeserializeOwned + HasDocumentUri,
    F: FnOnce(&Document, &P, &std::sync::MutexGuard<'_, DocumentStore>) -> Response,
{
    let params: P = match try_parse_params(req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let store_guard = match try_lock(store.as_ref(), req) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    match store_guard.documents.get(params.document_uri()) {
        Some(doc) => f(doc, &params, &store_guard),
        None => null_response(req),
    }
}

#[inline]
pub fn null_response(req: &Request) -> Response {
    null_response_id(req.id.clone())
}

#[inline]
pub fn null_response_id(id: lsp_server::RequestId) -> Response {
    Response {
        id,
        result: Some(Value::Null),
        error: None,
    }
}

#[inline]
pub fn success_response<T: serde::Serialize>(req: &Request, result: T) -> Response {
    success_response_id(req.id.clone(), result)
}

#[inline]
pub fn success_response_id<T: serde::Serialize>(id: lsp_server::RequestId, result: T) -> Response {
    Response {
        id,
        result: Some(serde_json::to_value(result).unwrap_or(Value::Null)),
        error: None,
    }
}

#[inline]
pub fn error_response(req: &Request, code: ErrorCode, message: impl Into<String>) -> Response {
    error_response_id(req.id.clone(), code, message)
}

#[inline]
pub fn error_response_id(
    id: lsp_server::RequestId,
    code: ErrorCode,
    message: impl Into<String>,
) -> Response {
    Response {
        id,
        result: None,
        error: Some(ResponseError {
            code: code as i32,
            message: message.into(),
            data: None,
        }),
    }
}

/// Attempts to parse request parameters into the given type.
/// Returns Ok(params) on success, or a null Response on parse failure.
///
/// # Example
/// ```ignore
/// let params = try_parse_params::<HoverParams>(&req)?;
/// ```
pub fn try_parse_params<T: DeserializeOwned>(req: &Request) -> Result<T, Response> {
    serde_json::from_value(req.params.clone()).map_err(|_| null_response(req))
}

/// Attempts to parse request parameters, returning an error response with details on failure.
///
/// # Example
/// ```ignore
/// let params = try_parse_params_with_error::<HoverParams>(&req)?;
/// ```
pub fn try_parse_params_with_error<T: DeserializeOwned>(req: &Request) -> Result<T, Response> {
    serde_json::from_value(req.params.clone()).map_err(|e| Response {
        id: req.id.clone(),
        result: Some(Value::Null),
        error: Some(ResponseError {
            code: ErrorCode::InvalidParams as i32,
            message: format!("Invalid params: {}", e),
            data: None,
        }),
    })
}

/// Attempts to acquire a mutex lock, returning a null response on failure.
///
/// # Example
/// ```ignore
/// let store = try_lock_store(&store_arc, &req)?;
/// ```
pub fn try_lock<'a, T>(
    mutex: &'a std::sync::Mutex<T>,
    req: &Request,
) -> Result<std::sync::MutexGuard<'a, T>, Response> {
    mutex.lock().map_err(|_| null_response(req))
}

/// Macro to parse params and return early on failure.
/// Reduces boilerplate in LSP handlers.
///
/// # Example
/// ```ignore
/// parse_params!(req => HoverParams);
/// // Expands to:
/// // let params: HoverParams = try_parse_params(&req)?;
/// ```
#[macro_export]
macro_rules! parse_params {
    ($req:expr => $type:ty) => {
        match $crate::lsp::helpers::try_parse_params::<$type>($req) {
            Ok(p) => p,
            Err(resp) => return resp,
        }
    };
}

/// Macro to lock a mutex and return early on failure.
///
/// # Example
/// ```ignore
/// lock_store!(store_arc => store, req);
/// // Expands to:
/// // let store = try_lock(&store_arc, &req)?;
/// ```
#[macro_export]
macro_rules! lock_store {
    ($mutex:expr, $req:expr) => {
        match $crate::lsp::helpers::try_lock($mutex, $req) {
            Ok(guard) => guard,
            Err(resp) => return resp,
        }
    };
}

pub use lock_store;
pub use parse_params;

pub fn zen_code_block(code: &str) -> String {
    format!("```zen\n{}\n```", code)
}

/// Converts a UTF-16 character offset (LSP standard) to a byte offset in a string.
/// Returns the byte offset, clamped to valid string boundaries.
///
/// LSP uses UTF-16 code units for character positions, but Rust strings are UTF-8.
/// This function safely converts between the two.
#[inline]
pub fn char_pos_to_byte_pos(line: &str, char_pos: usize) -> usize {
    let mut byte_pos = 0;
    let mut utf16_pos = 0;

    for c in line.chars() {
        if utf16_pos >= char_pos {
            break;
        }
        byte_pos += c.len_utf8();
        utf16_pos += c.len_utf16();
    }

    byte_pos.min(line.len())
}
