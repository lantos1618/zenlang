// Rename Module for Zen LSP
// Handles textDocument/rename and textDocument/prepareRename requests

use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::primitives;
use crate::ast::{Declaration, Statement};
use crate::lexer::{Lexer, Token};

use super::document_store::DocumentStore;
use super::helpers::{null_response, success_response, try_lock, try_parse_params, with_document};
use super::navigation::find_symbol_at_position;
use super::navigation::utils::{find_function_range, find_function_range_from_doc};
use super::types::{Document, SymbolScope};

// ============================================================================
// IDENTIFIER VALIDATION USING LEXER
// ============================================================================

/// Check if a name is a valid Zen identifier that can be used/renamed.
/// Uses the lexer to properly tokenize and validate.
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Use the lexer to tokenize the name
    let mut lexer = Lexer::new(name);
    let token = lexer.next_token_with_span();

    // Must be an Identifier token and consume the entire input
    matches!(token.token, Token::Identifier(_)) && lexer.next_token_with_span().token == Token::Eof
}

/// Check if a name is a keyword or special token that cannot be renamed.
/// Returns an error message if it cannot be renamed, None if it's valid.
fn check_rename_target(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Empty name".to_string());
    }

    // Use the lexer to tokenize
    let mut lexer = Lexer::new(name);
    let token = lexer.next_token_with_span();

    match &token.token {
        // Actual keyword tokens in Zen
        Token::Pub => Some("Cannot rename keyword 'pub'".to_string()),
        Token::AtStd => Some("Cannot rename builtin '@std'".to_string()),
        Token::AtThis => Some("Cannot rename builtin '@this'".to_string()),
        Token::AtMeta => Some("Cannot rename builtin '@meta'".to_string()),
        Token::AtExport => Some("Cannot rename builtin '@export'".to_string()),
        Token::AtBuiltin => Some("Cannot rename builtin '@builtin'".to_string()),

        // Numeric literals
        Token::Integer(_) | Token::Float(_) => Some("Cannot rename numeric literal".to_string()),

        // String literals
        Token::StringLiteral(_) => Some("Cannot rename string literal".to_string()),

        // Identifiers are generally valid, but check for reserved names
        // using the centralized definitions
        Token::Identifier(ident) => {
            // Use shared reserved identifier check (primitives, literals, self)
            if primitives::is_reserved_identifier(ident) {
                if primitives::is_primitive_name(ident) {
                    return Some(format!("Cannot rename primitive type '{}'", ident));
                }
                if primitives::is_literal_identifier(ident) {
                    return Some(format!("Cannot rename literal '{}'", ident));
                }
                // self/Self
                return Some(format!("Cannot rename '{}'", ident));
            }

            // Valid identifier
            None
        }

        // Any other token type cannot be renamed
        _ => Some(format!("Cannot rename '{}'", name)),
    }
}

/// Validate a new name for an identifier.
/// Returns an error message if invalid, None if valid.
pub fn validate_new_name(new_name: &str) -> Option<String> {
    if new_name.is_empty() {
        return Some("New name cannot be empty".to_string());
    }

    // Must be a valid identifier according to the lexer
    if !is_valid_identifier(new_name) {
        return Some(format!("'{}' is not a valid identifier", new_name));
    }

    // Check if it's a reserved name
    if let Some(error) = check_rename_target(new_name) {
        // Rephrase error for "new name" context
        if error.contains("Cannot rename") {
            return Some(format!(
                "'{}' cannot be used as an identifier name",
                new_name
            ));
        }
        return Some(error);
    }

    None
}

// ============================================================================
// PREPARE RENAME HANDLER
// ============================================================================

pub fn handle_prepare_rename(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    with_document::<TextDocumentPositionParams, _>(&req, store, |doc, params, _store| {
        let position = params.position;

        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            if let Some(error) = check_rename_target(&symbol_name) {
                return crate::lsp::helpers::error_response_id(
                    req.id.clone(),
                    lsp_server::ErrorCode::InvalidRequest,
                    error,
                );
            }

            let lines: Vec<&str> = doc.content.lines().collect();
            if let Some(line) = lines.get(position.line as usize) {
                if let Some(start) =
                    find_symbol_start(line, position.character as usize, &symbol_name)
                {
                    let range = Range {
                        start: Position {
                            line: position.line,
                            character: start as u32,
                        },
                        end: Position {
                            line: position.line,
                            character: (start + symbol_name.len()) as u32,
                        },
                    };

                    let response = PrepareRenameResponse::RangeWithPlaceholder {
                        range,
                        placeholder: symbol_name,
                    };

                    return Response {
                        id: req.id.clone(),
                        result: Some(serde_json::to_value(response).unwrap_or(Value::Null)),
                        error: None,
                    };
                }
            }
        }

        null_response(&req)
    })
}

