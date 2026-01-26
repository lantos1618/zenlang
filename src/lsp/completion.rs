// Completion functionality for Zen Language Server
// Provides code completion with type-aware method suggestions

use lsp_server::{Request, Response};
use lsp_types::*;
use std::collections::{HashMap, HashSet};

// Import from other LSP modules
use super::document_store::DocumentStore;
use super::helpers::{char_pos_to_byte_pos, success_response, try_lock, try_parse_params};
use super::semantic_completion::{get_semantic_dot_completions, resolve_receiver_type};
use super::stdlib_resolver::StdlibResolver;
use super::type_inference::infer_receiver_type_with_context;
use super::types::{SymbolInfo, ZenCompletionContext};
use super::utils::{format_type, symbol_kind_to_completion_kind};
use crate::well_known::well_known;

// ============================================================================
// PUBLIC COMPLETION HANDLER
// ============================================================================

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

    // Check if we're completing after a dot (UFC method call)
    if let Some(doc) = store
        .documents
        .get(&params.text_document_position.text_document.uri)
    {
        let position = params.text_document_position.position;

        if let Some(context) = get_completion_context(&doc.content, position, store) {
            match context {
                ZenCompletionContext::UfcMethod { receiver_type } => {
                    // Try semantic completion first (using TypeContext)
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

                    // Fallback to heuristics-based completion
                    // Provide struct field completions first (most relevant)
                    let mut completions = get_struct_field_completions(&receiver_type, store);

                    // Only add UFC methods if we didn't find struct fields (to avoid stdlib clutter)
                    // Or if receiver_type is not a struct name (starts with uppercase)
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
                    // Provide module path completions (e.g., @std.io, @std.types)
                    let completions = get_module_path_completions(&base, store);
                    return success_response(&req, CompletionResponse::Array(completions));
                }
                ZenCompletionContext::General => {
                    // Fall through to provide general completions
                }
            }
        }
    }

    // Provide general completions
    let mut completions = vec![
        // Keywords
        CompletionItem {
            label: "main".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("main = () i32 { ... }".to_string()),
            documentation: Some(Documentation::String("Entry point function".to_string())),
            ..Default::default()
        },
        CompletionItem {
            label: "loop".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("loop() { ... }".to_string()),
            documentation: Some(Documentation::String(
                "Infinite loop with break statement".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("return value".to_string()),
            documentation: Some(Documentation::String("Return from function".to_string())),
            ..Default::default()
        },
        CompletionItem {
            label: "break".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("break".to_string()),
            documentation: Some(Documentation::String("Break from loop".to_string())),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("continue".to_string()),
            documentation: Some(Documentation::String(
                "Continue to next iteration".to_string(),
            )),
            ..Default::default()
        },
        // Common types
        {
            let wk = well_known();
            CompletionItem {
                label: wk.option_name().to_string(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some(format!("{}<T>", wk.option_name())),
                documentation: Some(Documentation::String("Optional value type".to_string())),
                ..Default::default()
            }
        },
        {
            let wk = well_known();
            CompletionItem {
                label: wk.result_name().to_string(),
                kind: Some(CompletionItemKind::ENUM),
                detail: Some(format!("{}<T, E>", wk.result_name())),
                documentation: Some(Documentation::String(
                    "Result type for error handling".to_string(),
                )),
                ..Default::default()
            }
        },
        {
            let wk = well_known();
            CompletionItem {
                label: wk.some_name().to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{}(value)", wk.some_name())),
                documentation: Some(Documentation::String(
                    "Option variant with value".to_string(),
                )),
                ..Default::default()
            }
        },
        {
            let wk = well_known();
            CompletionItem {
                label: wk.none_name().to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(wk.none_name().to_string()),
                documentation: Some(Documentation::String(
                    "Option variant without value".to_string(),
                )),
                ..Default::default()
            }
        },
        {
            let wk = well_known();
            CompletionItem {
                label: wk.ok_name().to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{}(value)", wk.ok_name())),
                documentation: Some(Documentation::String(
                    "Success variant of Result".to_string(),
                )),
                ..Default::default()
            }
        },
        {
            let wk = well_known();
            CompletionItem {
                label: wk.err_name().to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{}(error)", wk.err_name())),
                documentation: Some(Documentation::String("Error variant of Result".to_string())),
                ..Default::default()
            }
        },
        // Collections
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
        // Module imports
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
    ];

    // Add primitive types (including StaticString - always available)
    for ty in [
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
    ] {
        completions.push(CompletionItem {
            label: ty.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(format!("{} - Built-in primitive type", ty)),
            documentation: Some(Documentation::String(format!(
                "Built-in primitive type `{}`. Always available, no import needed.",
                ty
            ))),
            ..Default::default()
        });
    }

    // Add document symbols (functions, structs, enums defined in current file)
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
                ..Default::default()
            });
        }
    }

    // Add stdlib symbols with auto-import (functions and types from standard library)
    let doc_content = store
        .documents
        .get(&params.text_document_position.text_document.uri)
        .map(|d| d.content.as_str())
        .unwrap_or("");

    for (name, symbol) in &store.stdlib_symbols {
        // Try to get module path from definition URI for auto-import
        if let Some(ref uri) = symbol.definition_uri {
            if let Some(module_path) = get_module_path_from_uri(uri) {
                completions.push(create_completion_with_import(
                    name,
                    symbol,
                    &module_path,
                    doc_content,
                ));
                continue;
            }
        }

        // Fallback without auto-import
        completions.push(CompletionItem {
            label: name.clone(),
            kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
            detail: symbol.detail.clone(),
            documentation: None,
            ..Default::default()
        });
    }

    // Add workspace symbols (from other files in the project)
    // Limit to prevent overwhelming the user
    let mut workspace_count = 0;
    const MAX_WORKSPACE_COMPLETIONS: usize = 50;

    for (name, symbol) in &store.workspace_symbols {
        if workspace_count >= MAX_WORKSPACE_COMPLETIONS {
            break;
        }

        // Only add if not already in completions (avoid duplicates)
        if !completions.iter().any(|c| c.label == *name) {
            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(symbol_kind_to_completion_kind(symbol.kind)),
                detail: symbol.detail.clone(),
                documentation: None,
                ..Default::default()
            });
            workspace_count += 1;
        }
    }

    success_response(&req, CompletionResponse::Array(completions))
}

