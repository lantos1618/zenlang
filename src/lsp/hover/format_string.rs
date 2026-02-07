// Format string parsing and hover

use lsp_types::Position;
use std::collections::HashMap;

use super::expressions::analyze_expression_hover;
use super::structs::{extract_struct_name_from_type, find_struct_definition_in_documents};
use crate::ast::AstType;
use crate::lexer::Lexer;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::char_pos_to_byte_pos;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;
use crate::parser::Parser;

/// Handle format string field access (e.g., ${person.name})
pub fn get_format_string_field_hover(
    content: &str,
    position: Position,
    symbol_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let byte_pos = char_pos_to_byte_pos(current_line, char_pos);

    // Find format strings: "${...}" but not "\${...}" (escaped)
    // Limit search to prevent infinite loops
    use crate::lsp::search_limits::MAX_ITERATIONS;
    let mut search_pos = 0;
    let mut iterations = 0;

    while iterations < MAX_ITERATIONS {
        iterations += 1;

        let search_range = search_pos..byte_pos;
        if search_range.is_empty() {
            break;
        }

        let actual_dollar_pos = if let Some(dollar_pos) = current_line[search_range].rfind('$') {
            search_pos + dollar_pos
        } else {
            // No more dollar signs found
            break;
        };

        // Check if it's escaped: \${...
        if actual_dollar_pos > 0 && current_line.as_bytes()[actual_dollar_pos - 1] == b'\\' {
            // This is escaped, skip it by moving past this position
            if actual_dollar_pos == 0 {
                break; // Can't go further back
            }
            search_pos = actual_dollar_pos - 1;
            continue;
        }

        let after_dollar = &current_line[actual_dollar_pos + 1..];
        if after_dollar.starts_with('{') {
            // We're in a format string
            // Extract the expression inside ${...}
            if let Some(close_brace) = after_dollar.find('}') {
                let expr_str = &after_dollar[1..close_brace];

                // Only parse if the expression is reasonable length (prevent DoS)
                if expr_str.len() > 1000 {
                    // Expression too long, skip
                    if actual_dollar_pos == 0 {
                        break;
                    }
                    search_pos = actual_dollar_pos.saturating_sub(1);
                    continue;
                }

                let expr_start = actual_dollar_pos + 2; // After "${

                // Parse the expression as valid Zen code (with error handling)
                let lexer = Lexer::new(expr_str);
                let mut parser = Parser::new(lexer);

                // Try parsing, but don't let errors block us
                if let Ok(expr) = parser.parse_expression() {
                    // Determine what part of the expression we're hovering on
                    let relative_pos = char_pos.saturating_sub(expr_start);

                    // Find the symbol at the relative position in the expression
                    if let Some(hover_info) = analyze_expression_hover(
                        &expr,
                        expr_str,
                        relative_pos,
                        symbol_name,
                        local_symbols,
                        store,
                    ) {
                        return Some(hover_info);
                    }
                }

                // Fallback: if parsing fails, try simple string matching
                let expr = expr_str.trim();
                let relative_pos = char_pos.saturating_sub(expr_start);

                // Check if symbol_name contains a dot (field access like "person.name")
                if symbol_name.contains('.') {
                    // We're hovering on a field access - determine which part
                    if let Some(dot_pos_in_symbol) = symbol_name.find('.') {
                        let var_part = &symbol_name[..dot_pos_in_symbol];
                        let field_part = &symbol_name[dot_pos_in_symbol + 1..];

                        // Check if the expression matches this pattern
                        if let Some(dot_pos_in_expr) = expr.find('.') {
                            let before_dot = expr[..dot_pos_in_expr].trim();
                            let after_dot = expr[dot_pos_in_expr + 1..].trim();

                            if before_dot == var_part && after_dot == field_part {
                                // Determine which part we're hovering on based on cursor position
                                if relative_pos <= dot_pos_in_expr {
                                    // Hovering on the variable part (person)
                                    return handle_variable_hover(var_part, local_symbols, store);
                                } else {
                                    // Hovering on the field part (name)
                                    return handle_field_hover(
                                        var_part,
                                        field_part,
                                        content,
                                        local_symbols,
                                        store,
                                    );
                                }
                            }
                        }
                    }
                }

                // Check if the expression contains a dot (field access)
                if let Some(dot_pos) = expr.find('.') {
                    let before_dot = expr[..dot_pos].trim();
                    let after_dot = expr[dot_pos + 1..].trim();

                    // Check if we're hovering on the field name
                    let field_start = dot_pos + 1;
                    let field_end = expr.len();

                    if relative_pos >= field_start
                        && relative_pos <= field_end
                        && after_dot == symbol_name
                    {
                        // We're hovering on the field name
                        return handle_field_hover(
                            before_dot,
                            after_dot,
                            content,
                            local_symbols,
                            store,
                        );
                    } else if relative_pos < dot_pos && before_dot == symbol_name {
                        // We're hovering on the variable name (e.g., "person" in "${person.name}")
                        return handle_variable_hover(before_dot, local_symbols, store);
                    }
                } else {
                    // Check if we're hovering on a simple variable name (e.g., "person" in "${person}")
                    // Handle both exact match and partial match (cursor might be in middle of word)
                    let expr_trimmed = expr.trim();
                    if expr_trimmed == symbol_name
                        || expr_trimmed.starts_with(symbol_name)
                        || symbol_name.starts_with(expr_trimmed)
                    {
                        // Check if cursor is within the variable name
                        let symbol_start_in_expr = expr_trimmed.find(symbol_name).unwrap_or(0);
                        let symbol_end_in_expr = symbol_start_in_expr + symbol_name.len();

                        if relative_pos >= symbol_start_in_expr
                            && relative_pos <= symbol_end_in_expr
                        {
                            return handle_variable_hover(symbol_name, local_symbols, store);
                        }
                    }
                }
            }
        }

        // Move search position to continue looking backwards
        if actual_dollar_pos == 0 {
            break;
        }
        search_pos = actual_dollar_pos.saturating_sub(1);
    }

    None
}

/// Handle hover on a field access
fn handle_field_hover(
    var_name: &str,
    field_name: &str,
    content: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    // Find the struct type of the variable
    if let Some(var_info) = store.resolve_symbol_local_first(local_symbols, var_name) {
        // Get the struct type
        let struct_name = if let Some(AstType::Struct { name, .. }) = &var_info.type_info {
            Some(name.clone())
        } else if !content.is_empty() {
            // Try to infer from variable assignment
            super::inference::infer_variable_type(var_name, local_symbols, store)
                .and_then(|inferred_type| extract_struct_name_from_type(&inferred_type))
        } else {
            None
        };

        if let Some(struct_name) = struct_name {
            // Find the struct definition
            if let Some(struct_def) =
                find_struct_definition_in_documents(&struct_name, &store.documents)
            {
                // Find the field type
                for field in &struct_def.fields {
                    if field.name == field_name {
                        return Some(format!(
                            "```zen\n{}: {}\n```\n\n**Field of:** `{}`\n\n**Type:** `{}`",
                            field_name,
                            format_type(&field.type_),
                            struct_name,
                            format_type(&field.type_)
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Handle hover on a variable
fn handle_variable_hover(
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    super::structs::handle_variable_hover(var_name, local_symbols, store)
}