/// Find the start position of a symbol in a line
fn find_symbol_start(line: &str, cursor_pos: usize, symbol_name: &str) -> Option<usize> {
    // Search backwards from cursor to find symbol start
    let mut pos = 0;
    while let Some(found) = line[pos..].find(symbol_name) {
        let start = pos + found;
        let end = start + symbol_name.len();

        // Check word boundaries
        let before_ok = start == 0 || !line.chars().nth(start - 1).unwrap_or(' ').is_alphanumeric();
        let after_ok = end >= line.len() || !line.chars().nth(end).unwrap_or(' ').is_alphanumeric();

        if before_ok && after_ok && cursor_pos >= start && cursor_pos <= end {
            return Some(start);
        }
        pos = start + 1;
        if pos >= line.len() {
            break;
        }
    }
    None
}

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_rename(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: RenameParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    // Validate the new name before proceeding
    if let Some(error) = validate_new_name(&params.new_name) {
        return crate::lsp::helpers::error_response_id(
            req.id,
            lsp_server::ErrorCode::InvalidParams,
            error,
        );
    }

    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, WorkspaceEdit::default()),
    };
    let new_name = params.new_name;
    let uri = &params.text_document_position.text_document.uri;

    if let Some(doc) = store.documents.get(uri) {
        let position = params.text_document_position.position;

        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            log::debug!(
                "[LSP] Rename: symbol='{}' -> '{}' at {}:{}",
                symbol_name,
                new_name,
                position.line,
                position.character
            );

            // Determine the scope of the symbol
            let symbol_scope = determine_symbol_scope(doc, &symbol_name, position);

            log::debug!("[LSP] Symbol scope: {:?}", symbol_scope);

            let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

            match symbol_scope {
                SymbolScope::Local { function_name } => {
                    // Local variable or parameter - only rename in current file, within function
                    log::debug!(
                        "[LSP] Renaming local symbol in function '{}'",
                        function_name
                    );

                    if let Some(edits) =
                        rename_local_symbol(&doc.content, &symbol_name, &new_name, &function_name)
                    {
                        if !edits.is_empty() {
                            changes.insert(uri.clone(), edits);
                        }
                    }
                }
                SymbolScope::ModuleLevel => {
                    // Module-level symbol (function, struct, enum) - rename across workspace
                    log::debug!("[LSP] Renaming module-level symbol across workspace");

                    // Find all workspace files that might reference this symbol
                    let workspace_files = collect_workspace_files(&store);

                    log::debug!(
                        "[LSP] Scanning {} workspace files for references",
                        workspace_files.len()
                    );

                    for (file_uri, file_content) in workspace_files {
                        if let Some(edits) = rename_in_file(&file_content, &symbol_name, &new_name)
                        {
                            if !edits.is_empty() {
                                log::debug!(
                                    "[LSP] Found {} occurrences in {}",
                                    edits.len(),
                                    file_uri.path()
                                );
                                changes.insert(file_uri, edits);
                            }
                        }
                    }
                }
                SymbolScope::Unknown => {
                    // Fallback: only rename in current file
                    log::debug!("[LSP] Unknown scope, renaming only in current file");

                    if let Some(edits) = rename_in_file(&doc.content, &symbol_name, &new_name) {
                        if !edits.is_empty() {
                            changes.insert(uri.clone(), edits);
                        }
                    }
                }
            }

            log::debug!(
                "[LSP] Rename will affect {} files with {} total edits",
                changes.len(),
                changes.values().map(|v| v.len()).sum::<usize>()
            );

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };

            return Response {
                id: req.id,
                result: Some(serde_json::to_value(workspace_edit).unwrap_or(Value::Null)),
                error: None,
            };
        }
    }

    null_response(&req)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub(crate) fn determine_symbol_scope(
    doc: &Document,
    symbol_name: &str,
    position: Position,
) -> SymbolScope {
    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let Declaration::Function(func) = decl {
                if let Some(func_range) = find_function_range_from_doc(doc, &func.name) {
                    if position.line >= func_range.start.line
                        && position.line <= func_range.end.line
                        && is_local_symbol_in_function(func, symbol_name)
                    {
                        return SymbolScope::Local {
                            function_name: func.name.clone(),
                        };
                    }
                }
            }
        }
    }

    if doc.symbols.contains_key(symbol_name) {
        return SymbolScope::ModuleLevel;
    }

    SymbolScope::Unknown
}

fn is_local_symbol_in_function(func: &crate::ast::Function, symbol_name: &str) -> bool {
    for (param_name, _param_type) in &func.args {
        if param_name == symbol_name {
            return true;
        }
    }

    is_symbol_in_statements(&func.body, symbol_name)
}