// ============================================================================
// CONTEXT DETECTION
// ============================================================================

fn get_completion_context(
    content: &str,
    position: Position,
    store: &DocumentStore,
) -> Option<ZenCompletionContext> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return Some(ZenCompletionContext::General);
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let byte_pos = char_pos_to_byte_pos(line, char_pos);

    // Check if we're completing after @std. (module path completion)
    if char_pos > 5 {
        let before_cursor = &line[..byte_pos];
        if before_cursor.ends_with("@std.") || before_cursor.contains("@std.") {
            // Check if we're right after @std.
            if let Some(std_pos) = before_cursor.rfind("@std.") {
                let after_std = &before_cursor[std_pos + 5..];
                // If there's no dot after @std., we're completing module names
                if !after_std.contains('.') {
                    return Some(ZenCompletionContext::ModulePath {
                        base: "@std".to_string(),
                    });
                }
            }
        }
    }

    // Check if we're after a dot
    if char_pos > 0 && line.chars().nth(char_pos - 1) == Some('.') {
        // Extract the receiver expression before the dot
        let chars: Vec<char> = line.chars().collect();
        let mut start = if char_pos > 1 {
            char_pos - 2
        } else {
            0
        };
        let mut paren_depth = 0;

        // Find the start of the receiver expression
        while start > 0 {
            match chars[start] {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    } else {
                        break;
                    }
                }
                ' ' | '\t' | '=' | '{' | '[' | ',' | ';' if paren_depth == 0 => {
                    start += 1;
                    break;
                }
                _ => {}
            }
            start -= 1;
        }

        let receiver: String = chars[start..(char_pos - 1)].iter().collect();
        let receiver = receiver.trim();

        let receiver_type =
            infer_receiver_type_with_context(receiver, Some(content), Some(position), store)
                .unwrap_or_else(|| "unknown".to_string());

        return Some(ZenCompletionContext::UfcMethod { receiver_type });
    }

    Some(ZenCompletionContext::General)
}

// ============================================================================
// TYPE INFERENCE
// ============================================================================

