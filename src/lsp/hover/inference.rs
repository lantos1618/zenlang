use lsp_types::Url;
use std::collections::HashMap;

use crate::ast::{Declaration, Expression, Statement};
use crate::lsp::type_query::TypeQuery;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;

pub fn extract_type_from_line(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let after_colon = &line[colon_pos + 1..];
        let type_end = after_colon
            .find('=')
            .or_else(|| after_colon.find('{'))
            .or_else(|| after_colon.find('('))
            .unwrap_or(after_colon.len());
        let type_str = after_colon[..type_end].trim();
        if !type_str.is_empty() {
            return Some(type_str.to_string());
        }
    }
    None
}

fn find_variable_in_ast(
    ast: &[Declaration],
    var_name: &str,
) -> Option<(Option<crate::ast::AstType>, Option<Expression>)> {
    for decl in ast {
        if let Declaration::Function(func) = decl {
            if let Some(result) = find_variable_in_statements(&func.body, var_name) {
                return Some(result);
            }
        }
    }
    None
}

fn find_variable_in_statements(
    stmts: &[Statement],
    var_name: &str,
) -> Option<(Option<crate::ast::AstType>, Option<Expression>)> {
    for stmt in stmts {
        match stmt {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                ..
            } if name == var_name => {
                return Some((type_.clone(), initializer.clone()));
            }
            Statement::Loop { body, .. } => {
                if let Some(result) = find_variable_in_statements(body, var_name) {
                    return Some(result);
                }
            }
            Statement::Block { statements, .. } => {
                if let Some(result) = find_variable_in_statements(statements, var_name) {
                    return Some(result);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn infer_variable_type(
    _content: &str,
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    documents: Option<&HashMap<Url, Document>>,
) -> Option<String> {
    if let Some(docs) = documents {
        for doc in docs.values() {
            let tq = TypeQuery::new(doc);
            if let Some(type_str) = tq.find_variable_type(var_name) {
                return Some(format!(
                    "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                    var_name, type_str, type_str
                ));
            }

            if let Some(ast) = &doc.ast {
                if let Some((type_opt, init_opt)) = find_variable_in_ast(ast, var_name) {
                    if let Some(type_) = type_opt {
                        let type_str = format_type(&type_);
                        return Some(format!(
                            "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                            var_name, type_str, type_str
                        ));
                    }

                    if let Some(init) = init_opt {
                        if let Some(type_str) = TypeQuery::infer_literal_type(&init) {
                            return Some(format!(
                                "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                                var_name, type_str, type_str
                            ));
                        }
                    }
                }
            }
        }
    }

    if let Some(sym) = local_symbols
        .get(var_name)
        .or_else(|| stdlib_symbols.get(var_name))
        .or_else(|| workspace_symbols.get(var_name))
    {
        if let Some(type_info) = &sym.type_info {
            let type_str = format_type(type_info);
            return Some(format!(
                "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                var_name, type_str, type_str
            ));
        }
    }

    None
}
