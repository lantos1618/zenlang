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

    // Find struct definition in documents
    use crate::lsp::hover::find_struct_definition_in_documents;
    if let Some(struct_def) = find_struct_definition_in_documents(struct_name, &store.documents) {
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

/// Get UFC method completions for a receiver type
pub fn get_ufc_method_completions(
    receiver_type: &str,
    store: &DocumentStore,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    for sig in store.compiler.get_methods_for_type(receiver_type) {
        items.push(CompletionItem {
            label: sig.name.clone(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(format!(
                "({}) -> {}",
                sig.params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, format_type(t)))
                    .collect::<Vec<_>>()
                    .join(", "),
                format_type(&sig.return_type)
            )),
            ..Default::default()
        });
    }

    items
}
