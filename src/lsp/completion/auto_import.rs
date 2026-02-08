//! Auto-import helpers
//!
//! Provides functionality for automatically adding import statements when completing symbols.

use lsp_types::*;
use std::collections::HashSet;

use crate::ast::Declaration;
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

/// Get symbols that are already imported in the document using AST declarations.
pub fn get_imported_symbols_from_ast(ast: &[Declaration]) -> HashSet<String> {
    let mut imported = HashSet::new();
    for decl in ast {
        if let Declaration::ModuleImport { alias, .. } = decl {
            imported.insert(alias.clone());
        }
    }
    imported
}

/// Find the best position to insert an import statement.
/// Uses the AST to find the last import declaration's span.
pub fn find_import_insert_position_from_ast(ast: &[Declaration]) -> Option<Position> {
    let mut last_import_line = None;
    for decl in ast {
        if let Declaration::ModuleImport { span: Some(s), .. } = decl {
            // span.line is 1-based from the parser
            let line = s.line.saturating_sub(1);
            last_import_line = Some(line);
        }
    }
    last_import_line.map(|line| Position {
        line: (line + 1) as u32,
        character: 0,
    })
}

/// Create a TextEdit for inserting an import statement.
/// Requires AST for analysis; returns None if AST is unavailable.
pub fn create_import_edit(
    symbol_name: &str,
    module_path: &str,
    ast: Option<&Vec<Declaration>>,
) -> Option<TextEdit> {
    let ast = ast?;
    let imported = get_imported_symbols_from_ast(ast);
    if imported.contains(symbol_name) {
        return None;
    }

    let insert_pos = find_import_insert_position_from_ast(ast)?;
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
    ast: Option<&Vec<Declaration>>,
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

    if let Some(edit) = create_import_edit(name, module_path, ast) {
        item.additional_text_edits = Some(vec![edit]);
        item.label_details = Some(CompletionItemLabelDetails {
            detail: Some(format!(" ({})", module_path)),
            description: None,
        });
    }

    item
}
