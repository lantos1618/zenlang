// Format string parsing and hover

use lsp_types::Position;
use std::collections::HashMap;

use super::expressions::analyze_expression_hover;
use crate::lexer::Lexer;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::char_pos_to_byte_pos;
use crate::lsp::types::*;
use crate::parser::Parser;
use crate::type_context::TypeContext;

/// Handle format string field access (e.g., ${person.name})
pub fn get_format_string_field_hover(
    content: &str,
    position: Position,
    symbol_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
    type_ctx: Option<&TypeContext>,
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
                        type_ctx,
                    ) {
                        return Some(hover_info);
                    }
                }

                // Parser handles all valid Zen expressions.
                // No string-based fallback needed.
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
