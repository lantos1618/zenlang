//! Pattern exhaustiveness checking - extracted from document_store.rs

use super::type_inference::get_base_type_name;
use super::types::{Document, SymbolInfo};
use crate::ast::{Expression, Pattern as AstPattern, PatternArm};
use crate::name_utils;
use crate::well_known::well_known;
use lsp_types::*;

/// Check pattern exhaustiveness in statements
pub fn check_pattern_exhaustiveness(
    statements: &[crate::ast::Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    infer_expression_type_string: impl Fn(&Expression) -> Option<String>,
    find_pattern_match_position: impl Fn(&str, &Expression) -> Option<Position>,
    find_missing_variants: impl Fn(&str, &[PatternArm]) -> Vec<String>,
) {
    check_pattern_exhaustiveness_with_depth(
        statements,
        diagnostics,
        content,
        &infer_expression_type_string,
        &find_pattern_match_position,
        &find_missing_variants,
        0,
    );
}

/// Internal function with recursion depth limit
fn check_pattern_exhaustiveness_with_depth(
    statements: &[crate::ast::Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    infer_expression_type_string: &impl Fn(&Expression) -> Option<String>,
    find_pattern_match_position: &impl Fn(&str, &Expression) -> Option<Position>,
    find_missing_variants: &impl Fn(&str, &[PatternArm]) -> Vec<String>,
    depth: usize,
) {
    // Prevent infinite recursion (reasonable limit for nested blocks)
    const MAX_RECURSION_DEPTH: usize = 50;
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    for stmt in statements {
        match stmt {
            crate::ast::Statement::Expression { expr, .. }
            | crate::ast::Statement::Return { expr, .. } => {
                check_exhaustiveness_in_expression(
                    expr,
                    diagnostics,
                    content,
                    infer_expression_type_string,
                    find_pattern_match_position,
                    find_missing_variants,
                    depth,
                );
            }
            crate::ast::Statement::VariableDeclaration {
                initializer: Some(expr),
                ..
            }
            | crate::ast::Statement::VariableAssignment { value: expr, .. } => {
                check_exhaustiveness_in_expression(
                    expr,
                    diagnostics,
                    content,
                    infer_expression_type_string,
                    find_pattern_match_position,
                    find_missing_variants,
                    depth,
                );
            }
            _ => {}
        }
    }
}

fn check_exhaustiveness_in_expression(
    expr: &Expression,
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    infer_expression_type_string: &impl Fn(&Expression) -> Option<String>,
    find_pattern_match_position: &impl Fn(&str, &Expression) -> Option<Position>,
    find_missing_variants: &impl Fn(&str, &[PatternArm]) -> Vec<String>,
    depth: usize,
) {
    match expr {
        Expression::PatternMatch { scrutinee, arms } => {
            if let Some(scrutinee_type) = infer_expression_type_string(scrutinee) {
                let missing_variants = find_missing_variants(&scrutinee_type, arms);

                if !missing_variants.is_empty() {
                    if let Some(position) = find_pattern_match_position(content, scrutinee) {
                        let variant_list = missing_variants.join(", ");
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: position,
                                end: Position {
                                    line: position.line,
                                    character: position.character + 10,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(lsp_types::NumberOrString::String(
                                "non-exhaustive-match".to_string(),
                            )),
                            source: Some("zen-lsp".to_string()),
                            message: format!(
                                "Non-exhaustive pattern match. Missing variants: {}",
                                variant_list
                            ),
                            related_information: None,
                            tags: None,
                            code_description: None,
                            data: None,
                        });
                    }
                }
            }

            check_exhaustiveness_in_expression(
                scrutinee,
                diagnostics,
                content,
                infer_expression_type_string,
                find_pattern_match_position,
                find_missing_variants,
                depth,
            );
            for arm in arms {
                check_exhaustiveness_in_expression(
                    &arm.body,
                    diagnostics,
                    content,
                    infer_expression_type_string,
                    find_pattern_match_position,
                    find_missing_variants,
                    depth,
                );
            }
        }
        Expression::Block(stmts) => {
            check_pattern_exhaustiveness_with_depth(
                stmts,
                diagnostics,
                content,
                infer_expression_type_string,
                find_pattern_match_position,
                find_missing_variants,
                depth + 1,
            );
        }
        Expression::Conditional { scrutinee, arms } => {
            check_exhaustiveness_in_expression(
                scrutinee,
                diagnostics,
                content,
                infer_expression_type_string,
                find_pattern_match_position,
                find_missing_variants,
                depth,
            );
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    check_exhaustiveness_in_expression(
                        guard,
                        diagnostics,
                        content,
                        infer_expression_type_string,
                        find_pattern_match_position,
                        find_missing_variants,
                        depth,
                    );
                }
                check_exhaustiveness_in_expression(
                    &arm.body,
                    diagnostics,
                    content,
                    infer_expression_type_string,
                    find_pattern_match_position,
                    find_missing_variants,
                    depth,
                );
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            check_exhaustiveness_in_expression(
                left,
                diagnostics,
                content,
                infer_expression_type_string,
                find_pattern_match_position,
                find_missing_variants,
                depth,
            );
            check_exhaustiveness_in_expression(
                right,
                diagnostics,
                content,
                infer_expression_type_string,
                find_pattern_match_position,
                find_missing_variants,
                depth,
            );
        }
        _ => {}
    }
}