fn is_symbol_in_statements(statements: &[Statement], symbol_name: &str) -> bool {
    for stmt in statements {
        match stmt {
            Statement::VariableDeclaration { name, .. } if name == symbol_name => {
                return true;
            }
            Statement::Loop { body, .. } => {
                if is_symbol_in_statements(body, symbol_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn rename_local_symbol(
    content: &str,
    symbol_name: &str,
    new_name: &str,
    function_name: &str,
) -> Option<Vec<TextEdit>> {
    let func_range = find_function_range(content, function_name)?;
    let mut edits = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line_num in func_range.start.line..=func_range.end.line {
        if line_num as usize >= lines.len() {
            break;
        }

        let line = lines[line_num as usize];
        let mut start_col = 0;

        while let Some(col) = line[start_col..].find(symbol_name) {
            let actual_col = start_col + col;

            let before_ok = actual_col == 0
                || !line
                    .chars()
                    .nth(actual_col - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            let after_ok = actual_col + symbol_name.len() >= line.len()
                || !line
                    .chars()
                    .nth(actual_col + symbol_name.len())
                    .unwrap_or(' ')
                    .is_alphanumeric();

            if before_ok && after_ok {
                edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: line_num,
                            character: actual_col as u32,
                        },
                        end: Position {
                            line: line_num,
                            character: (actual_col + symbol_name.len()) as u32,
                        },
                    },
                    new_text: new_name.to_string(),
                });
            }

            start_col = actual_col + 1;
        }
    }

    Some(edits)
}

fn collect_workspace_files(store: &DocumentStore) -> Vec<(Url, String)> {
    use std::path::PathBuf;

    fn collect_zen_files_recursive(
        dir: &std::path::Path,
        files: &mut Vec<PathBuf>,
        max_depth: usize,
    ) {
        if max_depth == 0 {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip hidden directories and target/
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" {
                        continue;
                    }
                }

                if path.is_dir() {
                    collect_zen_files_recursive(&path, files, max_depth - 1);
                } else if path.extension().and_then(|e| e.to_str()) == Some("zen") {
                    if let Ok(canonical) = path.canonicalize() {
                        files.push(canonical);
                    }
                }
            }
        }
    }

    let mut result = Vec::new();

    // Add all open documents
    for (uri, doc) in &store.documents {
        result.push((uri.clone(), doc.content.clone()));
    }

    // Add all workspace files recursively
    if let Some(workspace_root) = &store.workspace_root {
        if let Ok(root_path) = workspace_root.to_file_path() {
            let mut zen_files = Vec::new();
            collect_zen_files_recursive(&root_path, &mut zen_files, 5);

            for path in zen_files {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(uri) = Url::from_file_path(&path) {
                        // Only add if not already in open documents
                        if !store.documents.contains_key(&uri) {
                            result.push((uri, content));
                        }
                    }
                }
            }
        }
    }

    result
}

fn rename_in_file(content: &str, symbol_name: &str, new_name: &str) -> Option<Vec<TextEdit>> {
    let mut edits = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let mut start_col = 0;

        while let Some(col) = line[start_col..].find(symbol_name) {
            let actual_col = start_col + col;

            let before_ok = actual_col == 0
                || !line
                    .chars()
                    .nth(actual_col - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            let after_ok = actual_col + symbol_name.len() >= line.len()
                || !line
                    .chars()
                    .nth(actual_col + symbol_name.len())
                    .unwrap_or(' ')
                    .is_alphanumeric();

            if before_ok && after_ok {
                edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: actual_col as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (actual_col + symbol_name.len()) as u32,
                        },
                    },
                    new_text: new_name.to_string(),
                });
            }

            start_col = actual_col + 1;
        }
    }

    Some(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_identifier() {
        // Valid identifiers
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("my_var123"));
        assert!(is_valid_identifier("CamelCase"));

        // Invalid identifiers
        assert!(!is_valid_identifier("")); // empty
        assert!(!is_valid_identifier("123")); // starts with number
        assert!(!is_valid_identifier("foo bar")); // space
        assert!(!is_valid_identifier("pub")); // keyword
    }

    #[test]
    fn test_check_rename_target() {
        // Keywords cannot be renamed
        assert!(check_rename_target("pub").is_some());

        // Builtins cannot be renamed
        assert!(check_rename_target("@std").is_some());
        assert!(check_rename_target("@this").is_some());
        assert!(check_rename_target("@meta").is_some());

        // Primitives cannot be renamed
        assert!(check_rename_target("i32").is_some());
        assert!(check_rename_target("bool").is_some());
        assert!(check_rename_target("void").is_some());

        // Literals cannot be renamed
        assert!(check_rename_target("true").is_some());
        assert!(check_rename_target("false").is_some());
        assert!(check_rename_target("null").is_some());

        // Regular identifiers CAN be renamed
        assert!(check_rename_target("my_function").is_none());
        assert!(check_rename_target("MyStruct").is_none());
        assert!(check_rename_target("foo").is_none());

        // Note: fn, struct, enum, etc. ARE valid identifiers in Zen!
        // They only have special meaning in parser context
        assert!(check_rename_target("fn").is_none());
        assert!(check_rename_target("struct").is_none());
        assert!(check_rename_target("enum").is_none());
    }

    #[test]
    fn test_validate_new_name() {
        // Valid new names
        assert!(validate_new_name("new_name").is_none());
        assert!(validate_new_name("_private").is_none());
        assert!(validate_new_name("MyType").is_none());

        // Invalid new names
        assert!(validate_new_name("").is_some()); // empty
        assert!(validate_new_name("123abc").is_some()); // starts with number
        assert!(validate_new_name("pub").is_some()); // keyword
        assert!(validate_new_name("i32").is_some()); // primitive type
        assert!(validate_new_name("true").is_some()); // literal
    }
}
