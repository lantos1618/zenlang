//! Code completion for Zen Language Server
//!
//! Provides intelligent code completion with:
//! - Context-aware completions (struct literals, method calls, module paths)
//! - Auto-import functionality
//! - UFC method suggestions
//! - Struct field completions

pub mod auto_import;
pub mod context;
mod methods;
mod modules;

// Re-export submodule functions
pub use auto_import::{create_completion_with_import, get_module_path_from_uri};
pub use context::{
    get_completion_context, get_pattern_match_completions, get_struct_literal_completions,
};
pub use methods::{get_struct_field_completions, get_ufc_method_completions};
pub use modules::get_module_path_completions;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{success_response, try_lock, try_parse_params};
use crate::lsp::semantic_completion::{get_semantic_dot_completions, resolve_receiver_type};
use crate::lsp::types::ZenCompletionContext;
use crate::lsp::utils::symbol_kind_to_completion_kind;
use crate::well_known::well_known;
use lsp_server::{Request, Response};
use lsp_types::*;

/// Main completion request handler
pub fn handle_completion(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    let store_guard = match try_lock(store.as_ref(), &req) {
        Ok(guard) => guard,
        Err(_) => return success_response(&req, CompletionResponse::Array(Vec::new())),
    };
    let store = &*store_guard;

    let params: CompletionParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    // Check context-specific completions
    if let Some(doc) = store
        .documents
        .get(&params.text_document_position.text_document.uri)
    {
        let position = params.text_document_position.position;

        if let Some(context) = get_completion_context(&doc.content, position, Some(doc)) {
            match context {
                ZenCompletionContext::UfcMethod { receiver_type } => {
                    // Try semantic completion first
                    if let Some(ast_type) = resolve_receiver_type(&receiver_type, doc, position) {
                        if let Some(semantic_completions) =
                            get_semantic_dot_completions(doc, &ast_type, store)
                        {
                            if !semantic_completions.is_empty() {
                                return success_response(
                                    &req,
                                    CompletionResponse::Array(semantic_completions),
                                );
                            }
                        }
                    }

                    // Fallback to heuristics
                    let mut completions = get_struct_field_completions(&receiver_type, store);
                    if completions.is_empty()
                        || !receiver_type
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                    {
                        completions.extend(get_ufc_method_completions(&receiver_type, store));
                    }

                    return success_response(&req, CompletionResponse::Array(completions));
                }
                ZenCompletionContext::ModulePath { base } => {
                    let completions = get_module_path_completions(&base, store);
                    return success_response(&req, CompletionResponse::Array(completions));
                }
                ZenCompletionContext::StructLiteral { struct_name } => {
                    let completions =
                        get_struct_literal_completions(&struct_name, doc, &doc.content, position);
                    return success_response(&req, CompletionResponse::Array(completions));
                }
                ZenCompletionContext::PatternMatch { matched_type } => {
                    let completions = get_pattern_match_completions(&matched_type, doc);
                    return success_response(&req, CompletionResponse::Array(completions));
                }
                ZenCompletionContext::General => {
                    // Fall through to general completions
                }
            }
        }
    }

    // Provide general completions
    let completions = build_general_completions(store, &params);
    success_response(&req, CompletionResponse::Array(completions))
}

/// Completion priority tiers (lower = higher priority)
mod priority {
    pub const KEYWORD: &str = "0"; // Keywords and built-in constructs
    pub const LOCAL_SYMBOL: &str = "1"; // Symbols in current document
    pub const PRIMITIVE: &str = "2"; // Primitive types
    pub const WORKSPACE: &str = "3"; // Symbols from other project files
    pub const STDLIB: &str = "4"; // Standard library (requires import)
}

/// Build general completions (keywords, types, symbols)
fn build_general_completions(
    store: &DocumentStore,
    params: &CompletionParams,
) -> Vec<CompletionItem> {
    let mut completions = build_keyword_completions();

    // Add primitive types (priority 2)
    for ty in PRIMITIVE_TYPES {
        completions.push(CompletionItem {
            label: ty.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(format!("{} - Built-in primitive type", ty)),
            documentation: Some(Documentation::String(format!(
                "Built-in primitive type `{}`. Always available, no import needed.",
                ty
            ))),
            sort_text: Some(format!("{}{}", priority::PRIMITIVE, ty)),
            ..Default::default()
        });
    }

    // Add document symbols (priority 1 - highest for user-defined)
    if let Some(doc) = store
        .documents
        .get(&params.text_document_position.text_document.uri)
    {
        for (name, symbol) in &doc.symbols {
            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
                detail: symbol.detail.clone(),
                documentation: None,
                sort_text: Some(format!("{}{}", priority::LOCAL_SYMBOL, name)),
                ..Default::default()
            });
        }
    }

    // Add stdlib symbols with auto-import (priority 4)
    let doc_content = store
        .documents
        .get(&params.text_document_position.text_document.uri)
        .map(|d| d.content.as_str())
        .unwrap_or("");

    for (name, symbol) in &store.stdlib_symbols {
        if let Some(ref uri) = symbol.definition_uri {
            if let Some(module_path) = get_module_path_from_uri(uri) {
                let mut item =
                    create_completion_with_import(name, symbol, &module_path, doc_content);
                item.sort_text = Some(format!("{}{}", priority::STDLIB, name));
                completions.push(item);
                continue;
            }
        }

        completions.push(CompletionItem {
            label: name.clone(),
            kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
            detail: symbol.detail.clone(),
            documentation: None,
            sort_text: Some(format!("{}{}", priority::STDLIB, name)),
            ..Default::default()
        });
    }

    // Add workspace symbols (priority 3)
    add_workspace_symbols(&mut completions, store);

    completions
}

