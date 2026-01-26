// Rename Module for Zen LSP
// Handles textDocument/rename requests

use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::{Declaration, Statement};

use super::document_store::DocumentStore;
use super::helpers::{null_response, success_response, try_lock, try_parse_params};
use super::navigation::find_symbol_at_position;
use super::navigation::utils::{find_function_range, find_function_range_from_doc};
use super::types::{Document, SymbolScope};

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_rename(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: RenameParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

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

// Use find_function_range from navigation::utils

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
