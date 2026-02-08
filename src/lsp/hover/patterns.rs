// Pattern matching hover functionality

use lsp_types::Position;
use std::collections::HashMap;

use crate::ast::{AstType, Declaration, Expression, Statement};
use crate::lsp::type_query::TypeQuery;
use crate::lsp::types::*;
use crate::lsp::utils::{find_pattern_match_question, format_type, is_pattern_arm_line};
use crate::name_utils;
use crate::well_known::well_known;

/// Extract the first two generic type arguments from a Result<T, E> or Option<T> AstType.
/// Returns (ok_or_inner, err) where err is None for Option.
fn extract_generic_pair(ast_type: &AstType) -> (Option<String>, Option<String>) {
    let wk = well_known();
    match ast_type {
        AstType::Generic { name, type_args } if wk.is_result(name) && type_args.len() == 2 => (
            Some(format_type(&type_args[0])),
            Some(format_type(&type_args[1])),
        ),
        AstType::Generic { name, type_args } if wk.is_option(name) && type_args.len() == 1 => {
            (Some(format_type(&type_args[0])), None)
        }
        _ => (None, None),
    }
}

/// Look up a function's return type across all documents via TypeQuery/AST,
/// and extract the generic type pair (for Result/Option).
fn infer_function_return_types_via_sema(
    func_name: &str,
    all_docs: &HashMap<lsp_types::Url, Document>,
) -> (Option<String>, Option<String>) {
    // Search SEMA (TypeContext) in every document
    for doc in all_docs.values() {
        let tq = TypeQuery::new(doc);
        if let Some(ret_ast) = tq.function_return_type_ast(func_name) {
            let pair = extract_generic_pair(&ret_ast);
            if pair.0.is_some() {
                return pair;
            }
        }
    }

    // Fallback: search AST directly (for documents where SEMA failed)
    for doc in all_docs.values() {
        if let Some(ast) = &doc.ast {
            for decl in ast {
                if let Declaration::Function(func) = decl {
                    if func.name == func_name {
                        let pair = extract_generic_pair(&func.return_type);
                        if pair.0.is_some() {
                            return pair;
                        }
                    }
                }
            }
        }
    }

    (None, None)
}

/// Try to find the function name from a variable's initializer expression in the AST.
/// Returns the function name if the variable is initialized with a function call.
fn find_scrutinee_function_call_in_ast(
    ast: &[Declaration],
    scrutinee_name: &str,
) -> Option<String> {
    for decl in ast {
        if let Declaration::Function(func) = decl {
            if let Some(name) = find_function_call_in_statements(&func.body, scrutinee_name) {
                return Some(name);
            }
        }
    }
    None
}

/// Search statements for a variable declaration whose initializer is a function call.
fn find_function_call_in_statements(stmts: &[Statement], var_name: &str) -> Option<String> {
    for stmt in stmts {
        if let Statement::VariableDeclaration {
            name,
            initializer: Some(expr),
            ..
        } = stmt
        {
            if name == var_name {
                return extract_function_name_from_expr(expr);
            }
        }
    }
    None
}

/// Extract the function name from a function call expression.
fn extract_function_name_from_expr(expr: &Expression) -> Option<String> {
    match expr {
        Expression::FunctionCall { name, .. } => Some(name.clone()),
        _ => None,
    }
}

pub fn get_pattern_match_hover(
    content: &str,
    position: Position,
    symbol_name: &str,
    _local_symbols: &HashMap<String, SymbolInfo>,
    _stdlib_symbols: &HashMap<String, SymbolInfo>,
    all_docs: &HashMap<lsp_types::Url, crate::lsp::types::Document>,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[position.line as usize];

    // Check if we're in a pattern match arm using lexer-based detection
    if !is_pattern_arm_line(current_line) {
        return None;
    }

    // Find the scrutinee by looking backwards for 'variable ?' using lexer-based detection
    let mut scrutinee_name = None;
    for i in (0..=position.line).rev() {
        let line = lines[i as usize].trim();
        // Use lexer-based pattern match detection
        if let Some(q_pos) = find_pattern_match_question(line) {
            // Found the pattern match - extract variable name
            let before_q = line[..q_pos].trim();
            // Get the last word before '?'
            if let Some(var) = before_q.split_whitespace().last() {
                scrutinee_name = Some(var.to_string());
                break;
            }
        }
        // Don't search too far back
        if position.line - i > 10 {
            break;
        }
    }

    if let Some(scrutinee) = scrutinee_name {
        // Try to find the function call via AST first (structured approach)
        let mut func_name_from_ast = None;
        for doc in all_docs.values() {
            if let Some(ast) = &doc.ast {
                if let Some(name) = find_scrutinee_function_call_in_ast(ast, &scrutinee) {
                    func_name_from_ast = Some(name);
                    break;
                }
            }
        }

        if let Some(func_name) = &func_name_from_ast {
            // Use SEMA to find the function's return type generics
            let (concrete_ok_type, concrete_err_type) =
                infer_function_return_types_via_sema(func_name, all_docs);

            if let Some(hover) = build_pattern_hover(
                symbol_name,
                current_line,
                func_name,
                &concrete_ok_type,
                &concrete_err_type,
            ) {
                return Some(hover);
            }
        }

        // Fallback: scan source text backwards for the assignment
        if let Some(scrutinee_line) = find_scrutinee_line(lines.as_slice(), position.line) {
            for i in (0..scrutinee_line).rev() {
                let line = lines[i as usize];
                if line.contains(&format!("{} =", scrutinee)) {
                    if let Some(eq_pos) = line.find('=') {
                        let rhs = line[eq_pos + 1..].trim();
                        if let Some(paren_pos) = rhs.find('(') {
                            let func_name = rhs[..paren_pos].trim();
                            let (concrete_ok_type, concrete_err_type) =
                                infer_function_return_types_via_sema(func_name, all_docs);

                            if let Some(hover) = build_pattern_hover(
                                symbol_name,
                                current_line,
                                func_name,
                                &concrete_ok_type,
                                &concrete_err_type,
                            ) {
                                return Some(hover);
                            }
                        }
                    }
                }
                if scrutinee_line - i > 20 {
                    break;
                }
            }
        }
    }

    None
}

