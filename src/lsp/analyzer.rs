//! Document analysis - type checking, allocator validation, pattern checking
//! Extracted from document_store.rs

use super::pattern_checking::{check_pattern_exhaustiveness, find_missing_variants};
use super::types::SymbolInfo;
use super::utils::{compile_error_to_diagnostic, compile_error_to_diagnostic_with_content};
use crate::ast::{Declaration, Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::module_system::ModuleSystem;
use crate::name_utils;
use crate::parser::Parser;
use crate::stdlib_types::stdlib_types;
use crate::type_context::TypeContext;
use crate::typechecker::TypeChecker;
use lsp_types::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Analyze document content and return diagnostics
pub fn analyze_document(
    content: &str,
    skip_expensive_analysis: bool,
    documents: &HashMap<Url, super::types::Document>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            if !skip_expensive_analysis {
                diagnostics.extend(run_compiler_analysis(&program, content));

                for decl in &program.declarations {
                    if let Declaration::Function(func) = decl {
                        check_allocator_usage(&func.body, &mut diagnostics, content);
                        check_pattern_exhaustiveness_wrapper(
                            &func.body,
                            &mut diagnostics,
                            content,
                            documents,
                            workspace_symbols,
                            stdlib_symbols,
                        );
                    }
                }
            }
        }
        Err(err) => {
            diagnostics.push(compile_error_to_diagnostic(err));
        }
    }

    diagnostics
}

/// Run type checker analysis on parsed program
fn run_compiler_analysis(program: &Program, content: &str) -> Vec<Diagnostic> {
    let (diagnostics, _) = run_compiler_analysis_with_context(program, content);
    diagnostics
}

/// Run type checker analysis and return both diagnostics and TypeContext.
/// This is the semantic analysis entry point for intelligent LSP features.
pub fn run_compiler_analysis_with_context(
    program: &Program,
    content: &str,
) -> (Vec<Diagnostic>, Option<Arc<TypeContext>>) {
    let mut diagnostics = Vec::new();

    // Load imported modules using the module system
    let (merged_program, module_system) = load_imports_for_program(program);

    let mut type_checker = TypeChecker::new();
    // Critical: Pass loaded stdlib modules to TypeChecker so it can extract type info
    // This matches what run_pipeline does in the CLI compiler path
    type_checker.with_stdlib_modules(module_system.get_modules());

    let (type_context, first_error) = type_checker.check_program_tolerant(&merged_program);
    if let Some(err) = first_error {
        diagnostics.push(compile_error_to_diagnostic_with_content(err, Some(content)));
    }
    (diagnostics, Some(Arc::new(type_context)))
}

/// Analyze document with full semantic analysis, returning TypeContext.
/// Used for background analysis to populate Document.type_context.
pub fn analyze_document_with_context(
    content: &str,
    documents: &HashMap<Url, super::types::Document>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
) -> (Vec<Diagnostic>, Option<Arc<TypeContext>>) {
    let mut diagnostics = Vec::new();

    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            let (type_diags, type_context) = run_compiler_analysis_with_context(&program, content);
            diagnostics.extend(type_diags);

            for decl in &program.declarations {
                if let Declaration::Function(func) = decl {
                    check_allocator_usage(&func.body, &mut diagnostics, content);
                    check_pattern_exhaustiveness_wrapper(
                        &func.body,
                        &mut diagnostics,
                        content,
                        documents,
                        workspace_symbols,
                        stdlib_symbols,
                    );
                }
            }

            (diagnostics, type_context)
        }
        Err(err) => {
            diagnostics.push(compile_error_to_diagnostic(err));
            (diagnostics, None)
        }
    }
}

/// Load imported modules and merge them with the main program
/// Returns both the merged program and the module system for type extraction
fn load_imports_for_program(program: &Program) -> (Program, ModuleSystem) {
    let mut module_system = ModuleSystem::new();

    // Load all imported modules
    for decl in &program.declarations {
        if let Declaration::ModuleImport { module_path, .. } = decl {
            // Try to load the module - ignore errors for LSP analysis
            // (we don't want to fail on missing modules, just show what we can)
            let _ = module_system.load_module(module_path);
        }
    }

    // Merge all loaded modules with the main program
    let merged = module_system.merge_programs(program.clone());

    (merged, module_system)
}