/// Find missing enum variants in pattern match
pub fn find_missing_variants(
    scrutinee_type: &str,
    arms: &[PatternArm],
    documents: &std::collections::HashMap<Url, Document>,
    workspace_symbols: &std::collections::HashMap<String, SymbolInfo>,
    stdlib_symbols: &std::collections::HashMap<String, SymbolInfo>,
) -> Vec<String> {
    let wk = well_known();

    // First check if it's a built-in enum type
    let base_type = get_base_type_name(scrutinee_type);
    let known_enum_variants: Vec<String> = if wk.is_option(&base_type) {
        vec![wk.some_name().to_string(), wk.none_name().to_string()]
    } else if wk.is_result(&base_type) {
        vec![wk.ok_name().to_string(), wk.err_name().to_string()]
    } else {
        // Try to look up custom enum from symbol tables
        // Extract just the enum name (before any :: - get_base_type_name already removes generics)
        let enum_name = name_utils::base_name(&base_type).trim();

        // Search in all available symbol sources
        let mut found_variants: Option<Vec<String>> = None;

        // 1. Check current document symbols (limit search for performance)
        for doc in documents
            .values()
            .take(crate::lsp::search_limits::ENUM_SEARCH)
        {
            if let Some(symbol) = doc.symbols.get(enum_name) {
                if let Some(ref variants) = symbol.enum_variants {
                    found_variants = Some(variants.clone());
                    break;
                }
            }
        }

        // 2. Check workspace symbols if not found
        if found_variants.is_none() {
            if let Some(symbol) = workspace_symbols.get(enum_name) {
                if let Some(ref variants) = symbol.enum_variants {
                    found_variants = Some(variants.clone());
                }
            }
        }

        // 3. Check stdlib symbols if not found
        if found_variants.is_none() {
            if let Some(symbol) = stdlib_symbols.get(enum_name) {
                if let Some(ref variants) = symbol.enum_variants {
                    found_variants = Some(variants.clone());
                }
            }
        }

        // If we found the enum, use its variants; otherwise return empty
        match found_variants {
            Some(variants) => variants,
            None => return Vec::new(),
        }
    };

    // Collect covered variants and check for wildcards
    let mut covered_variants = std::collections::HashSet::new();
    let mut has_wildcard = false;

    for arm in arms {
        match &arm.pattern {
            AstPattern::EnumVariant { variant, .. } => {
                covered_variants.insert(variant.clone());
            }
            AstPattern::EnumLiteral { variant, .. } => {
                covered_variants.insert(variant.clone());
            }
            AstPattern::Wildcard => {
                has_wildcard = true;
            }
            _ => {}
        }
    }

    if has_wildcard {
        return Vec::new();
    }

    // Return missing variants
    known_enum_variants
        .into_iter()
        .filter(|v| !covered_variants.contains(v))
        .collect()
}
