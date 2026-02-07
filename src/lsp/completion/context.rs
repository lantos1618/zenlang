//! Completion context detection
//!
//! Determines what kind of completion to provide based on cursor position.

use lsp_types::*;
use std::collections::HashSet;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::char_pos_to_byte_pos;
use crate::lsp::type_inference::infer_receiver_type_with_context;
use crate::lsp::types::ZenCompletionContext;
use crate::lsp::utils::format_type;
use crate::name_utils;

/// Detect the completion context at the given position
pub fn get_completion_context(
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
            if let Some(std_pos) = before_cursor.rfind("@std.") {
                let after_std = &before_cursor[std_pos + 5..];
                if !after_std.contains('.') {
                    return Some(ZenCompletionContext::ModulePath {
                        base: "@std".to_string(),
                    });
                }
            }
        }
    }

    // Check if we're in a pattern match context: `expr ? | ▊`
    if let Some(matched_type) = detect_pattern_match_context(content, position) {
        return Some(ZenCompletionContext::PatternMatch { matched_type });
    }

    // Check if we're inside a struct literal: `StructName { field: value, ▊`
    if let Some(struct_name) = detect_struct_literal_context(content, position) {
        return Some(ZenCompletionContext::StructLiteral { struct_name });
    }

    // Check if we're after a dot
    if char_pos > 0 && line.chars().nth(char_pos - 1) == Some('.') {
        let chars: Vec<char> = line.chars().collect();
        let mut start = if char_pos > 1 {
            char_pos - 2
        } else {
            0
        };
        let mut paren_depth = 0;

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

/// Detect if cursor is in a pattern match context: `expr ? | ▊`
/// Returns the type of the matched expression if we're right after `|`
fn detect_pattern_match_context(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let byte_pos = char_pos_to_byte_pos(line, char_pos);
    let before_cursor = &line[..byte_pos];
    let trimmed = before_cursor.trim_end();

    // Check if we're right after `|` (pattern arm start)
    if !trimmed.ends_with('|') {
        return None;
    }

    // Look backwards to find the `?` and the expression before it
    // Pattern: `expr ?` or `expr?` followed by `| pattern1 { } | ▊`
    let before_pipe = trimmed.trim_end_matches('|').trim_end();

    // If there's a `{` before the pipe, we might be in a multi-arm pattern
    // Look for the `?` that starts this pattern match
    let full_context = if before_pipe.is_empty() {
        // Check previous lines for the `?`
        let mut context = String::new();
        for i in (0..position.line as usize).rev() {
            let prev_line = lines[i];
            context = format!("{}\n{}", prev_line, context);
            if prev_line.contains('?') {
                break;
            }
            // Don't look too far back
            if position.line as usize - i > 5 {
                break;
            }
        }
        context + before_cursor
    } else {
        before_cursor.to_string()
    };

    // Find the `?` and extract the expression before it
    if let Some(question_pos) = full_context.rfind('?') {
        let before_question = full_context[..question_pos].trim_end();
        // Extract the last expression (simple heuristic: last word or identifier chain)
        let expr = extract_expression_before(before_question);
        if !expr.is_empty() {
            // Return the expression - the completion handler will resolve its type
            return Some(expr);
        }
    }

    None
}

/// Extract the expression immediately before the cursor position
fn extract_expression_before(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut end = chars.len();
    let mut start = end;
    let mut paren_depth = 0;

    // Skip trailing whitespace
    while start > 0 && chars[start - 1].is_whitespace() {
        start -= 1;
        end -= 1;
    }

    // Walk backwards to find the start of the expression
    while start > 0 {
        start -= 1;
        let ch = chars[start];
        match ch {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                } else {
                    start += 1;
                    break;
                }
            }
            ' ' | '\t' | '\n' | '=' | '{' | ';' | ',' if paren_depth == 0 => {
                start += 1;
                break;
            }
            _ => {}
        }
    }

    chars[start..end]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// Detect if cursor is inside a struct literal and return the struct name.
