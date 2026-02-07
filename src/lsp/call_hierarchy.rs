// Call Hierarchy Module for Zen LSP
// Handles call hierarchy requests

use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

use super::document_store::DocumentStore;
use super::navigation::find_symbol_at_position;

// ============================================================================
// PUBLIC HANDLER FUNCTIONS
// ============================================================================

/// Handle textDocument/prepareCallHierarchy requests
pub fn handle_prepare_call_hierarchy(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: CallHierarchyPrepareParams = match serde_json::from_value(req.params) {
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
                result: Some(
                    serde_json::to_value(Vec::<CallHierarchyItem>::new())
                        .unwrap_or(serde_json::Value::Null),
                ),
                error: None,
            };
        }
    };
    if let Some(doc) = store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        let position = params.text_document_position_params.position;

        // Find the symbol at the cursor position
        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            // Check if it's a function
            if let Some(symbol_info) = doc.symbols.get(&symbol_name) {
                if symbol_info.kind == SymbolKind::FUNCTION
                    || symbol_info.kind == SymbolKind::METHOD
                {
                    let call_hierarchy_item = CallHierarchyItem {
                        name: symbol_info.name.clone(),
                        kind: symbol_info.kind,
                        tags: None,
                        detail: symbol_info.detail.clone(),
                        uri: params
                            .text_document_position_params
                            .text_document
                            .uri
                            .clone(),
                        range: symbol_info.range,
                        selection_range: symbol_info.selection_range,
                        data: None,
                    };

                    return Response {
                        id: req.id,
                        result: Some(
                            serde_json::to_value(vec![call_hierarchy_item]).unwrap_or(Value::Null),
                        ),
                        error: None,
                    };
                }
            }

            // Check stdlib symbols
            if let Some(symbol_info) = store.stdlib_symbols.get(&symbol_name) {
                if symbol_info.kind == SymbolKind::FUNCTION
                    || symbol_info.kind == SymbolKind::METHOD
                {
                    if let Some(uri) = &symbol_info.definition_uri {
                        let call_hierarchy_item = CallHierarchyItem {
                            name: symbol_info.name.clone(),
                            kind: symbol_info.kind,
                            tags: None,
                            detail: symbol_info.detail.clone(),
                            uri: uri.clone(),
                            range: symbol_info.range,
                            selection_range: symbol_info.selection_range,
                            data: None,
                        };

                        return Response {
                            id: req.id,
                            result: Some(
                                serde_json::to_value(vec![call_hierarchy_item])
                                    .unwrap_or(Value::Null),
                            ),
                            error: None,
                        };
                    }
                }
            }
        }
    }

    Response {
        id: req.id,
        result: Some(Value::Null),
        error: None,
    }
}

/// Handle callHierarchy/incomingCalls requests
pub fn handle_incoming_calls(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: CallHierarchyIncomingCallsParams = match serde_json::from_value(req.params) {
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
                result: Some(
                    serde_json::to_value(Vec::<CallHierarchyIncomingCall>::new())
                        .unwrap_or(serde_json::Value::Null),
                ),
                error: None,
            };
        }
    };
    let mut incoming_calls = Vec::new();

    // Find all references to this function across all documents
    let target_name = &params.item.name;

    for (uri, doc) in &store.documents {
        // Search for function calls to this function
        let refs = find_function_references(&doc.content, target_name);
        for range in refs {
            incoming_calls.push(CallHierarchyIncomingCall {
                from: CallHierarchyItem {
                    name: target_name.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: None,
                    uri: uri.clone(),
                    range,
                    selection_range: range,
                    data: None,
                },
                from_ranges: vec![range],
            });
        }
    }

    Response {
        id: req.id,
        result: Some(serde_json::to_value(incoming_calls).unwrap_or(Value::Null)),
        error: None,
    }
}

