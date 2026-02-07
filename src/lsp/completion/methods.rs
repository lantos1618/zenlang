//! Method and field completions
//!
//! Handles UFC method completions and struct field access completions.

use lsp_types::*;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::utils::format_type;
use crate::name_utils;

/// Get struct field completions for dot access (e.g., `person.` -> name, age)
pub fn get_struct_field_completions(
    receiver_type: &str,
    store: &DocumentStore,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Extract struct name from type string
    let struct_name = name_utils::strip_generics(receiver_type).trim();

    if let Some(struct_def) = store.find_struct_definition(struct_name) {
        for field in &struct_def.fields {
            items.push(CompletionItem {
                label: field.name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("{}: {}", field.name, format_type(&field.type_))),
                documentation: Some(Documentation::String(format!(
                    "Field of `{}` struct",
                    struct_name
                ))),
                ..Default::default()
            });
        }
    }

    items
}

/// Get UFC method completions for a receiver type.
/// TODO: Reimplement using TypeContext/TypeQuery once stdlib is indexed through SEMA.
pub fn get_ufc_method_completions(
    _receiver_type: &str,
    _store: &DocumentStore,
) -> Vec<CompletionItem> {
    Vec::new()
}
