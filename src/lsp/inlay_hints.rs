use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, Mutex};

use crate::ast::primitives;
use crate::ast::{looks_like_type_name, Declaration, Statement};

use super::document_store::DocumentStore;
use super::types::Document;
use super::utils::format_type;
use crate::name_utils;
use crate::type_context::TypeContext;

pub fn handle_inlay_hints(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: InlayHintParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => return empty_response(req.id),
    };

    let store = match store.lock() {
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
    } else {
        // Fallback: use heuristic-based inference
        collect_hints_from_content(&doc.content, &mut hints, &mut seen);

        if let Some(ast) = &doc.ast {
            for decl in ast {
                if let Declaration::Function(func) = decl {
                    collect_hints_from_statements(
                        &func.body,
                        &doc.content,
                        doc,
                        &mut hints,
                        &mut seen,
                    );
                }
            }
        }
    }

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

fn collect_hints_from_content(
    content: &str,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    let mut in_struct_def = false;
    let mut brace_depth: i32 = 0;
    let mut pending_struct_def = false; // True when we saw "Name:" and expect "{" next

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track whether this line should be skipped (starts or is inside struct/enum def)
        let mut skip_this_line = in_struct_def;

        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Check for pending struct def (previous line was "Name:", now we see "{")
            if pending_struct_def && trimmed.starts_with('{') {
                in_struct_def = true;
                brace_depth = 0;
                skip_this_line = true;
                pending_struct_def = false;
            }

            // Check for struct definition start: "Name: {" or "Name:" (with brace on next line)
            if let Some(colon_pos) = trimmed.find(':') {
                let before = trimmed[..colon_pos].trim();
                let after = trimmed[colon_pos + 1..].trim();

                // If it looks like a struct definition (PascalCase: { ... } or PascalCase:)
                if !before.is_empty()
                    && looks_like_type_name(before)
                    && !before.contains('=')
                    && !before.contains(' ')
                {
                    if after.starts_with('{') || after.contains('{') {
                        // Brace on same line: "Person: {"
                        in_struct_def = true;
                        brace_depth = 0;
                        skip_this_line = true;
                        pending_struct_def = false;
                    } else if after.is_empty() {
                        // Brace might be on next line: "Person:"
                        pending_struct_def = true;
                        skip_this_line = true; // Skip the definition line too
                    }
                }
            } else {
                // Line doesn't have a colon - clear pending if it's not just a brace
                if !trimmed.starts_with('{') {
                    pending_struct_def = false;
                }
            }

            // Track brace depth for struct definitions
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

        // Skip lines inside struct/enum definitions
        if skip_this_line {
            continue;
        }

        if let Some(hint) = try_create_hint_for_line(line, line_num as u32, seen) {
            hints.push(hint);
        }
    }
}

fn try_create_hint_for_line(
    line: &str,
    line_num: u32,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) -> Option<InlayHint> {
    let trimmed = line.trim();

    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with('{')
        || trimmed.starts_with('}')
        || trimmed.starts_with('|')
        || trimmed.contains("= (")
        || trimmed.contains("= @")
    {
        return None;
    }

    let (_var_name, hint_pos, has_explicit_type, is_mutable, is_colon_eq, rhs) =
        parse_var_decl(line)?;

    if has_explicit_type {
        return None;
    }

    let key = (line_num, hint_pos);
    if seen.contains(&key) {
        return None;
    }

    let inferred = infer_type(&rhs)?;
    seen.insert(key);

    // Format hint based on declaration type:
    // - Mutable (::=): show ":: Type" after variable name → "name :: Type = value"
    // - Colon-eq (:=): show " Type" after the colon → "name: Type = value"
    // - Immutable (=): show ": Type" after variable name → "name: Type = value"
    let (hint_label, hint_char_pos) = if is_mutable {
        // ::= → name :: Type = value
        // Position hint between :: and = in the ::= operator
        let dce_pos = line.find("::=")?;
        (format!(" {} ", inferred), (dce_pos + 2) as u32)
    } else if is_colon_eq {
        // := → name: Type = value
        // Position hint right after the colon (before the =)
        // Find the := in the line and position after the :
        let colon_eq_pos = line.find(":=")?;
        (format!(" {} ", inferred), (colon_eq_pos + 1) as u32)
    } else {
        // = → name: Type = value (with space before =)
        (format!(": {} ", inferred), hint_pos)
    };

    Some(InlayHint {
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
    })
}

