// Type Inference Utilities for LSP
// Extracted from ZenLanguageServer for better code organization

use super::document_store::DocumentStore;
use super::types::Document;
use super::utils::format_type;
use crate::name_utils;
use crate::stdlib_types::stdlib_types;
use crate::well_known::well_known;
use lsp_types::*;
use std::collections::HashMap;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extract the base type name without generic parameters
/// Uses the parser for accurate extraction
pub fn get_base_type_name(type_str: &str) -> String {
    let (base, _) = parse_generic_type(type_str);
    base
}

/// Split generic arguments by comma, respecting nested <> brackets
fn split_generic_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in args.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                if !current.trim().is_empty() {
                    result.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// Infer type from expression string
/// First tries to parse as an AST expression, then falls back to heuristics
fn infer_type_from_prefix(expr: &str) -> Option<String> {
    let trimmed = expr.trim();

    // Try parsing the expression using the real parser
    if let Some(type_name) = try_infer_from_parsed_expr(trimmed) {
        return Some(type_name);
    }

    // Fallback: simple literal detection for cases parser might not handle
    let wk = well_known();

    // String literals
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        return Some("String".to_string());
    }

    // None variant
    if trimmed == wk.none_name() {
        return Some(wk.option_name().to_string());
    }

    // Array literal [size; type]
    if trimmed.starts_with('[') && trimmed.contains(';') && trimmed.ends_with(']') {
        return Some("Array".to_string());
    }

    None
}

/// Try to parse an expression and infer its type from the AST
fn try_infer_from_parsed_expr(expr: &str) -> Option<String> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    let lexer = Lexer::new(expr);
    let mut parser = Parser::new(lexer);
    let parsed = parser.parse_expression().ok()?;

    infer_type_from_ast_expr(&parsed)
}

/// Infer type from a parsed AST Expression
fn infer_type_from_ast_expr(expr: &crate::ast::Expression) -> Option<String> {
    use crate::ast::Expression;

    let wk = well_known();

    match expr {
        // Integer literals
        Expression::Integer8(_)
        | Expression::Integer16(_)
        | Expression::Integer32(_)
        | Expression::Integer64(_) => Some("i32".to_string()),

        // Unsigned integer literals
        Expression::Unsigned8(_)
        | Expression::Unsigned16(_)
        | Expression::Unsigned32(_)
        | Expression::Unsigned64(_) => Some("u64".to_string()),

        // Float literals
        Expression::Float32(_) | Expression::Float64(_) => Some("f64".to_string()),

        // Boolean
        Expression::Boolean(_) => Some("bool".to_string()),

        // String literal
        Expression::String(_) => Some("String".to_string()),

        // Constructor calls - the function name IS the type
        Expression::FunctionCall { name, .. } => {
            // Check if this is a known type constructor
            let base_name = get_base_type_name(name);
            if is_type_constructor(&base_name) {
                Some(base_name)
            } else {
                None
            }
        }

        // Struct literal - the name IS the type
        Expression::StructLiteral { name, .. } => Some(name.clone()),

        // Enum variant construction
        Expression::EnumVariant {
            enum_name, variant, ..
        } => {
            // Some(...) -> Option, Ok(...)/Err(...) -> Result
            if wk.is_option(enum_name) || wk.is_option_variant(variant) {
                Some(wk.option_name().to_string())
            } else if wk.is_result(enum_name) || wk.is_result_variant(variant) {
                Some(wk.result_name().to_string())
            } else {
                Some(enum_name.clone())
            }
        }

        // Enum literal (shorthand like .Some(x))
        Expression::EnumLiteral { variant, .. } => {
            if wk.is_option_variant(variant) {
                Some(wk.option_name().to_string())
            } else if wk.is_result_variant(variant) {
                Some(wk.result_name().to_string())
            } else {
                None // Can't determine enum type from shorthand alone
            }
        }

        // Array literal
        Expression::ArrayLiteral(_) => Some("Array".to_string()),

        _ => None,
    }
}

