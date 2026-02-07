// Struct-related hover functionality

use std::collections::HashMap;

use crate::ast::AstType;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;

/// Format struct definition with fields for display
pub fn format_struct_definition(struct_def: &crate::ast::StructDefinition) -> String {
    let mut result = format!("{} {{\n", struct_def.name);
    for field in &struct_def.fields {
        result.push_str(&format!(
            "    {}: {},\n",
            field.name,
            format_type(&field.type_)
        ));
    }
    result.push('}');
    result
}

/// Extract struct name from type string
pub fn extract_struct_name_from_type(type_str: &str) -> Option<String> {
    // Look for patterns like "Type: `Person`" or "Type: `Person`\n"
    if let Some(start) = type_str.find("**Type:** `") {
        let after_type = &type_str[start + 11..];
        if let Some(end) = after_type.find('`') {
            let struct_name = after_type[..end].to_string();
            // Check if it looks like a struct name (starts with uppercase)
            if struct_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                return Some(struct_name);
            }
        }
    }
    None
}

/// Handle hover on a variable that might be a struct
pub fn handle_variable_hover(
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    if let Some(var_info) = store.resolve_symbol_local_first(local_symbols, var_name) {
        if let Some(AstType::Struct { name, .. }) = &var_info.type_info {
            if let Some(struct_def) = store.find_struct_definition(name) {
                return Some(format!(
                    "```zen\n{}\n```",
                    format_struct_definition(&struct_def)
                ));
            }
        }
    }
    None
}
