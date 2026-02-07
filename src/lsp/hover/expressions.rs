// Expression analysis for hover

use std::collections::HashMap;

use crate::ast::{AstType, Expression};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;

/// Analyze expression AST to determine hover information
pub fn analyze_expression_hover(
    expr: &Expression,
    expr_str: &str,
    relative_pos: usize,
    symbol_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<String> {
    match expr {
        Expression::MemberAccess { object, member } => {
            // Find the last dot position (for nested member access)
            let mut last_dot_pos = 0;
            for (i, ch) in expr_str.char_indices() {
                if ch == '.' {
                    last_dot_pos = i;
                }
            }

            let member_start = last_dot_pos + 1;
            let member_end = expr_str.len();

            // Check if symbol_name contains a dot (like "person.name")
            let is_hovering_on_field = if symbol_name.contains('.') {
                // Extract the field part from symbol_name (e.g., "name" from "person.name")
                if let Some(dot_pos) = symbol_name.find('.') {
                    let field_part = &symbol_name[dot_pos + 1..];
                    member == field_part
                        && relative_pos >= member_start
                        && relative_pos <= member_end
                } else {
                    false
                }
            } else {
                member == symbol_name && relative_pos >= member_start && relative_pos <= member_end
            };

            if is_hovering_on_field {
                if let Some(obj_type) = resolve_expression_type(object, local_symbols, store) {
                    if let Some(type_name) = obj_type.base_name() {
                        if let Some(struct_def) = store.find_struct_definition(type_name) {
                            for field in &struct_def.fields {
                                if &field.name == member {
                                    return Some(format!(
                                        "```zen\n{}: {}\n```\n\n**Field of:** `{}`\n\n**Type:** `{}`",
                                        member,
                                        format_type(&field.type_),
                                        type_name,
                                        format_type(&field.type_)
                                    ));
                                }
                            }
                        }
                    }
                }
            } else if relative_pos < last_dot_pos {
                // We're hovering on the object part - check if symbol_name matches
                let is_hovering_on_object = if symbol_name.contains('.') {
                    // Extract the variable part from symbol_name (e.g., "person" from "person.name")
                    if let Some(dot_pos) = symbol_name.find('.') {
                        let var_part = &symbol_name[..dot_pos];
                        // Check if the object is an identifier matching var_part
                        if let Expression::Identifier(obj_name) = object.as_ref() {
                            obj_name == var_part
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    // Check if object matches symbol_name
                    if let Expression::Identifier(obj_name) = object.as_ref() {
                        obj_name == symbol_name
                    } else {
                        false
                    }
                };

                if is_hovering_on_object {
                    // Recursively analyze the object
                    let effective_symbol = symbol_name
                        .find('.')
                        .map(|pos| &symbol_name[..pos])
                        .unwrap_or(symbol_name);
                    return analyze_expression_hover(
                        object,
                        &expr_str[..last_dot_pos],
                        relative_pos,
                        effective_symbol,
                        local_symbols,
                        store,
                    );
                }
            }
        }
        Expression::MethodCall { object, method, .. } => {
            // Similar to MemberAccess but for method calls
            if let Some(dot_pos) = expr_str.rfind('.') {
                let method_start = dot_pos + 1;
                let method_end = expr_str.find('(').unwrap_or(expr_str.len());

                if relative_pos >= method_start
                    && relative_pos <= method_end
                    && method == symbol_name
                {
                    // Hovering on method name - could show method signature
                    // For now, just show that it's a method
                    return Some(format!(
                        "```zen\n{}.{}()\n```\n\n**Method**",
                        match object.as_ref() {
                            Expression::Identifier(name) => name.clone(),
                            _ => "object".to_string(),
                        },
                        method
                    ));
                } else if relative_pos < dot_pos {
                    // Hovering on object
                    return analyze_expression_hover(
                        object,
                        &expr_str[..dot_pos],
                        relative_pos,
                        symbol_name,
                        local_symbols,
                        store,
                    );
                }
            }
        }
        Expression::Identifier(var_name) => {
            if var_name == symbol_name {
                return super::structs::handle_variable_hover(var_name, local_symbols, store);
            }
        }
        _ => {}
    }
    None
}

/// Resolve the type of an expression
pub fn resolve_expression_type(
    expr: &Expression,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
) -> Option<AstType> {
    match expr {
        Expression::Identifier(var_name) => {
            if let Some(var_info) = store.resolve_symbol_local_first(local_symbols, var_name) {
                return var_info.type_info.clone();
            }
        }
        Expression::MemberAccess { object, member } => {
            if let Some(obj_type) = resolve_expression_type(object, local_symbols, store) {
                if let Some(type_name) = obj_type.base_name() {
                    if let Some(struct_def) = store.find_struct_definition(type_name) {
                        for field in &struct_def.fields {
                            if field.name == *member {
                                return Some(field.type_.clone());
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    None
}