/// Check if a name is a known type constructor
/// Uses stdlib_types and well_known instead of hardcoded list
fn is_type_constructor(name: &str) -> bool {
    use crate::stdlib_types::stdlib_types;

    let wk = well_known();

    // Check well-known types (Option, Result, Ptr variants)
    if wk.is_option(name)
        || wk.is_result(name)
        || wk.is_ptr(name)
        || wk.is_mutable_ptr(name)
        || wk.is_raw_ptr(name)
    {
        return true;
    }

    // Check enum variant names using centralized WellKnownTypes
    if wk.is_option_variant(name) || wk.is_result_variant(name) {
        return true;
    }

    // Check stdlib types (Vec, HashMap, String, etc.)
    if stdlib_types().get_struct_definition(name).is_some() {
        return true;
    }

    false
}

// ============================================================================
// MAIN FUNCTIONS
// ============================================================================

pub fn parse_generic_type(type_str: &str) -> (String, Vec<String>) {
    let (name, type_args) = crate::parser::parse_generic_type_string(type_str);
    let args = type_args.iter().map(super::utils::format_type).collect();
    (name, args)
}

pub fn infer_receiver_type(receiver: &str, documents: &HashMap<Url, Document>) -> Option<String> {
    // Check common patterns first using helper
    if let Some(t) = infer_type_from_prefix(receiver) {
        return Some(t);
    }

    // Check for numeric literals
    let trimmed = receiver.trim();
    if trimmed
        .chars()
        .all(|c| c.is_numeric() || c == '.' || c == '-')
        && !trimmed.is_empty()
    {
        return Some(
            if trimmed.contains('.') {
                "f64"
            } else {
                "i32"
            }
            .to_string(),
        );
    }

    // Check if receiver is a known variable in symbols (limit search)
    for doc in documents
        .values()
        .take(crate::lsp::search_limits::TYPE_INFERENCE_SEARCH)
    {
        if let Some(symbol) = doc.symbols.get(receiver) {
            if let Some(type_info) = &symbol.type_info {
                return Some(format_type(type_info));
            }
            if let Some(detail) = &symbol.detail {
                if let Some(t) = extract_type_from_detail(detail) {
                    return Some(t);
                }
            }
        }
    }

    None
}

/// Extract type from symbol detail string
fn extract_type_from_detail(detail: &str) -> Option<String> {
    // Parse function return types: "name = (...) ReturnType"
    if detail.contains(" = ") && detail.contains(')') {
        if let Some(return_type) = detail.split(" = ").nth(1) {
            if let Some(ret) = return_type.split(')').nth(1).map(|s| s.trim()) {
                if !ret.is_empty() && ret != "void" {
                    let (base, _) = parse_generic_type(ret);
                    return Some(base);
                }
            }
        }
    }

    // Try type after ':'
    if let Some(colon_pos) = detail.find(':') {
        let after = detail[colon_pos + 1..].trim();
        let (base, _) = parse_generic_type(after);
        if !base.is_empty() {
            return Some(base);
        }
    }

    // Try type after ')'
    if let Some(close_paren) = detail.rfind(')') {
        let after = detail[close_paren + 1..].trim();
        if !after.is_empty() && after != "void" {
            let (base, _) = parse_generic_type(after);
            if !base.is_empty() {
                return Some(base);
            }
        }
    }

    // Fallback: check for known type names
    let wk = well_known();
    let known_types = [
        "HashMap",
        "DynVec",
        "Vec",
        "Array",
        wk.option_name(),
        wk.result_name(),
        "String",
        "StaticString",
    ];
    for type_name in known_types {
        if detail.contains(type_name) {
            return Some(type_name.to_string());
        }
    }

    None
}

pub fn infer_chained_method_type(
    receiver: &str,
    documents: &HashMap<Url, Document>,
) -> Option<String> {
    // Parse method chains from left to right, tracking type through each call
    let mut current_type: Option<String> = None;
    let parts = split_method_chain(receiver);

    // Process each part of the chain
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // First part - infer its base type
            current_type = infer_base_expression_type(part, documents);
        } else if let Some(ref curr_type) = current_type {
            // Subsequent parts are method calls on the current type
            if let Some(method_name) = part.split('(').next() {
                current_type = stdlib_types()
                    .get_method_return_type(curr_type, method_name)
                    .map(format_type);
            }
        }
    }

    current_type
}