/// Check for allocator usage in statements
pub fn check_allocator_usage(
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
) {
    for stmt in statements {
        match stmt {
            Statement::Expression { expr, .. } | Statement::Return { expr, .. } => {
                check_allocator_in_expression(expr, diagnostics, content);
            }
            Statement::VariableDeclaration {
                initializer: Some(expr),
                ..
            }
            | Statement::VariableAssignment { value: expr, .. } => {
                check_allocator_in_expression(expr, diagnostics, content);
            }
            _ => {}
        }
    }
}

fn check_allocator_in_expression(
    expr: &Expression,
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
) {
    match expr {
        Expression::FunctionCall { name, args, .. } => {
            // Check if this type requires an allocator based on its struct definition
            let base_name = name_utils::strip_generics(name);

            // Use stdlib registry to check if the type has an allocator field
            let requires_alloc = stdlib_types().requires_allocator(base_name);

            if requires_alloc && (args.is_empty() || !has_allocator_arg(args)) {
                if let Some(position) = find_text_position(name, content) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: position,
                            end: Position {
                                line: position.line,
                                character: position.character + name.len() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("allocator-required".to_string())),
                        code_description: None,
                        source: Some("zen-lsp".to_string()),
                        message: format!(
                            "{} requires an allocator for memory management. Add get_default_allocator() as the last parameter.",
                            base_name
                        ),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            for arg in args {
                check_allocator_in_expression(arg, diagnostics, content);
            }
        }
        Expression::MethodCall {
            object,
            method: _,
            args,
            ..
        } => {
            check_allocator_in_expression(object, diagnostics, content);
            for arg in args {
                check_allocator_in_expression(arg, diagnostics, content);
            }
        }
        Expression::Block(stmts) => {
            check_allocator_usage(stmts, diagnostics, content);
        }
        Expression::Conditional { scrutinee, arms } => {
            check_allocator_in_expression(scrutinee, diagnostics, content);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    check_allocator_in_expression(guard, diagnostics, content);
                }
                check_allocator_in_expression(&arm.body, diagnostics, content);
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            check_allocator_in_expression(left, diagnostics, content);
            check_allocator_in_expression(right, diagnostics, content);
        }
        _ => {}
    }
}

fn has_allocator_arg(args: &[Expression]) -> bool {
    for arg in args {
        match arg {
            Expression::FunctionCall { name, .. } => {
                if name.contains("allocator") || name == "get_default_allocator" {
                    return true;
                }
            }
            Expression::Identifier(name) => {
                if name.contains("alloc") || name.ends_with("_allocator") || name == "allocator" {
                    return true;
                }
            }
            Expression::MethodCall { object, method, .. } => {
                if method.contains("allocator") || method == "get_allocator" {
                    return true;
                }
                if has_allocator_arg(&[(**object).clone()]) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Wrapper for pattern exhaustiveness checking
fn check_pattern_exhaustiveness_wrapper(
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    documents: &HashMap<Url, super::types::Document>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
) {
    check_pattern_exhaustiveness(
        statements,
        diagnostics,
        content,
        |expr| infer_expression_type_string(expr, documents),
        find_pattern_match_position,
        |scrutinee_type, arms| {
            find_missing_variants(
                scrutinee_type,
                arms,
                documents,
                workspace_symbols,
                stdlib_symbols,
            )
        },
    );
}

pub fn infer_expression_type_string(
    expr: &Expression,
    documents: &HashMap<Url, super::types::Document>,
) -> Option<String> {
    use crate::lsp::type_query::TypeQuery;

    for doc in documents
        .values()
        .take(crate::lsp::search_limits::QUICK_TYPE_SEARCH)
    {
        let tq = TypeQuery::new(doc);

        match expr {
            Expression::Identifier(name) => {
                if let Some(type_str) = tq.find_variable_type(name) {
                    return Some(type_str);
                }
            }
            Expression::FunctionCall { name, .. } => {
                if let Some(ret) = tq.function_return_type(name) {
                    return Some(ret);
                }
            }
            _ => {}
        }
    }

    if let Some(type_str) = crate::lsp::type_query::TypeQuery::infer_literal_type(expr) {
        return Some(type_str);
    }

    None
}

fn find_pattern_match_position(content: &str, scrutinee: &Expression) -> Option<Position> {
    if let Expression::Identifier(name) = scrutinee {
        return find_text_position(name, content);
    }
    None
}

pub fn find_text_position(text: &str, content: &str) -> Option<Position> {
    let lines: Vec<&str> = content.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(col) = line.find(text) {
            return Some(Position {
                line: line_num as u32,
                character: col as u32,
            });
        }
    }
    None
}
