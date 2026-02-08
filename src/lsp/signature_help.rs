// Signature Help Module for Zen LSP
// Handles textDocument/signatureHelp requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, RwLock};

use super::document_store::DocumentStore;
use super::helpers::{
    char_pos_to_byte_pos, null_response, success_response, try_parse_params, try_read,
};
use super::types::SymbolInfo;
use crate::lsp::utils::format_type;
use crate::type_context::TypeContext;

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_signature_help(req: Request, store: &Arc<RwLock<DocumentStore>>) -> Response {
    let params: SignatureHelpParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let empty_help = SignatureHelp {
        signatures: vec![],
        active_signature: None,
        active_parameter: None,
    };

    let store = match try_read(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, empty_help),
    };

    let doc = match store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        Some(d) => d,
        None => return null_response(&req),
    };

    // Find function call at cursor position
    let position = params.text_document_position_params.position;
    let function_call = find_function_call_at_position(&doc.content, position);

    log::debug!(
        "[LSP] Signature help at {}:{} - function_call: {:?}",
        position.line,
        position.character,
        function_call
    );

    let signature_help = match function_call {
        Some((function_name, active_param)) => {
            log::debug!(
                "[LSP] Looking for function '{}' in {} doc symbols",
                function_name,
                doc.symbols.len()
            );

            let mut signature_info = None;

            // PRIORITY: Use TypeContext for authoritative parameter info
            if let Some(type_ctx) = &doc.type_context {
                signature_info = find_function_in_type_context(type_ctx, &function_name);
            }

            // Check document symbols
            if signature_info.is_none() {
                if let Some(symbol) = doc.symbols.get(&function_name) {
                    log::debug!("[LSP] Found '{}' in document symbols", function_name);
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            // Check stdlib symbols if not found
            if signature_info.is_none() {
                if let Some(symbol) = store.stdlib_symbols.get(&function_name) {
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            // Check workspace symbols if not found
            if signature_info.is_none() {
                if let Some(symbol) = store.workspace_symbols.get(&function_name) {
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            match signature_info {
                Some(sig_info) => SignatureHelp {
                    signatures: vec![sig_info],
                    active_signature: Some(0),
                    active_parameter: Some(active_param as u32),
                },
                None => SignatureHelp {
                    signatures: vec![],
                    active_signature: None,
                    active_parameter: None,
                },
            }
        }
        None => SignatureHelp {
            signatures: vec![],
            active_signature: None,
            active_parameter: None,
        },
    };

    Response {
        id: req.id,
        result: serde_json::to_value(signature_help).ok(),
        error: None,
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn find_function_call_at_position(content: &str, position: Position) -> Option<(String, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    // Build context string from current line and potential previous lines for multi-line calls
    let mut context = String::new();
    let mut line_offset = 0;

    // Look back up to 5 lines for multi-line function calls
    let start_line = position.line.saturating_sub(5);
    for i in start_line..=position.line {
        if i as usize >= lines.len() {
            break;
        }
        if i == position.line {
            line_offset = context.len();
        }
        context.push_str(lines[i as usize]);
        context.push(' ');
    }

    // Convert character position to byte position for the current line
    let current_line = lines.get(position.line as usize).unwrap_or(&"");
    let char_byte_offset = char_pos_to_byte_pos(current_line, position.character as usize);
    let cursor_pos = (line_offset + char_byte_offset).min(context.len());

    // Find the function call - look backwards from cursor for opening paren
    let mut paren_count = 0;
    let mut current_pos = cursor_pos;
    let context_bytes = context.as_bytes();

    // Move to the nearest opening paren (using bytes - parens are ASCII)
    while current_pos > 0 {
        let byte = context_bytes[current_pos - 1];
        if byte == b')' {
            paren_count += 1;
        } else if byte == b'(' {
            if paren_count == 0 {
                break;
            }
            paren_count -= 1;
        }
        current_pos -= 1;
    }

    if current_pos == 0 {
        return None; // No opening paren found
    }

    // Extract function name before the opening paren
    let before_paren = &context[..current_pos - 1];
    let function_name = before_paren
        .split(|c: char| {
            c.is_whitespace() || c == '=' || c == ',' || c == ';' || c == '{' || c == '('
        })
        .next_back()?
        .trim()
        .split('.')
        .next_back()?
        .to_string();

    // Count parameters by counting commas at paren_depth = 0
    let inside_parens = &context[current_pos..cursor_pos];
    let mut active_param = 0;
    let mut depth = 0;

    for ch in inside_parens.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => active_param += 1,
            _ => {}
        }
    }

    Some((function_name, active_param))
}

/// Find function in TypeContext and create SignatureInformation from typechecker data
fn find_function_in_type_context(
    type_ctx: &TypeContext,
    function_name: &str,
) -> Option<SignatureInformation> {
    // Check direct function match
    if let Some(func_type) = type_ctx.functions.get(function_name) {
        let params_str = func_type
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_str = format_type(&func_type.return_type);
        let label = format!("{} = ({}) {}", function_name, params_str, ret_str);

        let parameters: Vec<ParameterInformation> = func_type
            .params
            .iter()
            .map(|(name, ty)| ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(format!("{}: {}", name, format_type(ty))),
                documentation: None,
            })
            .collect();

        return Some(SignatureInformation {
            label,
            documentation: None,
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
            active_parameter: None,
        });
    }

    // Check methods — try "Type.method" patterns
    // The function_name from the call site might just be the method name,
    // so search method_params for any key ending with ".function_name"
    for (method_key, params) in &type_ctx.method_params {
        if method_key.ends_with(&format!(".{}", function_name)) {
            let ret_type = type_ctx.methods.get(method_key)?;
            let ret_str = format_type(ret_type);
            let params_str = params
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let label = format!("{} = ({}) {}", method_key, params_str, ret_str);

            let parameters: Vec<ParameterInformation> = params
                .iter()
                .map(|(name, ty)| ParameterInformation {
                    label: lsp_types::ParameterLabel::Simple(format!(
                        "{}: {}",
                        name,
                        format_type(ty)
                    )),
                    documentation: None,
                })
                .collect();

            return Some(SignatureInformation {
                label,
                documentation: None,
                parameters: if parameters.is_empty() {
                    None
                } else {
                    Some(parameters)
                },
                active_parameter: None,
            });
        }
    }

    None
}

/// Create SignatureInformation from a SymbolInfo, using structured params when available.
fn create_signature_info(symbol: &SymbolInfo) -> SignatureInformation {
    let label = symbol
        .detail
        .clone()
        .unwrap_or_else(|| format!("{}(...)", symbol.name));

    // Use structured params from SymbolInfo when available
    let parameters = if let Some(params) = &symbol.params {
        params
            .iter()
            .map(|(name, ty)| ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(format!("{}: {}", name, format_type(ty))),
                documentation: None,
            })
            .collect()
    } else {
        vec![] // No structured params available — don't guess from text
    };

    SignatureInformation {
        label,
        documentation: symbol
            .documentation
            .as_ref()
            .map(|doc| Documentation::String(doc.clone())),
        parameters: if parameters.is_empty() {
            None
        } else {
            Some(parameters)
        },
        active_parameter: None,
    }
}