pub fn infer_variable_type(
    content: &str,
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
) -> Option<String> {
    // Look for variable assignment: var_name = function_call() or var_name: Type = ...
    let lines: Vec<&str> = content.lines().collect();

    // Limit to first 1000 lines to prevent performance issues
    let max_lines = lines.len().min(1000);
    for line in lines.iter().take(max_lines) {
        // Pattern: var_name = function_call()
        if line.contains(&format!("{} =", var_name)) || line.contains(&format!("{}=", var_name)) {
            // Check for type annotation first: var_name: Type = ...
            if let Some(colon_pos) = line.find(':') {
                if let Some(eq_pos) = line.find('=') {
                    if colon_pos < eq_pos {
                        // Extract type between : and =
                        let type_str = line[colon_pos + 1..eq_pos].trim();
                        if !type_str.is_empty() {
                            return Some(format!(
                                "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                                var_name, type_str, type_str
                            ));
                        }
                    }
                }
            }

            // Try to find function call and infer return type
            if let Some(eq_pos) = line.find('=') {
                let rhs = &line[eq_pos + 1..].trim();

                // Check if it's a function call
                if let Some(paren_pos) = rhs.find('(') {
                    let func_name = rhs[..paren_pos].trim();

                    // Look up function in symbols
                    if let Some(func_info) = local_symbols
                        .get(func_name)
                        .or_else(|| stdlib_symbols.get(func_name))
                        .or_else(|| workspace_symbols.get(func_name))
                    {
                        if let Some(type_info) = &func_info.type_info {
                            let type_str = format_type(type_info);
                            return Some(format!(
                                "```zen\n{} = {}()\n```\n\n**Type:** `{}`\n\n**Inferred from:** `{}` function return type",
                                var_name, func_name, type_str, func_name
                            ));
                        }

                        // Try to parse from detail string
                        if let Some(detail) = &func_info.detail {
                            // Parse "func_name = (args) return_type"
                            if let Some(arrow_or_paren_close) = detail.rfind(')') {
                                let return_part = detail[arrow_or_paren_close + 1..].trim();
                                if !return_part.is_empty() && return_part != "void" {
                                    return Some(format!(
                                        "```zen\n{} = {}()\n```\n\n**Type:** `{}`\n\n**Inferred from:** `{}` function return type",
                                        var_name, func_name, return_part, func_name
                                    ));
                                }
                            }
                        }
                    }
                }

                // Check for constructor calls (Type { ... } or Type(...))
                if let Some(brace_pos) = rhs.find('{') {
                    let type_name = rhs[..brace_pos].trim();
                    if type_name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        return Some(format!(
                            "```zen\n{} = {} {{ ... }}\n```\n\n**Type:** `{}`\n\n**Inferred from:** constructor",
                            var_name, type_name, type_name
                        ));
                    }
                }

                // Check for literals
                let trimmed = rhs.trim();
                if trimmed.starts_with('"') || trimmed.starts_with('\'') {
                    return Some(format!(
                        "```zen\n{} = {}\n```\n\n**Type:** `StaticString`",
                        var_name, trimmed
                    ));
                }
                if trimmed.parse::<i32>().is_ok() {
                    return Some(format!(
                        "```zen\n{} = {}\n```\n\n**Type:** `i32`",
                        var_name, trimmed
                    ));
                }
                if trimmed.parse::<f64>().is_ok() && trimmed.contains('.') {
                    return Some(format!(
                        "```zen\n{} = {}\n```\n\n**Type:** `f64`",
                        var_name, trimmed
                    ));
                }
                if trimmed == "true" || trimmed == "false" {
                    return Some(format!(
                        "```zen\n{} = {}\n```\n\n**Type:** `bool`",
                        var_name, trimmed
                    ));
                }
            }
        }
    }

    None
}

pub fn find_stdlib_location(
    stdlib_path: &str,
    method_name: &str,
    store: &DocumentStore,
) -> Option<Location> {
    // Try to find the method in the stdlib file
    // First check if we have the stdlib file open
    for (uri, doc) in &store.documents {
        if uri.path().contains(stdlib_path) {
            // Look for the method in this file's symbols
            if let Some(symbol) = doc.symbols.get(method_name) {
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol.range,
                });
            }
        }
    }

    // If not found in open documents, we could potentially open and parse the stdlib file
    // For now, return None to indicate it's a built-in method
    None
}

// ============================================================================
// MODULE PATH COMPLETIONS
// ============================================================================

fn get_module_path_completions(base: &str, store: &DocumentStore) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    if base == "@std" {
        // Get actual modules from stdlib resolver (dynamic based on file structure)
        let available_modules = store.stdlib_resolver.list_modules();

        for module_name in available_modules {
            completions.push(CompletionItem {
                label: format!("{}.{}", base, module_name),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(format!("Import from {} module", module_name)),
                documentation: Some(Documentation::String(format!(
                    "Standard library module: {}",
                    module_name
                ))),
                ..Default::default()
            });
        }
    } else if base.starts_with("@std.") {
        // Handle nested paths like @std.collections -> show hashmap, list, etc.
        let submodule = base.strip_prefix("@std.").unwrap_or("");
        let _submodule_path = format!("@std.{}", submodule);

        // Try to resolve the submodule to see what's inside
        // Note: resolve_module_path returns the file path, but we need the directory
        // For now, construct the directory path manually
        let submodule_dir = store
            .stdlib_resolver
            .stdlib_root
            .join(submodule.replace('.', "/"));
        if submodule_dir.exists() && submodule_dir.is_dir() {
            let dir_path = &submodule_dir;
            if dir_path.is_dir() {
                // Scan directory for available modules/files
                let mut submodules = Vec::new();
                StdlibResolver::scan_directory(dir_path, &mut submodules, submodule);

                for submod in submodules {
                    completions.push(CompletionItem {
                        label: format!("{}.{}", base, submod),
                        kind: Some(CompletionItemKind::MODULE),
                        detail: Some(format!("Nested module: {}", submod)),
                        documentation: None,
                        ..Default::default()
                    });
                }
            }
        }
    }

    completions
}

