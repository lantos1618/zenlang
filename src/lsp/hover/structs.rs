// Struct-related hover functionality

use std::collections::HashMap;

use crate::ast::{AstType, Declaration};
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

/// Find struct definition by name in documents
pub fn find_struct_definition(
    struct_name: &str,
    doc: &Document,
    store: &DocumentStore,
) -> Option<crate::ast::StructDefinition> {
    // First check current document AST
    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let Declaration::Struct(struct_def) = decl {
                if struct_def.name == struct_name {
                    return Some(struct_def.clone());
                }
            }
        }
    }

    // Check other documents in the store
    for other_doc in store.documents.values() {
        if let Some(ast) = &other_doc.ast {
            for decl in ast {
                if let Declaration::Struct(struct_def) = decl {
                    if struct_def.name == struct_name {
                        return Some(struct_def.clone());
                    }
                }
            }
        }
    }

    None
}

/// Find struct definition in documents map
pub fn find_struct_definition_in_documents(
    struct_name: &str,
    documents: &HashMap<lsp_types::Url, Document>,
) -> Option<crate::ast::StructDefinition> {
    for doc in documents.values() {
        if let Some(ast) = &doc.ast {
            for decl in ast {
                if let Declaration::Struct(struct_def) = decl {
                    if struct_def.name == struct_name {
                        return Some(struct_def.clone());
                    }
                }
            }
        }
    }
    None
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
    if let Some(var_info) = local_symbols
        .get(var_name)
        .or_else(|| store.workspace_symbols.get(var_name))
        .or_else(|| store.stdlib_symbols.get(var_name))
    {
        if let Some(AstType::Struct { name, .. }) = &var_info.type_info {
            if let Some(struct_def) = find_struct_definition_in_documents(name, &store.documents) {
                return Some(format!(
                    "```zen\n{}\n```",
                    format_struct_definition(&struct_def)
                ));
            }
        }
    }
    None
}
