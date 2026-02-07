// Variable symbol extraction from statements
use super::super::types::SymbolInfo;
use super::super::utils::format_type;
use super::utilities::{make_range, make_symbol};
use super::DocumentStore;
use crate::ast::{AstType, Expression, Statement};
use lsp_types::*;
use std::collections::HashMap;

impl DocumentStore {
    pub(super) fn extract_variables_from_statements(
        &self,
        statements: &[Statement],
        content: &str,
        symbols: &mut HashMap<String, SymbolInfo>,
    ) {
        for stmt in statements {
            match stmt {
                Statement::VariableDeclaration {
                    name,
                    type_,
                    initializer,
                    ..
                } => {
                    // Find the position of this variable in the content
                    if let Some((line, char_pos)) = self.find_variable_position(content, name) {
                        let range = make_range(line, char_pos, name.len());
                        let type_info = self.infer_variable_type(type_, initializer);
                        let detail = self.format_variable_detail(name, &type_info, initializer);
                        symbols.insert(
                            name.clone(),
                            make_symbol(
                                name.clone(),
                                SymbolKind::VARIABLE,
                                range,
                                detail,
                                None,
                                type_info,
                            ),
                        );
                    }
                }
                Statement::Loop { body, .. } => {
                    self.extract_variables_from_statements(body, content, symbols);
                }
                _ => {}
            }
        }
    }

    pub(super) fn find_variable_position(
        &self,
        content: &str,
        var_name: &str,
    ) -> Option<(usize, usize)> {
        for (line_num, line) in content.lines().enumerate() {
            // Look for variable declaration pattern: name = or name: Type =
            if let Some(eq_pos) = line.find('=') {
                let before_eq = line[..eq_pos].trim();
                // Check if it matches our variable name
                if before_eq == var_name || before_eq.ends_with(&format!(" {}", var_name)) {
                    if let Some(char_pos) = line.find(var_name) {
                        return Some((line_num, char_pos));
                    }
                }
            }
            // Also check for name: Type = pattern
            if let Some(colon_pos) = line.find(':') {
                let before_colon = line[..colon_pos].trim();
                if before_colon == var_name {
                    if let Some(char_pos) = line.find(var_name) {
                        return Some((line_num, char_pos));
                    }
                }
            }
        }
        None
    }

    fn infer_variable_type(
        &self,
        type_: &Option<AstType>,
        initializer: &Option<Expression>,
    ) -> Option<AstType> {
        if type_.is_some() {
            return type_.clone();
        }
        if let Some(init) = initializer {
            use crate::lsp::type_query::TypeQuery;
            if let Some(type_str) = TypeQuery::infer_literal_type(init) {
                return crate::parser::parse_type_from_string(&type_str).ok();
            }
        }
        None
    }

    /// Format variable detail string for display
    fn format_variable_detail(
        &self,
        name: &str,
        type_info: &Option<AstType>,
        initializer: &Option<Expression>,
    ) -> Option<String> {
        if let Some(t) = type_info {
            return Some(format!("{}: {}", name, format_type(t)));
        }
        if let Some(init) = initializer {
            if let Some(inferred) = self.infer_type_from_expression(init) {
                return Some(format!("{}: {}", name, inferred));
            }
        }
        Some(name.to_string())
    }

    pub(super) fn infer_type_from_expression(&self, expr: &Expression) -> Option<String> {
        use crate::lsp::type_query::TypeQuery;
        TypeQuery::infer_literal_type(expr)
    }
}