pub fn infer_base_expression_type(
    expr: &str,
    documents: &HashMap<Url, Document>,
) -> Option<String> {
    let expr = expr.trim();

    // Check common patterns first using helper
    if let Some(t) = infer_type_from_prefix(expr) {
        return Some(t);
    }

    // Numeric literals
    if expr.parse::<i64>().is_ok() {
        return Some("i32".to_string());
    }
    if expr.parse::<f64>().is_ok() {
        return Some("f64".to_string());
    }

    // Check if it's a known variable
    if let Some(type_name) = expr.split('(').next() {
        for doc in documents
            .values()
            .take(crate::lsp::search_limits::TYPE_INFERENCE_SEARCH)
        {
            if let Some(symbol) = doc.symbols.get(type_name) {
                if let Some(type_info) = &symbol.type_info {
                    let type_str = format_type(type_info);
                    return Some(name_utils::strip_generics(&type_str).to_string());
                }
            }
        }
    }

    None
}

pub fn get_method_return_type(
    receiver_type: &str,
    method_name: &str,
    store: &DocumentStore,
) -> Option<String> {
    // Prefer compiler integration data
    if let Some(ast_type) = store
        .compiler
        .get_method_return_type(receiver_type, method_name)
    {
        return Some(format_type(&ast_type));
    }

    // Fallback: use symbol tables if compiler doesn't know this method yet
    if let Some(symbol) = store
        .stdlib_symbols
        .get(method_name)
        .or_else(|| store.workspace_symbols.get(method_name))
    {
        if let Some(type_info) = &symbol.type_info {
            return Some(format_type(type_info));
        }

        if let Some(detail) = &symbol.detail {
            if let Some(return_type) = extract_return_type_from_detail(detail) {
                return Some(return_type);
            }
        }
    }

    // Final fallback: check stdlib_types registry
    if let Some(ret) = stdlib_types().get_method_return_type(receiver_type, method_name) {
        return Some(format_type(ret));
    }

    None
}

fn extract_return_type_from_detail(detail: &str) -> Option<String> {
    if let Some(close_paren) = detail.rfind(')') {
        let after = detail[close_paren + 1..].trim();
        if !after.is_empty() && after != "void" {
            // Strip trailing block braces if present (e.g., "String {" -> "String")
            let cleaned = after.trim_end_matches('{').trim();
            if !cleaned.is_empty() {
                return Some(cleaned.to_string());
            }
        }
    }
    None
}

fn split_method_chain(receiver: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth: usize = 0;
    let mut in_string = false;

    for ch in receiver.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(ch);
            }
            '.' if !in_string && paren_depth == 0 => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

// Wrapper functions that work with DocumentStore

pub fn find_self_type_in_enclosing_function(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let current_line = position.line as usize;

    for line_idx in (0..=current_line).rev() {
        let line = lines.get(line_idx)?;

        if let Some(self_pos) = line.find("self:") {
            let after_self = &line[self_pos + 5..];
            let type_end = after_self.find([',', ')']).unwrap_or(after_self.len());
            let self_type = after_self[..type_end].trim();
            if !self_type.is_empty() {
                return Some(self_type.to_string());
            }
        }
    }
    None
}

pub fn infer_receiver_type_with_context(
    receiver: &str,
    content: Option<&str>,
    position: Option<Position>,
    store: &DocumentStore,
) -> Option<String> {
    if receiver == "self" {
        if let (Some(content), Some(pos)) = (content, position) {
            if let Some(self_type) = find_self_type_in_enclosing_function(content, pos) {
                return Some(self_type);
            }
        }
    }

    if receiver.contains('.') && receiver.contains('(') {
        if let Some(ty) = infer_chained_method_type_from_store(receiver, store) {
            return Some(ty);
        }
    }

    infer_receiver_type(receiver, &store.documents)
}

pub fn infer_receiver_type_from_store(receiver: &str, store: &DocumentStore) -> Option<String> {
    infer_receiver_type_with_context(receiver, None, None, store)
}

