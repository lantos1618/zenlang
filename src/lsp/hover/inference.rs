use std::collections::HashMap;

use crate::ast::Declaration;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::type_query::TypeQuery;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;

pub fn extract_type_from_line(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let after_colon = &line[colon_pos + 1..];
        let type_end = after_colon
            .find('=')
            .or_else(|| after_colon.find('{'))
            .or_else(|| after_colon.find('('))
            .unwrap_or(after_colon.len());
        let type_str = after_colon[..type_end].trim();
        if !type_str.is_empty() {
            return Some(type_str.to_string());
        }
    }
    None
}

fn format_variable_hover(var_name: &str, type_str: &str) -> String {
    format!(
        "```zen\n{}: {}\n```\n\n**Type:** `{}`",
        var_name, type_str, type_str
    )
}

pub fn infer_variable_type(
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    for doc in store.documents.values() {
        let tq = TypeQuery::new(doc);

        if let Some(ast) = &doc.ast {
            let stmts: Vec<_> = ast
                .iter()
                .filter_map(|decl| {
                    if let Declaration::Function(func) = decl {
                        Some(func.body.as_slice())
                    } else {
                        None
                    }
                })
                .flatten()
                .cloned()
                .collect();

            if let Some(ty) = tq.infer_variable_type_from_statements(var_name, &stmts) {
                return Some(format_variable_hover(var_name, &format_type(&ty)));
            }
        } else if let Some(ty) = tq.find_variable_type_ast(var_name) {
            return Some(format_variable_hover(var_name, &format_type(&ty)));
        }
    }

    if let Some(sym) = store.resolve_symbol_local_first(local_symbols, var_name) {
        if let Some(type_info) = &sym.type_info {
            return Some(format_variable_hover(var_name, &format_type(type_info)));
        }
    }

    None
}
