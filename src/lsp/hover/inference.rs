use lsp_types::Url;
use std::collections::HashMap;

use crate::ast::Program;
use crate::ast::{Declaration, Expression, Statement};
use crate::lsp::compiler_integration::CompilerIntegration;
use crate::lsp::types::*;
use crate::lsp::utils::format_type;
use crate::stdlib_types::stdlib_types;

/// Extract type from a definition line (fallback for when AST is not available)
pub fn extract_type_from_line(line: &str) -> Option<String> {
    // Look for type annotations: name: Type or name: Type =
    if let Some(colon_pos) = line.find(':') {
        let after_colon = &line[colon_pos + 1..];
        // Extract type (stop at =, {, or end)
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

/// Find variable declaration in AST and return type info
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

/// Infer variable type from assignment (AST-first approach)
pub fn infer_variable_type(
    _content: &str,
    var_name: &str,
    local_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    documents: Option<&HashMap<Url, Document>>,
) -> Option<String> {
    // Try AST-based lookup first
    if let Some(docs) = documents {
        for doc in docs.values() {
            if let Some(ast) = &doc.ast {
                if let Some((type_opt, init_opt)) = find_variable_in_ast(ast, var_name) {
                    // If we have explicit type annotation
                    if let Some(type_) = type_opt {
                        let type_str = format_type(&type_);
                        return Some(format!(
                            "```zen\n{}: {}\n```\n\n**Type:** `{}`",
                            var_name, type_str, type_str
                        ));
                    }

                    // Try to infer from initializer using compiler integration
                    if let Some(init) = init_opt {
                        let program = Program {
                            declarations: ast.clone(),
                            statements: vec![],
                        };
                        let mut compiler = CompilerIntegration::new();
                        if let Ok(inferred_type) = compiler.infer_expression_type(&program, &init) {
                            let type_str = format_type(&inferred_type);
                            return Some(format!(
                                "```zen\n{}: {}\n```\n\n**Type:** `{}`\n\n**Inferred from:** initializer expression",
                                var_name, type_str, type_str
                            ));
                        }

                        // Fallback: try direct expression type inference
                        if let Some(type_str) = infer_type_from_ast_expr(&init) {
                            return Some(format!(
                                "```zen\n{}: {}\n```\n\n**Type:** `{}`\n\n**Inferred from:** expression analysis",
                                var_name, type_str, type_str
                            ));
                        }
                    }
                }
            }
        }
    }

    // Fallback: look up in symbol tables
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

/// Infer type from AST expression
fn infer_type_from_ast_expr(expr: &Expression) -> Option<String> {
    use crate::well_known::well_known;
    let wk = well_known();

    match expr {
        Expression::Integer8(_) => Some("i8".to_string()),
        Expression::Integer16(_) => Some("i16".to_string()),
        Expression::Integer32(_) => Some("i32".to_string()),
        Expression::Integer64(_) => Some("i64".to_string()),
        Expression::Float32(_) => Some("f32".to_string()),
        Expression::Float64(_) => Some("f64".to_string()),
        Expression::String(_) => Some("StaticString".to_string()),
        Expression::Boolean(_) => Some("bool".to_string()),

        Expression::StructLiteral { name, .. } => Some(name.clone()),

        Expression::FunctionCall { name, .. } => {
            // Look up function return type from stdlib
            stdlib_types()
                .get_function_return_type("", name)
                .map(format_type)
        }

        Expression::MethodCall { object, method, .. } => {
            // Try to infer receiver type first
            if let Some(recv_type) = infer_type_from_ast_expr(object) {
                if let Some(ret) = stdlib_types().get_method_return_type(&recv_type, method) {
                    return Some(format_type(ret));
                }
            }
            None
        }

        // Handle Option wrapping
        Expression::Identifier(name) if name == "None" => Some(wk.option_name().to_string()),

        _ => None,
    }
}
