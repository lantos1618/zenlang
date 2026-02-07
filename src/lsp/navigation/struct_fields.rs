//! Struct field navigation
//!
//! Handles go-to-definition for struct field access (e.g., `point.x` -> field definition)

use lsp_types::*;

use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::Document;
use crate::name_utils;

/// Find struct field definition for navigation.
/// Handles `receiver.field` patterns and navigates to the field in the struct definition.
pub fn find_struct_field_definition(
    content: &str,
    position: Position,
    doc: &Document,
    store: &std::sync::MutexGuard<'_, DocumentStore>,
) -> Option<Location> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let chars: Vec<char> = line.chars().collect();

    // Find if we're on a field access pattern (receiver.field)
    let mut dot_pos = None;
    for i in (0..char_pos.min(chars.len())).rev() {
        if chars[i] == '.' {
            dot_pos = Some(i);
            break;
        } else if chars[i] == ' ' || chars[i] == '(' || chars[i] == ')' || chars[i] == '=' {
            break;
        }
    }

    let dot = dot_pos?;

    // Extract the field name after the dot
    let mut field_end = dot + 1;
    while field_end < chars.len() && (chars[field_end].is_alphanumeric() || chars[field_end] == '_')
    {
        field_end += 1;
    }
    let field_name: String = chars[(dot + 1)..field_end].iter().collect();

    // Check if this is a method call (has parentheses after) - skip if so
    if field_end < chars.len() && chars[field_end] == '(' {
        return None;
    }

    // Extract the receiver expression before the dot
    let mut obj_start = dot;
    let mut paren_depth = 0;
    while obj_start > 0 {
        obj_start -= 1;
        match chars[obj_start] {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                } else {
                    obj_start += 1;
                    break;
                }
            }
            ' ' | '\t' | '=' | '{' | '[' | ',' | ';' if paren_depth == 0 => {
                obj_start += 1;
                break;
            }
            _ => {}
        }
    }

    let receiver: String = chars[obj_start..dot].iter().collect();
    let receiver = receiver.trim();

    // Infer the type of the receiver
    let receiver_type = infer_receiver_type(receiver, content, doc, store)?;

    log::debug!(
        "[LSP] Struct field navigation: receiver='{}', type='{}', field='{}'",
        receiver,
        receiver_type,
        field_name
    );

    find_field_in_struct(&receiver_type, &field_name, doc, store)
}

/// Infer the type of a receiver expression
fn infer_receiver_type(
    receiver: &str,
    content: &str,
    doc: &Document,
    store: &std::sync::MutexGuard<'_, DocumentStore>,
) -> Option<String> {
    // Try TypeContext first - most accurate source
    if let Some(type_ctx) = doc.type_context.as_ref() {
        // Check variables with various key formats
        // Format: "function_name::var_name" or just "var_name"
        for (key, var_type) in &type_ctx.variables {
            // Match "func::receiver" or just "receiver"
            if key.ends_with(&format!("::{}", receiver)) || key == receiver {
                let type_str = crate::lsp::utils::format_type(var_type);
                // Extract struct name from type (handle generics like Vec<T>)
                let struct_name = name_utils::strip_generics(&type_str).trim();
                if !struct_name.is_empty() {
                    return Some(struct_name.to_string());
                }
            }
        }

        // Also check if receiver is a struct type itself (for Type.field access)
        if type_ctx.structs.contains_key(receiver) {
            return Some(receiver.to_string());
        }
    }

    // Try local symbols
    if let Some(symbol) = doc.symbols.get(receiver) {
        if let Some(type_info) = &symbol.type_info {
            let type_str = crate::lsp::utils::format_type(type_info);
            let struct_name = name_utils::strip_generics(&type_str).trim();
            return Some(struct_name.to_string());
        }
    }

    // Try chained field access: for "a.b.c", we need to resolve step by step
    if receiver.contains('.') {
        return infer_chained_receiver_type(receiver, doc);
    }

    // Fallback: search for variable declaration in content
    for line in content.lines() {
        if line.contains(&format!("{} =", receiver)) || line.contains(&format!("{}=", receiver)) {
            // Check for struct literal: receiver = TypeName { ... }
            if let Some(eq_pos) = line.find('=') {
                let rhs = line[eq_pos + 1..].trim();
                if let Some(brace_pos) = rhs.find('{') {
                    let type_name = rhs[..brace_pos].trim();
                    if type_name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        return Some(type_name.to_string());
                    }
                }
            }
            // Check for type annotation: receiver: TypeName = ...
            if let Some(colon_pos) = line.find(':') {
                if let Some(eq_pos) = line.find('=') {
                    if colon_pos < eq_pos {
                        let type_str = line[colon_pos + 1..eq_pos].trim();
                        if !type_str.is_empty() {
                            let struct_name = name_utils::strip_generics(type_str).trim();
                            return Some(struct_name.to_string());
                        }
                    }
                }
            }
        }
    }

    crate::lsp::type_inference::infer_receiver_type_from_store(receiver, store)
}

