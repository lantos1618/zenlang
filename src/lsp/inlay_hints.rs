use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, RwLock};

use crate::ast::looks_like_type_name;

use super::document_store::DocumentStore;
use super::utils::format_type;
use crate::type_context::TypeContext;

pub fn handle_inlay_hints(req: Request, store: &Arc<RwLock<DocumentStore>>) -> Response {
    let params: InlayHintParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => return empty_response(req.id),
    };

    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return empty_response(req.id),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(d) => d,
        None => return empty_response(req.id),
    };

    let mut hints = Vec::new();
    let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();

    // If we have TypeContext from the typechecker, use it for authoritative hints
    if let Some(type_ctx) = &doc.type_context {
        collect_hints_from_type_context(&doc.content, type_ctx, &mut hints, &mut seen);
    }
    // When TypeContext is not available, we don't show type hints rather than
    // guessing with fragile string-based heuristics. The TypeContext should be
    // populated by background analysis for any actively edited file.

    crate::lsp::helpers::success_response_id(req.id, hints)
}

fn empty_response(id: lsp_server::RequestId) -> Response {
    crate::lsp::helpers::success_response_id(id, Vec::<InlayHint>::new())
}

/// Collect inlay hints using authoritative type data from the TypeContext.
/// This uses the real typechecker output instead of heuristics.
fn collect_hints_from_type_context(
    content: &str,
    type_ctx: &TypeContext,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    // Iterate through all variables in the TypeContext
    // Keys are "function_name::var_name", values are AstType
    for (scoped_key, var_type) in &type_ctx.variables {
        // Parse "function_name::var_name"
        let var_name = match scoped_key.rsplit_once("::") {
            Some((_, name)) => name,
            None => continue,
        };

        // Find the variable declaration in source to get position
        if let Some((line_num, hint_pos, is_mutable, is_colon_eq)) =
            find_var_decl_position(content, var_name)
        {
            let key = (line_num, hint_pos);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            let type_str = format_type(var_type);
            let line = content.lines().nth(line_num as usize).unwrap_or("");

            let (hint_label, hint_char_pos) = if is_mutable {
                if let Some(dce_pos) = line.find("::=") {
                    (format!(" {} ", type_str), (dce_pos + 2) as u32)
                } else {
                    continue;
                }
            } else if is_colon_eq {
                if let Some(colon_eq_pos) = line.find(":=") {
                    (format!(" {} ", type_str), (colon_eq_pos + 1) as u32)
                } else {
                    continue;
                }
            } else {
                (format!(": {} ", type_str), hint_pos)
            };

            hints.push(InlayHint {
                position: Position {
                    line: line_num,
                    character: hint_char_pos,
                },
                label: InlayHintLabel::String(hint_label),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip: None,
                padding_left: None,
                padding_right: None,
                data: None,
            });
        }
    }
}

/// Find a variable declaration position in source content.
/// Returns (line_num, char_after_varname, is_mutable, is_colon_eq) or None.
fn find_var_decl_position(content: &str, var_name: &str) -> Option<(u32, u32, bool, bool)> {
    let mut in_struct_def = false;
    let mut brace_depth: i32 = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track struct/enum definitions to skip them
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            if let Some(colon_pos) = trimmed.find(':') {
                let before = trimmed[..colon_pos].trim();
                let after = trimmed[colon_pos + 1..].trim();
                if !before.is_empty()
                    && looks_like_type_name(before)
                    && !before.contains('=')
                    && !before.contains(' ')
                    && (after.starts_with('{') || after.contains('{'))
                {
                    in_struct_def = true;
                    brace_depth = 0;
                }
            }
            if in_struct_def {
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            in_struct_def = false;
                        }
                    }
                }
            }
        }

        if in_struct_def {
            continue;
        }

        // Look for variable declaration
        if let Some(pos) = line.find(var_name) {
            let before_ok = pos == 0
                || line
                    .as_bytes()
                    .get(pos - 1)
                    .map(|&b| b.is_ascii_whitespace())
                    .unwrap_or(true);
            let after = &line[pos + var_name.len()..].trim_start();

            if !before_ok {
                continue;
            }

            // Detect declaration type
            let has_explicit_type = (after.starts_with("::") && !after.starts_with("::="))
                || (after.starts_with(':') && !after.starts_with(":="));

            if has_explicit_type {
                continue; // Already has type annotation
            }

            let is_mutable = after.starts_with("::=");
            let is_colon_eq = after.starts_with(":=");
            let is_assign = after.starts_with('=') || is_mutable || is_colon_eq;

            if is_assign {
                // Don't show hints for function definitions
                if after.contains("= (") || after.contains("= @") {
                    continue;
                }
                // Skip PascalCase (type/struct names)
                if var_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    continue;
                }
                return Some((
                    line_num as u32,
                    (pos + var_name.len()) as u32,
                    is_mutable,
                    is_colon_eq,
                ));
            }
        }
    }
    None
}
