// Symbol extraction from AST - delegates to shared core logic
use super::super::symbol_extraction::{
    extract_declaration_symbols, insert_trait_impl_symbols, ExtractionOptions,
};
use super::super::types::SymbolInfo;
use super::DocumentStore;
use crate::ast::Declaration;
use lsp_types::*;
use std::collections::HashMap;

impl DocumentStore {
    pub(super) fn extract_symbols(&self, content: &str) -> HashMap<String, SymbolInfo> {
        if let Some(ast) = self.parse(content) {
            return self.extract_symbols_from_ast(&ast, content);
        }
        HashMap::new()
    }

    pub(super) fn extract_symbols_from_ast(
        &self,
        ast: &[Declaration],
        content: &str,
    ) -> HashMap<String, SymbolInfo> {
        let opts = ExtractionOptions::default();

        // First pass: extract definitions using the shared core
        let mut symbols = extract_declaration_symbols(
            ast,
            &|decl, decl_index| {
                let (line, char_pos) = self.find_declaration_position(content, decl, decl_index);
                let symbol_name = match decl {
                    Declaration::Function(f) => &f.name,
                    Declaration::Struct(s) => &s.name,
                    Declaration::Enum(e) => &e.name,
                    Declaration::Constant { name, .. } => name,
                    _ => return Range::default(),
                };
                let name_end = char_pos + symbol_name.len();
                Range {
                    start: Position {
                        line: line as u32,
                        character: char_pos as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: name_end as u32,
                    },
                }
            },
            &opts,
        );

        // Second pass: references, variables, and trait implementation methods
        for decl in ast {
            match decl {
                Declaration::Function(func) => {
                    self.find_references_in_statements(&func.body, &mut symbols);
                    self.extract_variables_from_statements(&func.body, content, &mut symbols);
                }
                Declaration::TraitImplementation(impl_block) => {
                    let impl_range = self.find_impl_block_range(content, &impl_block.type_name);
                    insert_trait_impl_symbols(impl_block, impl_range, &opts, &mut symbols);
                }
                _ => {}
            }
        }

        symbols
    }

    pub(super) fn find_impl_block_range(&self, content: &str, type_name: &str) -> Range {
        let pattern = format!("{}.implements", type_name);
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(pos) = line.find(&pattern) {
                return Range {
                    start: Position {
                        line: line_num as u32,
                        character: pos as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: (pos + pattern.len()) as u32,
                    },
                };
            }
        }
        Range::default()
    }
}