/// Infer type for chained field access like "point.position" -> get type of position field
fn infer_chained_receiver_type(receiver: &str, doc: &Document) -> Option<String> {
    let parts: Vec<&str> = receiver.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let type_ctx = doc.type_context.as_ref()?;

    // Start with the first part's type
    let mut current_type: Option<String> = None;

    for (key, var_type) in &type_ctx.variables {
        if key.ends_with(&format!("::{}", parts[0])) || key == parts[0] {
            current_type = Some(crate::lsp::utils::format_type(var_type));
            break;
        }
    }

    let mut current_type = current_type?;

    // Resolve each subsequent field
    for field_name in &parts[1..] {
        let struct_name = name_utils::strip_generics(&current_type).trim();
        if let Some(fields) = type_ctx.structs.get(struct_name) {
            let field_type = fields.iter().find(|(name, _)| name == *field_name);
            if let Some((_, field_ast_type)) = field_type {
                current_type = crate::lsp::utils::format_type(field_ast_type);
            } else {
                return None; // Field not found
            }
        } else {
            return None; // Struct not found
        }
    }

    // Return the final type (struct name without generics)
    let struct_name = name_utils::strip_generics(&current_type).trim();
    Some(struct_name.to_string())
}

/// Find a field definition within a struct
fn find_field_in_struct(
    type_name: &str,
    field_name: &str,
    doc: &Document,
    store: &std::sync::MutexGuard<'_, DocumentStore>,
) -> Option<Location> {
    let uri = &doc.uri;

    // Try to find struct in current document's AST
    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let crate::ast::Declaration::Struct(struct_def) = decl {
                if struct_def.name == type_name {
                    for field in &struct_def.fields {
                        if field.name == field_name {
                            if let Some(range) =
                                find_field_in_struct_content(&doc.content, type_name, field_name)
                            {
                                return Some(Location {
                                    uri: uri.clone(),
                                    range,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Search other documents
    for (other_uri, other_doc) in &store.documents {
        if other_uri == uri {
            continue;
        }

        if let Some(ast) = &other_doc.ast {
            for decl in ast {
                if let crate::ast::Declaration::Struct(struct_def) = decl {
                    if struct_def.name == type_name {
                        for field in &struct_def.fields {
                            if field.name == field_name {
                                if let Some(range) = find_field_in_struct_content(
                                    &other_doc.content,
                                    type_name,
                                    field_name,
                                ) {
                                    return Some(Location {
                                        uri: other_uri.clone(),
                                        range,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find a field definition in struct content using text search
fn find_field_in_struct_content(content: &str, type_name: &str, field_name: &str) -> Option<Range> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_struct = false;
    let mut brace_depth = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if !in_struct {
            // Pattern: TypeName: { or TypeName = {
            if (trimmed.starts_with(&format!("{}: ", type_name))
                || trimmed.starts_with(&format!("{}:", type_name))
                || trimmed.starts_with(&format!("{} = ", type_name)))
                && trimmed.contains('{')
            {
                in_struct = true;
                brace_depth = 1;
                continue;
            }
        }

        if in_struct {
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            in_struct = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if brace_depth > 0 {
                let field_pattern = format!("{}:", field_name);
                if let Some(pos) = trimmed.find(&field_pattern) {
                    let before_ok = pos == 0
                        || !trimmed[..pos]
                            .chars()
                            .last()
                            .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    if before_ok {
                        let char_pos = line.find(&field_pattern).unwrap_or(0);
                        return Some(Range {
                            start: Position {
                                line: line_idx as u32,
                                character: char_pos as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (char_pos + field_name.len()) as u32,
                            },
                        });
                    }
                }
            }
        }
    }

    None
}
