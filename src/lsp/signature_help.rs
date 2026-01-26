// Signature Help Module for Zen LSP
// Handles textDocument/signatureHelp requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, Mutex};

use super::document_store::DocumentStore;
use super::helpers::{
    char_pos_to_byte_pos, null_response, success_response, try_lock, try_parse_params,
};
use super::types::SymbolInfo;
use crate::ast::Declaration;
use crate::lsp::utils::format_type;

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_signature_help(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: SignatureHelpParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let empty_help = SignatureHelp {
        signatures: vec![],
        active_signature: None,
        active_parameter: None,
    };

    let store = match try_lock(store.as_ref(), &req) {
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

            // Try AST-based lookup first (highest fidelity)
            let mut signature_info = None;

            // Check document AST first for accurate parameter information
            if let Some(ast) = &doc.ast {
                if let Some(sig) = find_function_in_ast(ast, &function_name) {
                    signature_info = Some(sig);
                }
            }

            // Check stdlib symbols if not found in AST
            if signature_info.is_none() {
                // Try document symbols (may have detail string)
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

/// Find function in AST and create SignatureInformation directly from AST nodes
fn find_function_in_ast(ast: &[Declaration], function_name: &str) -> Option<SignatureInformation> {
    for decl in ast {
        match decl {
            Declaration::Function(func) if func.name == function_name => {
                // Build label from AST
                let args_str: String = func
                    .args
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let label = format!(
                    "{} = ({}) {}",
                    func.name,
                    args_str,
                    format_type(&func.return_type)
                );

                // Build parameters directly from AST
                let parameters: Vec<ParameterInformation> = func
                    .args
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
            Declaration::Struct(struct_def) => {
                // Check methods in struct
                for method in &struct_def.methods {
                    if method.name == function_name
                        || method.name == format!("{}.{}", struct_def.name, function_name)
                    {
                        let args_str: String = method
                            .args
                            .iter()
                            .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let label = format!(
                            "{}.{} = ({}) {}",
                            struct_def.name,
                            method.name,
                            args_str,
                            format_type(&method.return_type)
                        );

                        let parameters: Vec<ParameterInformation> = method
                            .args
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
            }
            _ => {}
        }
    }
    None
}

fn create_signature_info(symbol: &SymbolInfo) -> SignatureInformation {
    // Extract function signature from symbol detail
    let label = symbol
        .detail
        .clone()
        .unwrap_or_else(|| format!("{}(...)", symbol.name));

    // Parse parameters from the function signature
    let parameters = parse_function_parameters(&label);

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

fn parse_function_parameters(signature: &str) -> Vec<ParameterInformation> {
    // Parse signature like "function_name = (param1: Type1, param2: Type2) ReturnType"
    let mut parameters = Vec::new();

    // Find the parameter section between ( and )
    if let Some(start) = signature.find('(') {
        if let Some(end) = signature[start..].find(')') {
            let params_str = &signature[start + 1..start + end];

            // Split by commas (simple for now, could be enhanced for nested types)
            for param in params_str.split(',') {
                let param = param.trim();
                if !param.is_empty() {
                    parameters.push(ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple(param.to_string()),
                        documentation: None,
                    });
                }
            }
        }
    }

    parameters
}