fn detect_struct_literal_context(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let byte_pos = char_pos_to_byte_pos(line, char_pos);
    let before_cursor = &line[..byte_pos];

    let chars: Vec<char> = before_cursor.chars().collect();
    let mut brace_depth = 0;
    let mut found_colon_after_comma_or_brace = false;
    let mut last_separator_pos = None;

    for (i, &ch) in chars.iter().enumerate().rev() {
        match ch {
            '}' => brace_depth += 1,
            '{' => {
                if brace_depth == 0 {
                    let before_brace = &before_cursor[..i].trim_end();

                    if found_colon_after_comma_or_brace {
                        return None;
                    }

                    let struct_name: String = before_brace
                        .chars()
                        .rev()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();

                    if struct_name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        return Some(struct_name);
                    }
                    return None;
                }
                brace_depth -= 1;
            }
            ':' if brace_depth == 0 => {
                if last_separator_pos.is_none() || last_separator_pos.is_some_and(|pos| i > pos) {
                    found_colon_after_comma_or_brace = true;
                }
            }
            ',' if brace_depth == 0 => {
                found_colon_after_comma_or_brace = false;
                last_separator_pos = Some(i);
            }
            _ => {}
        }
    }

    None
}

/// Get fields that have already been assigned in a struct literal
pub fn get_assigned_fields(content: &str, position: Position) -> HashSet<String> {
    let mut assigned = HashSet::new();
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return assigned;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let byte_pos = char_pos_to_byte_pos(line, char_pos);
    let before_cursor = &line[..byte_pos];

    if let Some(brace_pos) = before_cursor.rfind('{') {
        let inside_braces = &before_cursor[brace_pos + 1..];
        for part in inside_braces.split(',') {
            let part = part.trim();
            if let Some(colon_pos) = part.find(':') {
                let field_name = part[..colon_pos].trim();
                if !field_name.is_empty() {
                    assigned.insert(field_name.to_string());
                }
            }
        }
    }

    assigned
}

/// Get struct field completions for a struct literal context.
pub fn get_struct_literal_completions(
    struct_name: &str,
    doc: &crate::lsp::types::Document,
    content: &str,
    position: Position,
) -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    let assigned_fields = get_assigned_fields(content, position);

    // Try TypeContext first
    if let Some(type_ctx) = doc.type_context.as_ref() {
        if let Some(fields) = type_ctx.get_struct_fields(struct_name) {
            for (field_name, field_type) in fields {
                if assigned_fields.contains(field_name) {
                    continue;
                }

                completions.push(CompletionItem {
                    label: field_name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(format!("{}: {}", field_name, format_type(field_type))),
                    documentation: Some(Documentation::String(format!(
                        "Field of struct `{}`",
                        struct_name
                    ))),
                    insert_text: Some(format!("{}: $0", field_name)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    sort_text: Some(format!("0{}", field_name)),
                    ..Default::default()
                });
            }

            if !completions.is_empty() {
                return completions;
            }
        }
    }

    // Fallback: search document AST
    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let crate::ast::Declaration::Struct(struct_def) = decl {
                if struct_def.name == struct_name {
                    for field in &struct_def.fields {
                        if assigned_fields.contains(&field.name) {
                            continue;
                        }

                        completions.push(CompletionItem {
                            label: field.name.clone(),
                            kind: Some(CompletionItemKind::FIELD),
                            detail: Some(format!("{}: {}", field.name, format_type(&field.type_))),
                            documentation: Some(Documentation::String(format!(
                                "Field of struct `{}`",
                                struct_name
                            ))),
                            insert_text: Some(format!("{}: $0", field.name)),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            sort_text: Some(format!("0{}", field.name)),
                            ..Default::default()
                        });
                    }
                    break;
                }
            }
        }
    }

    completions
}