pub fn infer_chained_method_type_from_store(
    receiver: &str,
    store: &DocumentStore,
) -> Option<String> {
    let mut current_type: Option<String> = None;
    let parts = split_method_chain(receiver);

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            current_type = infer_base_expression_type_from_store(part, store);
        } else if let Some(ref curr_type) = current_type {
            if let Some(method_name) = part.split('(').next() {
                let method_name = method_name.trim();
                current_type = get_method_return_type(curr_type, method_name, store);
            }
        }
    }

    current_type
}

pub fn infer_base_expression_type_from_store(expr: &str, store: &DocumentStore) -> Option<String> {
    infer_base_expression_type(expr, &store.documents)
}

// Additional parsing and extraction functions
use crate::ast::AstType;

pub fn extract_generic_types(ast_type: &AstType) -> (Option<String>, Option<String>) {
    let wk = well_known();
    match ast_type {
        // Result<T, E>
        AstType::Generic { name, type_args } if wk.is_result(name) && type_args.len() == 2 => {
            let ok_type = format_type(&type_args[0]);
            let err_type = format_type(&type_args[1]);
            (Some(ok_type), Some(err_type))
        }
        // Option<T>
        AstType::Generic { name, type_args } if wk.is_option(name) && type_args.len() == 1 => {
            let inner_type = format_type(&type_args[0]);
            (Some(inner_type), None)
        }
        _ => (None, None),
    }
}

pub fn parse_function_from_source(
    content: &str,
    func_name: &str,
) -> (Option<String>, Option<String>) {
    // Find the function definition line
    // Example: "divide = (a: f64, b: f64) Result<f64, StaticString> {"

    for line in content.lines() {
        if line.contains(&format!("{} =", func_name)) || line.contains(&format!("{}=", func_name)) {
            // Found the function definition
            if let Some(result_pos) = line.find("Result<") {
                // Extract Result<T, E> by finding the matching >
                let after_result = &line[result_pos..];
                if let Some(start) = after_result.find('<') {
                    // Find the matching closing >
                    let mut depth = 0;
                    let mut end_pos = start;
                    for (i, ch) in after_result[start..].chars().enumerate() {
                        match ch {
                            '<' => depth += 1,
                            '>' => {
                                depth -= 1;
                                if depth == 0 {
                                    end_pos = start + i;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if end_pos > start {
                        let generics = &after_result[start + 1..end_pos];
                        let parts = split_generic_args(generics);
                        if parts.len() >= 2 {
                            return (
                                Some(parts[0].trim().to_string()),
                                Some(parts[1].trim().to_string()),
                            );
                        } else if parts.len() == 1 {
                            // Only one part found - maybe parsing error
                            return (Some(parts[0].trim().to_string()), Some("E".to_string()));
                        }
                    }
                }
            } else if let Some(option_pos) = line.find("Option<") {
                // Extract Option<T>
                if let Some(start) = line[option_pos..].find('<') {
                    if let Some(end) = line[option_pos..].rfind('>') {
                        let inner = line[option_pos + start + 1..option_pos + end].trim();
                        return (Some(inner.to_string()), None);
                    }
                }
            }
        }
    }

    (None, None)
}

pub fn infer_function_return_types(
    func_name: &str,
    all_docs: &HashMap<Url, Document>,
) -> (Option<String>, Option<String>) {
    use crate::ast::Declaration;

    // Try AST first (most accurate)
    for doc in all_docs.values() {
        if let Some(ast) = &doc.ast {
            for decl in ast {
                if let Declaration::Function(func) = decl {
                    if func.name == func_name {
                        // Found it! Extract types from the return type
                        let result = extract_generic_types(&func.return_type);
                        if result.0.is_some() {
                            return result;
                        }
                    }
                }
            }
        }
    }

    // Fallback: parse from source code if AST unavailable (e.g., parse errors)
    for doc in all_docs.values() {
        let result = parse_function_from_source(&doc.content, func_name);
        if result.0.is_some() && result.1.is_some() {
            return result;
        }
    }

    (None, None)
}
