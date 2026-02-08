// Expression analysis for hover

use std::collections::HashMap;

use crate::ast::{AstType, Expression};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;
use crate::type_context::TypeContext;

/// Analyze expression AST to determine hover information
pub fn analyze_expression_hover(
    expr: &Expression,
    expr_str: &str,
    relative_pos: usize,
    symbol_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
    type_ctx: Option<&TypeContext>,
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
                if let Some(obj_type) =
                    resolve_expression_type(object, local_symbols, store, type_ctx)
                {
                    if let Some(type_name) = obj_type.base_name() {
                        // Try TypeContext first (authoritative), fall back to DocumentStore
                        let field_type = type_ctx
                            .and_then(|tc| tc.get_struct_field_type(type_name, member))
                            .or_else(|| {
                                store.find_struct_definition(type_name).and_then(|sd| {
                                    sd.fields
                                        .iter()
                                        .find(|f| &f.name == member)
                                        .map(|f| f.type_.clone())
                                })
                            });

                        if let Some(field_type) = field_type {
                            let type_str = format_type(&field_type);
                            return Some(format!(
                                "```zen\n{}: {}\n```\n\n**Field of:** `{}`\n\n**Type:** `{}`",
                                member, type_str, type_name, type_str
                            ));
                        }
                    }
                }
            } else if relative_pos < last_dot_pos {
                // We're hovering on the object part
                let is_hovering_on_object = if symbol_name.contains('.') {
                    if let Some(dot_pos) = symbol_name.find('.') {
                        let var_part = &symbol_name[..dot_pos];
                        matches!(object.as_ref(), Expression::Identifier(n) if n == var_part)
                    } else {
                        false
                    }
                } else {
                    matches!(object.as_ref(), Expression::Identifier(n) if n == symbol_name)
                };

                if is_hovering_on_object {
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
                        type_ctx,
                    );
                }
            }
        }
        Expression::MethodCall { object, method, .. } => {
            if let Some(dot_pos) = expr_str.rfind('.') {
                let method_start = dot_pos + 1;
                let method_end = expr_str.find('(').unwrap_or(expr_str.len());

                if relative_pos >= method_start
                    && relative_pos <= method_end
                    && method == symbol_name
                {
                    // Try to get method signature from TypeContext
                    if let Some(obj_type) =
                        resolve_expression_type(object, local_symbols, store, type_ctx)
                    {
                        if let Some(type_name) = obj_type.base_name() {
                            if let Some(tc) = type_ctx {
                                if let Some(ret_type) = tc.get_method_return_type(type_name, method)
                                {
                                    let ret_str = format_type(&ret_type);
                                    let params_str = tc
                                        .get_method_params(type_name, method)
                                        .map(|params| {
                                            params
                                                .iter()
                                                .map(|(n, t)| format!("{}: {}", n, format_type(t)))
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        })
                                        .unwrap_or_default();
                                    return Some(format!(
                                        "```zen\n{}.{} = ({}) {}\n```\n\n**Method**",
                                        type_name, method, params_str, ret_str
                                    ));
                                }
                            }
                        }
                    }

                    // Fallback: basic method display
                    let obj_name = match object.as_ref() {
                        Expression::Identifier(name) => name.clone(),
                        _ => "object".to_string(),
                    };
                    return Some(format!(
                        "```zen\n{}.{}()\n```\n\n**Method**",
                        obj_name, method
                    ));
                } else if relative_pos < dot_pos {
                    return analyze_expression_hover(
                        object,
                        &expr_str[..dot_pos],
                        relative_pos,
                        symbol_name,
                        local_symbols,
                        store,
                        type_ctx,
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

/// Resolve the type of an expression using TypeContext when available
pub fn resolve_expression_type(
    expr: &Expression,
    local_symbols: &HashMap<String, SymbolInfo>,
    store: &DocumentStore,
    type_ctx: Option<&TypeContext>,
) -> Option<AstType> {
    match expr {
        Expression::Identifier(var_name) => {
            // Try symbol table first (has type_info from analysis)
            if let Some(var_info) = store.resolve_symbol_local_first(local_symbols, var_name) {
                return var_info.type_info.clone();
            }
        }
        Expression::MemberAccess { object, member } => {
            if let Some(obj_type) = resolve_expression_type(object, local_symbols, store, type_ctx)
            {
                if let Some(type_name) = obj_type.base_name() {
                    // Try TypeContext first (authoritative)
                    if let Some(field_type) =
                        type_ctx.and_then(|tc| tc.get_struct_field_type(type_name, member))
                    {
                        return Some(field_type);
                    }
                    // Fall back to DocumentStore
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