/// Get pattern match completions (enum variants, true/false, etc.)
pub fn get_pattern_match_completions(
    matched_expr: &str,
    doc: &crate::lsp::types::Document,
    content: &str,
    store: &crate::lsp::document_store::DocumentStore,
) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // First, try to infer the type of the matched expression
    let matched_type = infer_matched_expression_type(matched_expr, doc, content, store);

    if let Some(type_name) = matched_type {
        // Check if it's a boolean
        if type_name == "bool" {
            completions.push(CompletionItem {
                label: "true".to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("Boolean true".to_string()),
                insert_text: Some("true { $0 }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some("0true".to_string()),
                ..Default::default()
            });
            completions.push(CompletionItem {
                label: "false".to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("Boolean false".to_string()),
                insert_text: Some("false { $0 }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some("0false".to_string()),
                ..Default::default()
            });
            return completions;
        }

        // Check TypeContext for enum variants
        if let Some(type_ctx) = doc.type_context.as_ref() {
            // Check if it's an enum
            if let Some(variants) = type_ctx.enums.get(&type_name) {
                for (variant_name, variant_type) in variants {
                    let (label, insert_text) = if variant_type.is_some() {
                        // Variant with payload: Some(value)
                        (
                            format!("{}()", variant_name),
                            format!("{}($1) {{ $0 }}", variant_name),
                        )
                    } else {
                        // Variant without payload: None
                        (variant_name.clone(), format!("{} {{ $0 }}", variant_name))
                    };

                    completions.push(CompletionItem {
                        label,
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: Some(format!("Variant of {}", type_name)),
                        insert_text: Some(insert_text),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        sort_text: Some(format!("0{}", variant_name)),
                        ..Default::default()
                    });
                }
            }
        }

        // Also check well-known types like Option and Result
        let wk = crate::well_known::well_known();
        if wk.is_option(&type_name) || type_name.starts_with("Option") {
            if completions.is_empty() {
                completions.push(CompletionItem {
                    label: format!("{}()", wk.some_name()),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some("Option with value".to_string()),
                    insert_text: Some(format!("{}($1) {{ $0 }}", wk.some_name())),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    sort_text: Some("0Some".to_string()),
                    ..Default::default()
                });
                completions.push(CompletionItem {
                    label: wk.none_name().to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some("Option without value".to_string()),
                    insert_text: Some(format!("{} {{ $0 }}", wk.none_name())),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    sort_text: Some("0None".to_string()),
                    ..Default::default()
                });
            }
        } else if (wk.is_result(&type_name) || type_name.starts_with("Result"))
            && completions.is_empty()
        {
            completions.push(CompletionItem {
                label: format!("{}()", wk.ok_name()),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("Success result".to_string()),
                insert_text: Some(format!("{}($1) {{ $0 }}", wk.ok_name())),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some("0Ok".to_string()),
                ..Default::default()
            });
            completions.push(CompletionItem {
                label: format!("{}()", wk.err_name()),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("Error result".to_string()),
                insert_text: Some(format!("{}($1) {{ $0 }}", wk.err_name())),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some("0Err".to_string()),
                ..Default::default()
            });
        }
    }

    // Always add true/false as fallback options for general pattern matching
    if completions.is_empty() {
        completions.push(CompletionItem {
            label: "true".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean pattern".to_string()),
            insert_text: Some("true { $0 }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some("1true".to_string()),
            ..Default::default()
        });
        completions.push(CompletionItem {
            label: "false".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean pattern".to_string()),
            insert_text: Some("false { $0 }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some("1false".to_string()),
            ..Default::default()
        });
    }

    completions
}

/// Infer the type of a matched expression for pattern matching
fn infer_matched_expression_type(
    expr: &str,
    doc: &crate::lsp::types::Document,
    _content: &str,
    _store: &crate::lsp::document_store::DocumentStore,
) -> Option<String> {
    // Try TypeContext first
    if let Some(type_ctx) = doc.type_context.as_ref() {
        // Check if expr is a variable
        for (key, var_type) in &type_ctx.variables {
            if key.ends_with(&format!("::{}", expr)) || key == expr {
                return Some(format_type(var_type));
            }
        }

        // Check if expr is a function call - look up return type
        if let Some(paren_pos) = expr.find('(') {
            let func_name = expr[..paren_pos].trim();
            if let Some(func_type) = type_ctx.functions.get(func_name) {
                return Some(format_type(&func_type.return_type));
            }
        }

        // Check for field access
        if expr.contains('.') {
            let parts: Vec<&str> = expr.split('.').collect();
            if parts.len() >= 2 {
                // Try to resolve the type through field access
                let base = parts[0];
                for (key, var_type) in &type_ctx.variables {
                    if key.ends_with(&format!("::{}", base)) || key == base {
                        let mut current_type = format_type(var_type);
                        for field_name in &parts[1..] {
                            let struct_name = name_utils::strip_generics(&current_type);
                            if let Some(fields) = type_ctx.structs.get(struct_name) {
                                if let Some((_, field_type)) =
                                    fields.iter().find(|(n, _)| n == *field_name)
                                {
                                    current_type = format_type(field_type);
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        }
                        return Some(current_type);
                    }
                }
            }
        }
    }

    // Check for comparison expressions (result is bool)
    if expr.contains("==") || expr.contains("!=") || expr.contains('<') || expr.contains('>') {
        return Some("bool".to_string());
    }

    None
}
