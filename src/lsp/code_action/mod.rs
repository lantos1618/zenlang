// Code Action Module for Zen LSP
// Handles textDocument/codeAction requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, Mutex};

use super::document_store::DocumentStore;
use super::helpers::{success_response, try_lock, try_parse_params};

mod imports;
mod quick_fixes;
mod refactorings;
mod suggestions;
mod utils;

// Re-export for backward compatibility if needed
pub use imports::*;
pub use quick_fixes::*;
pub use refactorings::*;
pub use suggestions::*;
pub use utils::*;

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_code_action(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: CodeActionParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let mut actions = Vec::new();
    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, Vec::<CodeActionOrCommand>::new()),
    };

    if let Some(doc) = store.documents.get(&params.text_document.uri) {
        // Check diagnostics in the requested range
        for diagnostic in &params.context.diagnostics {
            if diagnostic.message.contains("requires an allocator") {
                // Create a code action to add get_default_allocator()
                actions.push(quick_fixes::create_allocator_fix_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ));
            }

            // Add code action for string conversions
            if diagnostic.message.contains("type mismatch")
                && (diagnostic.message.contains("StaticString")
                    || diagnostic.message.contains("String"))
            {
                if let Some(action) = quick_fixes::create_string_conversion_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ) {
                    actions.push(action);
                }
            }

            if diagnostic.message.contains("Result") && diagnostic.message.contains("unwrap") {
                if let Some(action) = quick_fixes::create_error_handling_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ) {
                    actions.push(action);
                }
            }

            // Quick-fix for undefined/undeclared variable - suggest similar names
            if diagnostic.message.contains("undeclared")
                || diagnostic.message.contains("undefined")
                || diagnostic.message.contains("not found")
            {
                actions.extend(suggestions::create_did_you_mean_actions(
                    diagnostic,
                    &params.text_document.uri,
                    doc,
                    &store,
                ));
            }

            // Quick-fix for unused variable - prefix with underscore
            if diagnostic.message.contains("unused")
                && (diagnostic.message.contains("variable")
                    || diagnostic.message.contains("binding"))
            {
                if let Some(action) = quick_fixes::create_unused_variable_fix(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ) {
                    actions.push(action);
                }
            }

            // Quick-fix for missing import - add import statement
            if diagnostic.message.contains("not found") || diagnostic.message.contains("undefined")
            {
                if let Some(action) = imports::create_missing_import_fix(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                    &store,
                ) {
                    actions.push(action);
                }
            }
        }

        // Add refactoring code actions (not tied to diagnostics)
        // Extract variable - only if there's a selection
        if params.range.start != params.range.end {
            if let Some(action) = refactorings::create_extract_variable_action(
                &params.range,
                &params.text_document.uri,
                &doc.content,
            ) {
                actions.push(action);
            }

            // Extract function - only if there's a multi-line selection or complex expression
            if let Some(action) = refactorings::create_extract_function_action(
                &params.range,
                &params.text_document.uri,
                &doc.content,
            ) {
                actions.push(action);
            }
        }

        if let Some(action) =
            imports::create_add_import_action(&params.text_document.uri, &doc.content)
        {
            actions.push(action);
        }
    }

    success_response(&req, actions)
}
