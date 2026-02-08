// Import-related code actions

use lsp_types::*;
use std::collections::HashMap;

use crate::ast::Declaration;
use crate::lsp::document_store::DocumentStore;

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
            if let Some(module_path) = get_module_path_from_uri(def_uri) {
                // Check if already imported using AST
                if is_symbol_imported(&undefined_name, ast, content) {
                    return None;
                }

                // Find import insert position
                let insert_pos = find_import_insert_position(content);
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

                let insert_pos = find_import_insert_position(content);
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

/// Check if a symbol is already imported, using AST if available, falling back to content scan.
fn is_symbol_imported(symbol: &str, ast: Option<&Vec<Declaration>>, content: &str) -> bool {
    // Prefer AST-based check
    if let Some(declarations) = ast {
        for decl in declarations {
            if let Declaration::ModuleImport { alias, .. } = decl {
                if alias == symbol {
                    return true;
                }
            }
        }
        // Also check for the symbol in the document's symbol table would be ideal,
        // but ModuleImport with destructuring is represented differently.
        // Fall through to content check for destructured imports like { io, math } = @std
    }

    // Fallback: scan content for destructured import patterns containing the symbol.
    // This is still needed because destructured imports (e.g., `{ io, math } = @std`)
    // are parsed as separate statements, not always as Declaration::ModuleImport.
    // We use a targeted line-by-line check rather than fragile substring matching.
    for line in content.lines() {
        let trimmed = line.trim();
        // Match lines like: { symbol, ... } = @... or { ..., symbol } = @...
        if trimmed.starts_with('{') && trimmed.contains("} =") {
            // Extract the names between { and }
            if let Some(brace_end) = trimmed.find('}') {
                let names_section = &trimmed[1..brace_end];
                for name in names_section.split(',') {
                    if name.trim() == symbol {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn find_import_insert_position(content: &str) -> Position {
    let lines: Vec<&str> = content.lines().collect();
    let mut last_import_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Check for import pattern: { ... } = @...
        if trimmed.starts_with('{') && trimmed.contains("} =") && trimmed.contains('@') {
            last_import_line = i + 1;
        }
        // Skip comments at the top
        if trimmed.starts_with("//") && last_import_line == 0 {
            last_import_line = i + 1;
        }
    }

    Position {
        line: last_import_line as u32,
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
// ADD IMPORT ACTION
// ============================================================================

pub fn create_add_import_action(uri: &Url, content: &str) -> Option<CodeAction> {
    // Check if io/allocator symbols are used but not imported.
    // We still check content for usage (since that's what the user typed),
    // but use structured line-by-line import detection instead of fragile substring matching.
    let has_std_import = content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with('{') && trimmed.contains("} =") && trimmed.contains("@std")
    });

    let has_io_import = content.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.contains("} =") {
            if let Some(brace_end) = trimmed.find('}') {
                let names = &trimmed[1..brace_end];
                return names.split(',').any(|n| n.trim() == "io");
            }
        }
        false
    });

    let needs_io = content.contains("io.") && !has_io_import;
    let needs_allocator = (content.contains("get_default_allocator")
        || content.contains("GPA")
        || content.contains("AsyncPool"))
        && !has_std_import;

    if !needs_io && !needs_allocator {
        return None;
    }

    let import_statement = if needs_io && needs_allocator {
        "{ io, GPA, AsyncPool } = @std\n"
    } else if needs_io {
        "{ io } = @std\n"
    } else {
        "{ GPA, AsyncPool } = @std\n"
    };

    let insert_pos = find_import_insert_position(content);

    let text_edit = TextEdit {
        range: Range {
            start: insert_pos,
            end: insert_pos,
        },
        new_text: import_statement.to_string(),
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

    Some(CodeAction {
        title: format!("Add import: {}", import_statement.trim()),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

// ============================================================================
// HELPER FUNCTION (from completion module)
// ============================================================================

pub fn get_module_path_from_uri(uri: &Url) -> Option<String> {
    let path = uri.path();

    // Check if it's a stdlib path
    if path.contains("/std/") || path.contains("/stdlib/") {
        // Extract module name from stdlib path
        if let Some(std_idx) = path.rfind("/std/").or_else(|| path.rfind("/stdlib/")) {
            let after_std = &path[std_idx + 5..]; // Skip "/std/"
            let module_name = after_std.trim_end_matches(".zen").replace('/', ".");
            return Some(format!("@std.{}", module_name));
        }

        // Handle top-level std files
        if let Some(file_name) = path.split('/').next_back() {
            let module_name = file_name.trim_end_matches(".zen");
            if !module_name.is_empty() {
                return Some(format!("@std.{}", module_name));
            }
        }
    }

    None
}
