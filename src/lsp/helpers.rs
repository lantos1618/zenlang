// LSP Response Helpers
// Consolidates common response patterns to reduce duplication across LSP handlers

use lsp_server::{ErrorCode, Request, Response, ResponseError};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Creates a null response for the given request.
/// Use when a request cannot be fulfilled but isn't an error.
#[inline]
pub fn null_response(req: &Request) -> Response {
    Response {
        id: req.id.clone(),
        result: Some(Value::Null),
        error: None,
    }
}

/// Creates a success response with the given result.
#[inline]
pub fn success_response<T: serde::Serialize>(req: &Request, result: T) -> Response {
    Response {
        id: req.id.clone(),
        result: Some(serde_json::to_value(result).unwrap_or(Value::Null)),
        error: None,
    }
}

/// Creates an error response with the given error.
#[inline]
pub fn error_response(req: &Request, code: ErrorCode, message: impl Into<String>) -> Response {
    Response {
        id: req.id.clone(),
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