/// Find the scrutinee line number by looking backwards for the pattern match question
fn find_scrutinee_line(lines: &[&str], current_line: u32) -> Option<u32> {
    for i in (0..=current_line).rev() {
        let line = lines[i as usize].trim();
        if find_pattern_match_question(line).is_some() {
            return Some(i);
        }
        if current_line - i > 10 {
            break;
        }
    }
    None
}

/// Build the hover text for a pattern match variable
fn build_pattern_hover(
    symbol_name: &str,
    current_line: &str,
    func_name: &str,
    concrete_ok_type: &Option<String>,
    concrete_err_type: &Option<String>,
) -> Option<String> {
    let pattern_arm = current_line.trim();

    if pattern_arm.contains(&format!("Ok({}", symbol_name))
        || pattern_arm.contains(&format!("Ok({})", symbol_name))
    {
        let type_display = concrete_ok_type.clone().unwrap_or_else(|| "T".to_string());
        let full_result_type = if let (Some(ok), Some(err)) = (concrete_ok_type, concrete_err_type)
        {
            format!("Result<{}, {}>", ok, err)
        } else {
            "Result<T, E>".to_string()
        };

        return Some(format!(
            "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `{}` (assigned from `{}()`)\n\nThis is the success value from the `Ok` variant.",
            symbol_name, type_display, full_result_type, func_name
        ));
    } else if pattern_arm.contains(&format!("Err({}", symbol_name))
        || pattern_arm.contains(&format!("Err({})", symbol_name))
    {
        let type_display = concrete_err_type.clone().unwrap_or_else(|| "E".to_string());
        let full_result_type = if let (Some(ok), Some(err)) = (concrete_ok_type, concrete_err_type)
        {
            format!("Result<{}, {}>", ok, err)
        } else {
            "Result<T, E>".to_string()
        };

        return Some(format!(
            "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `{}` (assigned from `{}()`)\n\nThis is the error value from the `Err` variant.",
            symbol_name, type_display, full_result_type, func_name
        ));
    } else if pattern_arm.contains(&format!("Some({}", symbol_name))
        || pattern_arm.contains(&format!("Some({})", symbol_name))
    {
        let inner_type = concrete_ok_type.clone().unwrap_or_else(|| "T".to_string());
        return Some(format!(
            "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `Option<{}>` (assigned from `{}()`)\n\nThis is the value from the `Some` variant.",
            symbol_name, inner_type, inner_type, func_name
        ));
    }

    None
}

/// Get hover information for enum variants.
/// When AST is available, uses Declaration::Enum for structured detection.
pub fn get_enum_variant_hover(
    content: &str,
    position: Position,
    symbol_name: &str,
    ast: Option<&Vec<Declaration>>,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[position.line as usize];

    // Check if this line is an enum variant (has ':' or ',' after the symbol)
    // Enums use comma-separated variants: MyEnum: Variant1, Variant2: Type, Variant3
    if !current_line.contains(&format!("{}:", symbol_name))
        && !current_line.contains(&format!("{},", symbol_name))
    {
        return None;
    }

    // Try AST-based enum detection first
    if let Some(ast) = ast {
        for decl in ast.iter() {
            if let Declaration::Enum(enum_def) = decl {
                for variant in &enum_def.variants {
                    if variant.name == symbol_name {
                        let payload_info = match &variant.payload {
                            Some(ty) => format!(" of type `{}`", format_type(ty)),
                            None => String::new(),
                        };

                        let enum_base = name_utils::strip_generics(&enum_def.name);
                        return Some(format!(
                            "```zen\n{}\n    {}...\n```\n\n**Enum variant** `{}`{}\n\nPart of enum `{}`",
                            enum_def.name, symbol_name, symbol_name, payload_info, enum_base
                        ));
                    }
                }
            }
        }
    }

    // Fallback: scan source text backwards for the enum definition
    let mut enum_name = None;
    for i in (0..position.line).rev() {
        let line = lines[i as usize].trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        // Check if this line is an enum definition (identifier followed by ':' at end)
        if line.ends_with(':') && !line.contains("::") {
            let name = line.trim_end_matches(':').trim();
            if name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '<' || c == '>' || c == ',')
            {
                enum_name = Some(name.to_string());
                break;
            }
        }
        // If we hit something that's not a variant, stop
        if !line.contains(':') && !line.contains(',') {
            break;
        }
    }

    if let Some(enum_name) = enum_name {
        // Extract variant payload info from current line
        let payload_info = if current_line.contains('{') {
            // Struct-like payload
            let start = current_line.find('{')?;
            let end = current_line.rfind('}')?;
            let fields = &current_line[start + 1..end];
            format!(" with fields: `{}`", fields.trim())
        } else if current_line.contains(": ") {
            // Type payload
            let parts: Vec<&str> = current_line.split(':').collect();
            if parts.len() >= 2 {
                let type_part = parts[1].trim().trim_end_matches(',');
                format!(" of type `{}`", type_part)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Some(format!(
            "```zen\n{}\n    {}...\n```\n\n**Enum variant** `{}`{}\n\nPart of enum `{}`",
            enum_name,
            symbol_name,
            symbol_name,
            payload_info,
            name_utils::strip_generics(&enum_name)
        ))
    } else {
        None
    }
}