/// Parse a variable declaration line.
/// Returns: (var_name, hint_position, has_explicit_type, is_mutable, is_colon_eq, rhs)
fn parse_var_decl(line: &str) -> Option<(String, u32, bool, bool, bool, String)> {
    let trimmed = line.trim();

    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }

    // Detect declaration type and operator:
    // - ::= means mutable with inferred type
    // - := means immutable with inferred type (shorthand)
    // - = means immutable with inferred type
    // is_mutable: true for ::=, false for := and =
    // is_colon_eq: true for :=, false for others
    let (eq_pos, eq_char_count, is_mutable, is_colon_eq) = if let Some(pos) = trimmed.find(" ::= ")
    {
        (pos, 5, true, false)
    } else if let Some(pos) = trimmed.find("::=") {
        (pos, 3, true, false)
    } else if let Some(pos) = trimmed.find(" := ") {
        (pos, 4, false, true)
    } else if let Some(pos) = trimmed.find(":=") {
        (pos, 2, false, true)
    } else if let Some(pos) = trimmed.find(" = ") {
        (pos, 3, false, false)
    } else {
        return None;
    };

    let before_eq = &trimmed[..eq_pos];
    let after_eq = &trimmed[eq_pos + eq_char_count..];

    // Check for explicit type annotation
    let has_explicit_type = if before_eq.contains("::") {
        // Mutable with explicit type: "name :: Type"
        let parts: Vec<&str> = before_eq.splitn(2, "::").collect();
        parts.len() == 2 && !parts[1].trim().is_empty()
    } else if before_eq.contains(':') {
        // Immutable with explicit type: "name: Type"
        let colon_pos = before_eq.find(':')?;
        let after_colon = before_eq[colon_pos + 1..].trim();
        !after_colon.is_empty()
    } else {
        false
    };

    // Extract variable name
    let var_name = if before_eq.contains("::") {
        before_eq.split("::").next()?.trim()
    } else if before_eq.contains(':') {
        before_eq.split(':').next()?.trim()
    } else {
        before_eq.trim()
    };

    if var_name.is_empty()
        || var_name.contains('(')
        || var_name.contains(')')
        || var_name.contains('.')
        || var_name.contains(' ')
        || var_name.chars().next()?.is_uppercase()
    {
        return None;
    }

    let var_start = line.find(var_name)?;
    let var_end_pos = (var_start + var_name.len()) as u32;

    Some((
        var_name.to_string(),
        var_end_pos,
        has_explicit_type,
        is_mutable,
        is_colon_eq,
        after_eq.trim().to_string(),
    ))
}

fn infer_type(rhs: &str) -> Option<String> {
    use crate::lexer::Lexer;
    use crate::lsp::type_query::TypeQuery;
    use crate::parser::Parser;
    use crate::well_known::well_known;

    let rhs = rhs.trim();

    let lexer = Lexer::new(rhs);
    let mut parser = Parser::new(lexer);
    if let Ok(expr) = parser.parse_expression() {
        if let Some(type_name) = TypeQuery::infer_literal_type(&expr) {
            return Some(type_name);
        }
    }

    if rhs.starts_with('"') || rhs.starts_with('\'') {
        return Some("StaticString".to_string());
    }

    if primitives::is_boolean_literal(rhs) {
        return Some("bool".to_string());
    }

    let wk = well_known();
    if rhs == wk.none_name() {
        return Some(wk.option_name().to_string());
    }

    None
}

fn collect_hints_from_statements(
    statements: &[Statement],
    content: &str,
    doc: &Document,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    for stmt in statements {
        match stmt {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                ..
            } => {
                if type_.is_none() {
                    if let Some(init) = initializer {
                        let tq = crate::lsp::type_query::TypeQuery::new(doc);
                        let inferred = tq.find_variable_type(name).or_else(|| {
                            crate::lsp::type_query::TypeQuery::infer_literal_type(init)
                        });
                        if let Some(inferred) = inferred {
                            if let Some(pos) = find_var_pos(content, name) {
                                let key = (pos.line, pos.character);
                                if !seen.contains(&key) {
                                    seen.insert(key);
                                    hints.push(InlayHint {
                                        position: pos,
                                        label: InlayHintLabel::String(format!(": {}", inferred)),
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
                    }
                }
            }
            Statement::Loop { body, .. } => {
                collect_hints_from_statements(body, content, doc, hints, seen);
            }
            _ => {}
        }
    }
}

fn find_var_pos(content: &str, var_name: &str) -> Option<Position> {
    let mut in_struct_def = false;
    let mut brace_depth: i32 = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track struct/enum definitions to skip them
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Detect struct definition start: "Name: {" or "Name<T>: {"
            if let Some(colon_pos) = trimmed.find(':') {
                let before = trimmed[..colon_pos].trim();
                let after = trimmed[colon_pos + 1..].trim();

                // Check for PascalCase type definition (might have generics like Name<T>)
                let base_name = name_utils::strip_generics(before);
                if !base_name.is_empty()
                    && base_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    && !before.contains('=')
                    && (after.starts_with('{') || after.contains('{'))
                {
                    in_struct_def = true;
                    brace_depth = 0;
                }
            }

            // Track brace depth
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

        // Skip lines inside struct definitions
        if in_struct_def {
            continue;
        }

        // Look for variable declaration (must have = operator, not just :)
        if let Some(pos) = line.find(var_name) {
            let before = pos == 0
                || line
                    .as_bytes()
                    .get(pos - 1)
                    .map(|&b| b.is_ascii_whitespace())
                    .unwrap_or(true);
            let after = &line[pos + var_name.len()..].trim_start();

            // Only match if followed by = (with optional : for type annotation)
            // This excludes struct field definitions like "name: Type"
            if before
                && (after.starts_with('=')
                    || after.starts_with(":=")
                    || after.starts_with("::=")
                    || (after.starts_with(':') && after.contains('=')))
            {
                return Some(Position {
                    line: line_num as u32,
                    character: (pos + var_name.len()) as u32,
                });
            }
        }
    }
    None
}
