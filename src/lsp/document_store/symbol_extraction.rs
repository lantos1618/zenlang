// Symbol extraction from AST
use super::super::types::SymbolInfo;
use super::utilities::{make_enum_symbol, make_range, make_symbol};
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
        let mut symbols = HashMap::new();

        // First pass: Extract symbol definitions
        for (decl_index, decl) in ast.iter().enumerate() {
            let (line, char_pos) = self.find_declaration_position(content, decl, decl_index);
            let symbol_name = match decl {
                Declaration::Function(f) => &f.name,
                Declaration::Struct(s) => &s.name,
                Declaration::Enum(e) => &e.name,
                Declaration::Constant { name, .. } => name,
                _ => continue,
            };
            let range = make_range(line, char_pos, symbol_name.len());

            match decl {
                Declaration::Function(func) => {
                    let args_str = func
                        .args
                        .iter()
                        .map(|(name, ty)| format!("{}: {}", name, ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let detail = format!("{} = ({}) {}", func.name, args_str, func.return_type);
                    symbols.insert(
                        func.name.clone(),
                        make_symbol(
                            func.name.clone(),
                            SymbolKind::FUNCTION,
                            range,
                            Some(detail),
                            None,
                            Some(func.return_type.clone()),
                        ),
                    );
                }
                Declaration::Struct(struct_def) => {
                    let detail = format!(
                        "{} struct with {} fields",
                        struct_def.name,
                        struct_def.fields.len()
                    );
                    symbols.insert(
                        struct_def.name.clone(),
                        make_symbol(
                            struct_def.name.clone(),
                            SymbolKind::STRUCT,
                            range,
                            Some(detail),
                            None,
                            None,
                        ),
                    );
                }
                Declaration::Enum(enum_def) => {
                    let detail = format!(
                        "{} enum with {} variants",
                        enum_def.name,
                        enum_def.variants.len()
                    );
                    let variant_names: Vec<String> =
                        enum_def.variants.iter().map(|v| v.name.clone()).collect();
                    symbols.insert(
                        enum_def.name.clone(),
                        make_enum_symbol(enum_def.name.clone(), range, detail, variant_names),
                    );
                    // Add enum variants as symbols
                    for variant in &enum_def.variants {
                        let variant_key = format!("{}::{}", enum_def.name, variant.name);
                        symbols.insert(
                            variant_key.clone(),
                            make_symbol(
                                variant.name.clone(),
                                SymbolKind::ENUM_MEMBER,
                                range,
                                Some(variant_key.clone()),
                                None,
                                None,
                            ),
                        );
                    }
                }
                Declaration::Constant { name, type_, .. } => {
                    symbols.insert(
                        name.clone(),
                        make_symbol(
                            name.clone(),
                            SymbolKind::CONSTANT,
                            range,
                            type_.as_ref().map(|t| t.to_string()),
                            None,
                            type_.clone(),
                        ),
                    );
                }
                _ => {}
            }
        }

        // Second pass: Find references, extract variables, and handle impl blocks
        for decl in ast {
            match decl {
                Declaration::Function(func) => {
                    self.find_references_in_statements(&func.body, &mut symbols);
                    self.extract_variables_from_statements(&func.body, content, &mut symbols);
                }
                Declaration::TraitImplementation(impl_block) => {
                    let impl_range = self.find_impl_block_range(content, &impl_block.type_name);
                    for method in &impl_block.methods {
                        let method_name = format!("{}.{}", impl_block.type_name, method.name);
                        let args_str = method
                            .args
                            .iter()
                            .map(|(name, ty)| format!("{}: {}", name, ty))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let detail = format!(
                            "{}.{} = ({}) {}",
                            impl_block.type_name, method.name, args_str, method.return_type
                        );
                        let doc = format!(
                            "Method from {}.implements({})",
                            impl_block.type_name, impl_block.trait_name
                        );
                        symbols.insert(
                            method_name,
                            make_symbol(
                                method.name.clone(),
                                SymbolKind::METHOD,
                                impl_range,
                                Some(detail),
                                Some(doc),
                                Some(method.return_type.clone()),
                            ),
                        );
                    }
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
