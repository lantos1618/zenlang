// Hover response creation utilities

use lsp_server::{RequestId, Response};
use lsp_types::*;

use crate::ast::AstType;
use crate::intrinsics::{well_known, WellKnownType};
use crate::lsp::types::SymbolInfo;
use crate::lsp::utils::format_type;
use crate::name_utils;

/// Create a hover response from symbol info
pub fn create_hover_response(
    id: RequestId,
    symbol_info: &SymbolInfo,
    range: Option<Range>,
) -> Response {
    let mut hover_content = Vec::with_capacity(6);

    // Show signature with syntax highlighting
    if let Some(detail) = &symbol_info.detail {
        hover_content.push(format!("```zen\n{}\n```", detail));

        // For functions, extract and display parameter info
        if symbol_info.kind == SymbolKind::FUNCTION || symbol_info.kind == SymbolKind::METHOD {
            if let Some(params_doc) =
                extract_parameter_docs_from_params(symbol_info.params.as_deref())
            {
                hover_content.push(params_doc);
            }
        }
    }

    // Show documentation if available
    if let Some(doc) = &symbol_info.documentation {
        hover_content.push(doc.clone());
    }

    // Show type information
    if let Some(type_info) = &symbol_info.type_info {
        let type_str = format_type(type_info);
        // Only show type if it's different from what's in detail
        if symbol_info
            .detail
            .as_ref()
            .is_none_or(|d| !d.contains(&type_str))
        {
            hover_content.push(format!("**Returns:** `{}`", type_str));
        }
    }

    // Add location information
    if let Some(def_uri) = &symbol_info.definition_uri {
        if let Ok(path) = def_uri.to_file_path() {
            hover_content.push(format!(
                "*Defined in* `{}:{}`",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown"),
                symbol_info.range.start.line + 1
            ));
        } else {
            hover_content.push("*Source:* Standard Library".to_string());
        }
    } else {
        hover_content.push("*Source:* Standard Library".to_string());
    }

    let contents = HoverContents::Markup(MarkupContent {
        kind: MarkupKind::Markdown,
        value: hover_content.join("\n\n"),
    });

    crate::lsp::helpers::success_response_id(id, Hover { contents, range })
}

/// Extract parameter documentation from structured parameter data
fn extract_parameter_docs_from_params(params: Option<&[(String, AstType)]>) -> Option<String> {
    let params = params?;
    if params.is_empty() {
        return None;
    }

    let mut doc = String::from("**Parameters:**\n");
    for (name, ty) in params {
        let type_str = format_type(ty);
        let hint = get_type_hint_from_ast(ty, &type_str);
        if let Some(h) = hint {
            doc.push_str(&format!("- `{}`: `{}` — {}\n", name, type_str, h));
        } else {
            doc.push_str(&format!("- `{}`: `{}`\n", name, type_str));
        }
    }

    Some(doc)
}

/// Get contextual hint for common parameter types using the compiler's well-known types
/// and the structured AstType directly.
fn get_type_hint_from_ast(ast_type: &AstType, type_str: &str) -> Option<&'static str> {
    let wk = well_known();

    // Extract the base type name using name_utils instead of manual .find('<')
    let base_type = name_utils::strip_generics(type_str);

    // Check well-known types from compiler registry
    if let Some(wk_type) = wk.get_type(base_type) {
        return Some(match wk_type {
            WellKnownType::Option => "May be `None`, handle with pattern match or `.unwrap()`",
            WellKnownType::Result => "May return an error, handle with `.raise()` or pattern match",
            WellKnownType::Ptr => "Immutable pointer — use `.val` to read, must be freed",
            WellKnownType::MutPtr => "Mutable pointer — use `.val` to read/write, must be freed",
            WellKnownType::RawPtr => "Raw pointer — low-level, for FFI and unsafe operations",
        });
    }

    // Check for allocator types using AstType structure
    match ast_type {
        AstType::Struct { name, .. } if name.to_lowercase().contains("allocator") => {
            return Some("Use `get_default_allocator()` or pass a specific allocator");
        }
        AstType::Generic { name, .. } if name.to_lowercase().contains("allocator") => {
            return Some("Use `get_default_allocator()` or pass a specific allocator");
        }
        _ => {}
    }

    // Additional hints for common stdlib types
    if base_type == "StaticString" {
        return Some("String literal or `.as_static()` result");
    }
    if base_type == "String" {
        return Some("Heap-allocated string");
    }
    if base_type == "usize" {
        return Some("Unsigned size type (array indices, lengths)");
    }

    None
}
