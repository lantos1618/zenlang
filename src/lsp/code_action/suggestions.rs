// "Did you mean" suggestions for undefined symbols

use lsp_types::*;
use std::collections::HashMap;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::Document;

use super::utils::{extract_symbol_from_diagnostic, levenshtein_distance};

// ============================================================================
// "DID YOU MEAN" QUICK-FIX FOR UNDEFINED SYMBOLS
// ============================================================================

pub fn create_did_you_mean_actions(
    diagnostic: &Diagnostic,
    uri: &Url,
    doc: &Document,
    store: &std::sync::MutexGuard<'_, DocumentStore>,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    // Extract the undefined symbol name from the diagnostic
    let undefined_name = extract_symbol_from_diagnostic(&diagnostic.message);
    if undefined_name.is_empty() {
        return actions;
    }

    // Collect all known symbols
    let mut candidates: Vec<(String, u32)> = Vec::new();

    // Add symbols from current document
    for name in doc.symbols.keys() {
        let dist = levenshtein_distance(&undefined_name, name);
        if dist <= 3 && dist > 0 {
            // Allow up to 3 edits, but not exact match
            candidates.push((name.clone(), dist));
        }
    }

    // Add symbols from workspace
    for name in store.workspace_symbols.keys() {
        let dist = levenshtein_distance(&undefined_name, name);
        if dist <= 3 && dist > 0 {
            candidates.push((name.clone(), dist));
        }
    }

    // Add symbols from stdlib
    for name in store.stdlib_symbols.keys() {
        let dist = levenshtein_distance(&undefined_name, name);
        if dist <= 3 && dist > 0 {
            candidates.push((name.clone(), dist));
        }
    }

    // Sort by distance and take top 3
    candidates.sort_by_key(|(_, dist)| *dist);
    candidates.dedup_by(|(a, _), (b, _)| a == b);
    candidates.truncate(3);

    for (suggestion, _) in candidates {
        let text_edit = TextEdit {
            range: diagnostic.range,
            new_text: suggestion.clone(),
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

        actions.push(CodeAction {
            title: format!("Did you mean '{}'?", suggestion),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(workspace_edit),
            command: None,
            is_preferred: Some(actions.is_empty()), // First suggestion is preferred
            disabled: None,
            data: None,
        });
    }

    actions
}
