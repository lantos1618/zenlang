// Import-related code actions

use lsp_types::*;
use std::collections::HashMap;

use crate::ast::Declaration;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::stdlib_resolver::StdlibResolver;

use super::utils::extract_symbol_from_diagnostic;

// ============================================================================
// MISSING IMPORT QUICK-FIX
// ============================================================================

pub fn create_missing_import_fix(
    diagnostic: &Diagnostic,
    uri: &Url,
    content: &str,
    store: &std::sync::RwLockReadGuard<'_, DocumentStore>,
) -> Option<CodeAction> {
    let undefined_name = extract_symbol_from_diagnostic(&diagnostic.message);
    if undefined_name.is_empty() {
        return None;
    }

    // Use AST from document to check existing imports
    let doc = store.documents.get(uri);
    let ast = doc.and_then(|d| d.ast.as_ref());

    // Check if symbol exists in stdlib
    if let Some(symbol_info) = store.stdlib_symbols.get(&undefined_name) {
        if let Some(ref def_uri) = symbol_info.definition_uri {
            if let Some(module_path) = get_module_path_from_uri(def_uri, &store.stdlib_resolver) {
                // Check if already imported using AST
                if is_symbol_imported(&undefined_name, ast) {
                    return None;
                }

                // Find import insert position using AST
                let insert_pos = find_import_insert_position(ast, content);
                let import_stmt = format!("{{ {} }} = {}\n", undefined_name, module_path);

                let text_edit = TextEdit {
                    range: Range {
                        start: insert_pos,
                        end: insert_pos,
                    },
                    new_text: import_stmt.clone(),
                };

                let workspace_edit = WorkspaceEdit {
                    changes: Some({
                        let mut changes = HashMap::new();
                        changes.insert(uri.clone(), vec![text_edit]);
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                };

                return Some(CodeAction {
                    title: format!("Import '{}' from {}", undefined_name, module_path),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(workspace_edit),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                });
            }
        }
    }

    // Check workspace symbols
    for (name, symbol_info) in &store.workspace_symbols {
        if name == &undefined_name {
            if let Some(ref def_uri) = symbol_info.definition_uri {
                // Generate relative import path
                let import_path = generate_import_path(uri, def_uri);
                if import_path.is_empty() {
                    continue;
                }

                let insert_pos = find_import_insert_position(ast, content);
                let import_stmt = format!("{{ {} }} = {}\n", undefined_name, import_path);

                let text_edit = TextEdit {
                    range: Range {
                        start: insert_pos,
                        end: insert_pos,
                    },
                    new_text: import_stmt.clone(),
                };

                let workspace_edit = WorkspaceEdit {
                    changes: Some({
                        let mut changes = HashMap::new();
                        changes.insert(uri.clone(), vec![text_edit]);
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                };

                return Some(CodeAction {
                    title: format!("Import '{}' from {}", undefined_name, import_path),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(workspace_edit),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                });
            }
        }
    }

    None
}

/// Check if a symbol is already imported using AST declarations.
/// The parser converts destructured imports like `{ io, math } = @std` into
/// individual `Declaration::ModuleImport` entries, so AST check is comprehensive.
fn is_symbol_imported(symbol: &str, ast: Option<&Vec<Declaration>>) -> bool {
    if let Some(declarations) = ast {
        for decl in declarations {
            if let Declaration::ModuleImport { alias, .. } = decl {
                if alias == symbol {
                    return true;
                }
            }
        }
    }
    false
}

/// Find the position to insert a new import statement.
/// Uses AST spans to find the last import declaration, falling back to after top comments.
pub fn find_import_insert_position(ast: Option<&Vec<Declaration>>, content: &str) -> Position {
    // Use AST to find the last import declaration's line
    if let Some(declarations) = ast {
        let mut last_import_line: Option<u32> = None;
        for decl in declarations {
            if let Declaration::ModuleImport { span: Some(s), .. } = decl {
                last_import_line = Some(s.line as u32);
            }
        }
        if let Some(line) = last_import_line {
            return Position {
                line: line + 1, // Insert after the last import
                character: 0,
            };
        }
    }

    // Fallback: skip past leading comments
    let mut insert_line = 0u32;
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            insert_line = (i + 1) as u32;
        } else if !trimmed.is_empty() {
            break;
        }
    }

    Position {
        line: insert_line,
        character: 0,
    }
}

fn generate_import_path(from_uri: &Url, to_uri: &Url) -> String {
    // Generate a relative import path from one file to another
    let from_path = from_uri.path();
    let to_path = to_uri.path();

    // Find common prefix
    let from_parts: Vec<&str> = from_path.split('/').collect();
    let to_parts: Vec<&str> = to_path.split('/').collect();

    let mut common_idx = 0;
    for (i, (a, b)) in from_parts.iter().zip(to_parts.iter()).enumerate() {
        if a == b {
            common_idx = i + 1;
        } else {
            break;
        }
    }

    // Build relative path
    let ups = from_parts.len() - common_idx - 1; // -1 for the filename
    let mut path_parts: Vec<String> = Vec::new();

    // Add parent directory markers
    for _ in 0..ups {
        path_parts.push("@parent".to_string());
    }

    // Add remaining path components (without .zen extension)
    for part in &to_parts[common_idx..] {
        let clean_part = part.trim_end_matches(".zen");
        if !clean_part.is_empty() {
            path_parts.push(clean_part.to_string());
        }
    }

    if path_parts.is_empty() {
        return String::new();
    }

    // Join with dots for Zen import syntax
    format!("@this.{}", path_parts.join("."))
}

// ============================================================================
// HELPER FUNCTION (from completion module)
// ============================================================================

/// Convert a URI to a module path using StdlibResolver for stdlib detection.
pub fn get_module_path_from_uri(uri: &Url, stdlib_resolver: &StdlibResolver) -> Option<String> {
    let path = std::path::Path::new(uri.path());
    stdlib_resolver.path_to_module_path(path)
}