/// Handle callHierarchy/outgoingCalls requests
pub fn handle_outgoing_calls(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let params: CallHierarchyOutgoingCallsParams = match serde_json::from_value(req.params) {
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
                result: Some(
                    serde_json::to_value(Vec::<CallHierarchyOutgoingCall>::new())
                        .unwrap_or(serde_json::Value::Null),
                ),
                error: None,
            };
        }
    };
    let mut outgoing_calls = Vec::new();

    // Find the function body and extract all function calls
    if let Some(doc) = store.documents.get(&params.item.uri) {
        // Find function calls within the function body
        let calls = find_function_calls_in_range(&doc.content, &params.item.range);

        for (func_name, ranges) in calls {
            // Try to find the definition of the called function
            if let Some(symbol_info) = store.resolve_symbol(doc, &func_name) {
                let target_uri = symbol_info
                    .definition_uri
                    .as_ref()
                    .unwrap_or(&params.item.uri);

                outgoing_calls.push(CallHierarchyOutgoingCall {
                    to: CallHierarchyItem {
                        name: func_name.clone(),
                        kind: symbol_info.kind,
                        tags: None,
                        detail: symbol_info.detail.clone(),
                        uri: target_uri.clone(),
                        range: symbol_info.range,
                        selection_range: symbol_info.selection_range,
                        data: None,
                    },
                    from_ranges: ranges,
                });
            }
        }
    }

    Response {
        id: req.id,
        result: Some(serde_json::to_value(outgoing_calls).unwrap_or(Value::Null)),
        error: None,
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn find_function_references(content: &str, func_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Look for function calls: func_name(...)
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(func_name) {
            let actual_pos = search_pos + pos;
            let after_name = &line[actual_pos + func_name.len()..];

            // Check if it's followed by '(' (function call)
            if after_name.trim_start().starts_with('(') {
                // Find the matching closing paren to get the full range
                let start_char = actual_pos as u32;
                let end_char =
                    if let Some(paren_end) = find_matching_paren(&line[actual_pos..], '(') {
                        (actual_pos + paren_end) as u32
                    } else {
                        (actual_pos + func_name.len()) as u32
                    };

                references.push(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_char,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: end_char,
                    },
                });
            }

            search_pos = actual_pos + func_name.len();
        }
    }

    references
}

fn find_function_calls_in_range(content: &str, range: &Range) -> Vec<(String, Vec<Range>)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut call_map: std::collections::HashMap<String, Vec<Range>> =
        std::collections::HashMap::new();

    // Only search within the specified range
    let start_line = range.start.line as usize;
    let end_line = (range.end.line as usize).min(lines.len());

    for line_idx in start_line..=end_line {
        if let Some(line) = lines.get(line_idx) {
            // Extract function calls from this line
            let func_calls = extract_function_calls_from_line(line, line_idx as u32);
            for (func_name, call_range) in func_calls {
                call_map.entry(func_name).or_default().push(call_range);
            }
        }
    }

    call_map.into_iter().collect()
}

fn extract_function_calls_from_line(line: &str, line_num: u32) -> Vec<(String, Range)> {
    let mut calls: Vec<(String, Range)> = Vec::new();
    let mut chars = line.chars().enumerate().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch.is_alphabetic() || ch == '_' {
            // Potential function name
            let start = idx;
            let mut name = String::from(ch);

            while let Some(&(_, next_ch)) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    if let Some((_, ch)) = chars.next() {
                        name.push(ch);
                    }
                } else {
                    break;
                }
            }

            // Check if followed by '('
            if let Some(&(_, '(')) = chars.peek() {
                // This is a function call
                let name_end = start + name.len();
                let call_end = if let Some(paren_end) = find_matching_paren(&line[name_end..], '(')
                {
                    name_end + paren_end
                } else {
                    name_end
                };

                calls.push((
                    name,
                    Range {
                        start: Position {
                            line: line_num,
                            character: start as u32,
                        },
                        end: Position {
                            line: line_num,
                            character: call_end as u32,
                        },
                    },
                ));
            }
        }
    }

    calls
}

fn find_matching_paren(text: &str, open: char) -> Option<usize> {
    let close = match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };

    let mut depth = 0;
    for (idx, ch) in text.chars().enumerate() {
        match ch {
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx + 1);
                }
            }
            _ => {}
        }
    }

    None
}
