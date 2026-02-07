use super::types::{Document, SymbolInfo};
use crate::ast::Expression;
use crate::typechecker::validation;
use lsp_types::*;
use std::collections::HashMap;

pub fn build_enum_registry(
    documents: &HashMap<Url, Document>,
    workspace_symbols: &HashMap<String, SymbolInfo>,
    stdlib_symbols: &HashMap<String, SymbolInfo>,
) -> HashMap<String, Vec<String>> {
    let mut registry = HashMap::new();

    for doc in documents
        .values()
        .take(crate::lsp::search_limits::ENUM_SEARCH)
    {
        for (name, symbol) in &doc.symbols {
            if let Some(ref variants) = symbol.enum_variants {
                registry
                    .entry(name.clone())
                    .or_insert_with(|| variants.clone());
            }
        }
    }

    for (name, symbol) in workspace_symbols {
        if let Some(ref variants) = symbol.enum_variants {
            registry
                .entry(name.clone())
                .or_insert_with(|| variants.clone());
        }
    }

    for (name, symbol) in stdlib_symbols {
        if let Some(ref variants) = symbol.enum_variants {
            registry
                .entry(name.clone())
                .or_insert_with(|| variants.clone());
        }
    }

    registry
}

pub fn check_pattern_exhaustiveness(
    statements: &[crate::ast::Statement],
    diagnostics: &mut Vec<Diagnostic>,
    content: &str,
    enum_registry: &HashMap<String, Vec<String>>,
    infer_expression_type_string: impl Fn(&Expression) -> Option<String>,
) {
    let violations = validation::check_pattern_exhaustiveness(
        statements,
        enum_registry,
        &infer_expression_type_string,
    );

    for v in violations {
        if let Some(position) = find_pattern_match_position(content, &v.scrutinee) {
            let variant_list = v.missing_variants.join(", ");
            diagnostics.push(Diagnostic {
                range: Range {
                    start: position,
                    end: Position {
                        line: position.line,
                        character: position.character + 10,
                    },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("non-exhaustive-match".to_string())),
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

fn find_pattern_match_position(content: &str, scrutinee: &Expression) -> Option<Position> {
    if let Expression::Identifier(name) = scrutinee {
        return crate::lsp::analyzer::find_text_position(name, content);
    }
    None
}
