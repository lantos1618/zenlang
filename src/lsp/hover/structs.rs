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

/// Handle hover on a variable that might be a struct
pub fn handle_variable_hover(
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    if let Some(var_info) = store.resolve_symbol_local_first(local_symbols, var_name) {
        if let Some(type_info) = &var_info.type_info {
            if matches!(type_info, AstType::Struct { .. }) {
                if let Some(name) = type_info.base_name() {
                    if let Some(struct_def) = store.find_struct_definition(name) {
                        return Some(format!(
                            "```zen\n{}\n```",
                            format_struct_definition(&struct_def)
                        ));
                    }
                }
            }
        }
    }
    None
}
