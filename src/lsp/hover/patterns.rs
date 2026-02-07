// Pattern matching hover functionality

use lsp_types::Position;
use std::collections::HashMap;

use crate::ast::AstType;
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
                if let crate::ast::Declaration::Function(func) = decl {
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
    let mut scrutinee_line = None;
    for i in (0..=position.line).rev() {
        let line = lines[i as usize].trim();
        // Use lexer-based pattern match detection
        if let Some(q_pos) = find_pattern_match_question(line) {
            // Found the pattern match - extract variable name
            let before_q = line[..q_pos].trim();
            // Get the last word before '?'
            if let Some(var) = before_q.split_whitespace().last() {
                scrutinee_name = Some(var.to_string());
                scrutinee_line = Some(i);
                break;
            }
        }
        // Don't search too far back
        if position.line - i > 10 {
            break;
        }
    }

    if let Some(scrutinee) = scrutinee_name {
        // Try to infer the type of the scrutinee by looking at its definition
        if let Some(scrutinee_line_num) = scrutinee_line {
            // Look backwards from scrutinee for its definition
            for i in (0..scrutinee_line_num).rev() {
                let line = lines[i as usize];
                // Check if this line defines the scrutinee
                if line.contains(&format!("{} =", scrutinee)) {
                    // Try to infer type from the assignment
                    // Example: result = divide(10.0, 2.0)
                    if let Some(eq_pos) = line.find('=') {
                        let rhs = line[eq_pos + 1..].trim();

                        // Check if it's a function call
                        if let Some(paren_pos) = rhs.find('(') {
                            let func_name = rhs[..paren_pos].trim();

                            // Use SEMA to find the function's return type generics
                            let (concrete_ok_type, concrete_err_type) =
                                infer_function_return_types_via_sema(func_name, all_docs);

                            // Now determine what pattern variable we're hovering over
                            // Example: | Ok(val) or | Err(msg)
                            let pattern_arm = current_line.trim();

                            if pattern_arm.contains(&format!("Ok({}", symbol_name))
                                || pattern_arm.contains(&format!("Ok({})", symbol_name))
                            {
                                // This is the Ok variant - extract the success type
                                let type_display =
                                    concrete_ok_type.clone().unwrap_or_else(|| "T".to_string());
                                let full_result_type = if let (Some(ok), Some(err)) =
                                    (&concrete_ok_type, &concrete_err_type)
                                {
                                    format!("Result<{}, {}>", ok, err)
                                } else {
                                    "Result<T, E>".to_string()
                                };

                                return Some(format!(
                                    "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `{}` (assigned from `{}()`)\n\nThis is the success value from the `Ok` variant.",
                                    symbol_name,
                                    type_display,
                                    full_result_type,
                                    func_name
                                ));
                            } else if pattern_arm.contains(&format!("Err({}", symbol_name))
                                || pattern_arm.contains(&format!("Err({})", symbol_name))
                            {
                                // This is the Err variant - extract the error type
                                let type_display =
                                    concrete_err_type.clone().unwrap_or_else(|| "E".to_string());
                                let full_result_type = if let (Some(ok), Some(err)) =
                                    (&concrete_ok_type, &concrete_err_type)
                                {
                                    format!("Result<{}, {}>", ok, err)
                                } else {
                                    "Result<T, E>".to_string()
                                };

                                return Some(format!(
                                    "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `{}` (assigned from `{}()`)\n\nThis is the error value from the `Err` variant.",
                                    symbol_name,
                                    type_display,
                                    full_result_type,
                                    func_name
                                ));
                            } else if pattern_arm.contains(&format!("Some({}", symbol_name))
                                || pattern_arm.contains(&format!("Some({})", symbol_name))
                            {
                                // This is Option.Some
                                let inner_type =
                                    concrete_ok_type.clone().unwrap_or_else(|| "T".to_string());
                                return Some(format!(
                                    "```zen\n{}: {}\n```\n\n**Pattern match variable**\n\nExtracted from `Option<{}>` (assigned from `{}()`)\n\nThis is the value from the `Some` variant.",
                                    symbol_name,
                                    inner_type,
                                    inner_type,
                                    func_name
                                ));
                            }
                        }
                    }
                }
                // Don't search too far back
                if scrutinee_line_num - i > 20 {
                    break;
                }
            }
        }
    }

    None
}

/// Get hover information for enum variants
pub fn get_enum_variant_hover(
    content: &str,
    position: Position,
    symbol_name: &str,
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

    // Find the enum name by looking backwards
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
