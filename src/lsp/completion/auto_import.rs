//! Auto-import helpers
//!
//! Provides functionality for automatically adding import statements when completing symbols.

use lsp_types::*;
use std::collections::HashSet;

use crate::lsp::types::SymbolInfo;
use crate::lsp::utils::symbol_kind_to_completion_kind;

/// Extract the module path from a definition URI (e.g., @std.collections.vec)
pub fn get_module_path_from_uri(uri: &Url) -> Option<String> {
    let path = uri.path();

    if let Some(stdlib_pos) = path.find("/stdlib/") {
        let relative = &path[stdlib_pos + 8..];
        let module_path = relative.trim_end_matches(".zen").replace('/', ".");
        return Some(format!("@std.{}", module_path));
    }

    None
}

/// Get symbols that are already imported in the document
pub fn get_imported_symbols(content: &str) -> HashSet<String> {
    let mut imported = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Match: { symbol1, symbol2 } = @std.module
        if trimmed.starts_with('{') && trimmed.contains("} =") {
            if let Some(brace_end) = trimmed.find('}') {
                let symbols_str = &trimmed[1..brace_end];
                for symbol in symbols_str.split(',') {
                    imported.insert(symbol.trim().to_string());
                }
            }
        }
    }

    imported
}

/// Find the best position to insert an import statement
pub fn find_import_insert_position(content: &str) -> Position {
    let mut last_import_line = 0;
    let mut found_import = false;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") {
            if !found_import {
                last_import_line = line_num + 1;
            }
            continue;
        }

        if trimmed.starts_with('{') && trimmed.contains("} =") && trimmed.contains('@') {
            last_import_line = line_num + 1;
            found_import = true;
        } else if !trimmed.is_empty() {
            break;
        }
    }

    Position {
        line: last_import_line as u32,
        character: 0,
    }
}

/// Create a TextEdit for inserting an import statement
pub fn create_import_edit(symbol_name: &str, module_path: &str, content: &str) -> Option<TextEdit> {
    let imported = get_imported_symbols(content);
    if imported.contains(symbol_name) {
        return None;
    }

    let insert_pos = find_import_insert_position(content);
    let import_line = format!("{{ {} }} = {}\n", symbol_name, module_path);

    Some(TextEdit {
        range: Range {
            start: insert_pos,
            end: insert_pos,
        },
        new_text: import_line,
    })
}

/// Create a completion item with auto-import
pub fn create_completion_with_import(
    name: &str,
    symbol: &SymbolInfo,
    module_path: &str,
    content: &str,
) -> CompletionItem {
    let mut item = CompletionItem {
        label: name.to_string(),
        kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
        detail: symbol.detail.clone(),
        documentation: Some(Documentation::String(format!(
            "Auto-import from `{}`",
            module_path
        ))),
        ..Default::default()
    };

    if let Some(edit) = create_import_edit(name, module_path, content) {
        item.additional_text_edits = Some(vec![edit]);
        item.label_details = Some(CompletionItemLabelDetails {
            detail: Some(format!(" ({})", module_path)),
            description: None,
        });
    }

    item
}
