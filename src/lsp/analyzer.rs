//! Document analysis - type checking, allocator validation, pattern checking
//! Extracted from document_store.rs

use super::pattern_checking;
use super::types::SymbolInfo;
use super::utils::{compile_error_to_diagnostic, compile_error_to_diagnostic_with_content};
use crate::ast::{Declaration, Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::module_system::ModuleSystem;
use crate::parser::Parser;
use crate::type_context::TypeContext;
use crate::typechecker::validation::check_allocator_violations;
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
    let loaded_modules = module_system.get_modules();
    type_checker.with_stdlib_modules(&loaded_modules);

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

pub fn check_allocator_usage(
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
) {
    for violation in check_allocator_violations(statements) {
        if let Some(position) = find_text_position(&violation.call_name, content) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: position,
                    end: Position {
                        line: position.line,
                        character: position.character + violation.call_name.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("allocator-required".to_string())),
                code_description: None,
                source: Some("zen-lsp".to_string()),
                message: format!(
                    "{} requires an allocator for memory management. Add get_default_allocator() as the last parameter.",
                    violation.type_name
                ),
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }
}

fn check_pattern_exhaustiveness_wrapper(
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    documents: &HashMap<Url, super::types::Document>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
) {
    let enum_registry =
        pattern_checking::build_enum_registry(documents, workspace_symbols, stdlib_symbols);
    pattern_checking::check_pattern_exhaustiveness(
        statements,
        diagnostics,
        content,
        &enum_registry,
        |expr| infer_expression_type_string(expr, documents),
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