/// Build keyword and common type completions
fn build_keyword_completions() -> Vec<CompletionItem> {
    let wk = well_known();
    vec![
        CompletionItem {
            label: "main".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("main = () i32 { ... }".to_string()),
            documentation: Some(Documentation::String("Entry point function".to_string())),
            sort_text: Some(format!("{}main", priority::KEYWORD)),
            ..Default::default()
        },
        CompletionItem {
            label: "loop".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("loop() { ... }".to_string()),
            documentation: Some(Documentation::String(
                "Infinite loop with break statement".to_string(),
            )),
            sort_text: Some(format!("{}loop", priority::KEYWORD)),
            ..Default::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("return value".to_string()),
            documentation: Some(Documentation::String("Return from function".to_string())),
            sort_text: Some(format!("{}return", priority::KEYWORD)),
            ..Default::default()
        },
        CompletionItem {
            label: "break".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("break".to_string()),
            documentation: Some(Documentation::String("Break from loop".to_string())),
            sort_text: Some(format!("{}break", priority::KEYWORD)),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("continue".to_string()),
            documentation: Some(Documentation::String(
                "Continue to next iteration".to_string(),
            )),
            sort_text: Some(format!("{}continue", priority::KEYWORD)),
            ..Default::default()
        },
        CompletionItem {
            label: wk.option_name().to_string(),
            kind: Some(CompletionItemKind::ENUM),
            detail: Some(format!("{}<T>", wk.option_name())),
            documentation: Some(Documentation::String("Optional value type".to_string())),
            ..Default::default()
        },
        CompletionItem {
            label: wk.result_name().to_string(),
            kind: Some(CompletionItemKind::ENUM),
            detail: Some(format!("{}<T, E>", wk.result_name())),
            documentation: Some(Documentation::String(
                "Result type for error handling".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: wk.some_name().to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(format!("{}(value)", wk.some_name())),
            documentation: Some(Documentation::String(
                "Option variant with value".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: wk.none_name().to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(wk.none_name().to_string()),
            documentation: Some(Documentation::String(
                "Option variant without value".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: wk.ok_name().to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(format!("{}(value)", wk.ok_name())),
            documentation: Some(Documentation::String(
                "Success variant of Result".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: wk.err_name().to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(format!("{}(error)", wk.err_name())),
            documentation: Some(Documentation::String("Error variant of Result".to_string())),
            ..Default::default()
        },
        CompletionItem {
            label: "Vec".to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some("Vec<T, size>".to_string()),
            documentation: Some(Documentation::String(
                "Fixed-size vector (stack allocated)".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "DynVec".to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some("DynVec<T>".to_string()),
            documentation: Some(Documentation::String(
                "Dynamic vector (requires allocator)".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "HashMap".to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some("HashMap<K, V>".to_string()),
            documentation: Some(Documentation::String(
                "Hash map (requires allocator)".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "@std".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("Standard library".to_string()),
            documentation: Some(Documentation::String(
                "Import standard library modules".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "@this".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Current module reference".to_string()),
            documentation: Some(Documentation::String(
                "Reference to current module".to_string(),
            )),
            ..Default::default()
        },
    ]
}

/// Add workspace symbols to completions (limited to avoid overwhelming)
fn add_workspace_symbols(completions: &mut Vec<CompletionItem>, store: &DocumentStore) {
    const MAX_WORKSPACE_COMPLETIONS: usize = 50;
    let mut count = 0;

    for (name, symbol) in &store.workspace_symbols {
        if count >= MAX_WORKSPACE_COMPLETIONS {
            break;
        }

        if !completions.iter().any(|c| c.label == *name) {
            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
                detail: symbol.detail.clone(),
                documentation: None,
                sort_text: Some(format!("{}{}", priority::WORKSPACE, name)),
                ..Default::default()
            });
            count += 1;
        }
    }
}

const PRIMITIVE_TYPES: &[&str] = &[
    "i8",
    "i16",
    "i32",
    "i64",
    "u8",
    "u16",
    "u32",
    "u64",
    "usize",
    "f32",
    "f64",
    "bool",
    "StaticString",
    "void",
];

// Re-export infer_variable_type from hover module (where it's now defined)
pub use crate::lsp::hover::infer_variable_type;

pub fn find_stdlib_location(
    stdlib_path: &str,
    method_name: &str,
    store: &DocumentStore,
) -> Option<Location> {
    crate::lsp::navigation::utils::find_stdlib_location(stdlib_path, method_name, None, store)
}