// ============================================================================
// STRUCT FIELD COMPLETIONS
// ============================================================================

fn get_struct_field_completions(receiver_type: &str, store: &DocumentStore) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Extract struct name from type string (handle cases like "Person", "Person<...>", etc.)
    let struct_name = receiver_type
        .split('<')
        .next()
        .unwrap_or(receiver_type)
        .trim();

    // Find struct definition in documents
    use super::hover::find_struct_definition_in_documents;
    if let Some(struct_def) = find_struct_definition_in_documents(struct_name, &store.documents) {
        // Add field completions
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

// ============================================================================
// UFC METHOD COMPLETIONS
// ============================================================================

fn get_ufc_method_completions(receiver_type: &str, store: &DocumentStore) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Query compiler integration for actual methods
    for sig in store.compiler.get_methods_for_type(receiver_type) {
        items.push(CompletionItem {
            label: sig.name.clone(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(format!(
                "({}) -> {}",
                sig.params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, super::utils::format_type(t)))
                    .collect::<Vec<_>>()
                    .join(", "),
                super::utils::format_type(&sig.return_type)
            )),
            ..Default::default()
        });
    }

    items
}

// ============================================================================
// AUTO-IMPORT HELPERS
// ============================================================================

/// Extract the module path from a definition URI (e.g., @std.collections.vec)
fn get_module_path_from_uri(uri: &Url) -> Option<String> {
    let path = uri.path();

    // Check if it's a stdlib file
    if let Some(stdlib_pos) = path.find("/stdlib/") {
        let relative = &path[stdlib_pos + 8..]; // Skip "/stdlib/"
        let module_path = relative.trim_end_matches(".zen").replace('/', ".");

        // Handle module names that match directory (e.g., collections/vec.zen -> @std.collections.vec)
        return Some(format!("@std.{}", module_path));
    }

    None
}

/// Get symbols that are already imported in the document
fn get_imported_symbols(content: &str) -> HashSet<String> {
    let mut imported = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Match: { symbol1, symbol2 } = @std.module
        if trimmed.starts_with('{') && trimmed.contains("} =") {
            if let Some(brace_end) = trimmed.find('}') {
                let symbols_str = &trimmed[1..brace_end];
                for symbol in symbols_str.split(',') {
                    imported.insert(symbol.trim().to_string());
                }
            }
        }
    }

    imported
}

/// Find the best position to insert an import statement
fn find_import_insert_position(content: &str) -> Position {
    let mut last_import_line = 0;
    let mut found_import = false;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments at the top
        if trimmed.starts_with("//") {
            if !found_import {
                last_import_line = line_num + 1;
            }
            continue;
        }

        // Track import lines
        if trimmed.starts_with('{') && trimmed.contains("} =") && trimmed.contains('@') {
            last_import_line = line_num + 1;
            found_import = true;
        } else if !trimmed.is_empty() && found_import {
            // Non-empty, non-import line after imports - stop here
            break;
        } else if !trimmed.is_empty() && !found_import {
            // Non-empty, non-import line before any imports - insert at top
            break;
        }
    }

    Position {
        line: last_import_line as u32,
        character: 0,
    }
}

/// Create a TextEdit for inserting an import statement
fn create_import_edit(symbol_name: &str, module_path: &str, content: &str) -> Option<TextEdit> {
    // Check if already imported
    let imported = get_imported_symbols(content);
    if imported.contains(symbol_name) {
        return None;
    }

    let insert_pos = find_import_insert_position(content);
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
fn create_completion_with_import(
    name: &str,
    symbol: &SymbolInfo,
    module_path: &str,
    content: &str,
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

    // Add the import as an additional text edit
    if let Some(edit) = create_import_edit(name, module_path, content) {
        item.additional_text_edits = Some(vec![edit]);
        item.label_details = Some(CompletionItemLabelDetails {
            detail: Some(format!(" ({})", module_path)),
            description: None,
        });
    }

    item
}
