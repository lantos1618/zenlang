//! Symbol extraction from AST - extracted from document_store.rs

use super::types::SymbolInfo;
use super::utils::format_type;
use crate::ast::{Declaration, Statement};
use crate::lexer::Lexer;
use crate::parser::Parser;
use lsp_types::*;
use std::collections::HashMap;

/// Extract symbols from content (static version, no document context)
pub fn extract_symbols_static(
    content: &str,
    file_path: Option<&str>,
) -> HashMap<String, SymbolInfo> {
    let mut symbols = HashMap::new();

    // Parse the content
    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer);
    let ast = match parser.parse_program() {
        Ok(program) => program.declarations,
        Err(e) => {
            if let Some(path) = file_path {
                eprintln!("[LSP] Parse error in {}: {:?}", path, e);
            } else {
                eprintln!("[LSP] Parse error: {:?}", e);
            }
            return symbols;
        }
    };

    // Extract symbol definitions only (no reference tracking for performance)
    for decl in ast {
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 100,
            },
        };

        match decl {
            Declaration::Function(func) => {
                let detail = format!(
                    "{} = ({}) {}",
                    func.name,
                    func.args
                        .iter()
                        .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    format_type(&func.return_type)
                );

                symbols.insert(
                    func.name.clone(),
                    SymbolInfo {
                        name: func.name.clone(),
                        kind: SymbolKind::FUNCTION,
                        range,
                        selection_range: range,
                        detail: Some(detail),
                        documentation: None,
                        type_info: Some(func.return_type.clone()),
                        definition_uri: None,
                        references: Vec::new(),
                        enum_variants: None,
                        params: Some(func.args.clone()),
                    },
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
                    SymbolInfo {
                        name: struct_def.name.clone(),
                        kind: SymbolKind::STRUCT,
                        range,
                        selection_range: range,
                        detail: Some(detail),
                        documentation: None,
                        type_info: None,
                        definition_uri: None,
                        references: Vec::new(),
                        enum_variants: None,
                        params: None,
                    },
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
                    SymbolInfo {
                        name: enum_def.name.clone(),
                        kind: SymbolKind::ENUM,
                        range,
                        selection_range: range,
                        detail: Some(detail),
                        documentation: None,
                        type_info: None,
                        definition_uri: None,
                        references: Vec::new(),
                        enum_variants: Some(variant_names.clone()),
                        params: None,
                    },
                );

                // Add enum variants as symbols
                for variant_name in variant_names {
                    let full_name = format!("{}::{}", enum_def.name, variant_name);
                    symbols.insert(
                        full_name.clone(),
                        SymbolInfo {
                            name: variant_name.clone(),
                            kind: SymbolKind::ENUM_MEMBER,
                            range,
                            selection_range: range,
                            detail: Some(full_name),
                            documentation: None,
                            type_info: None,
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: None,
                            params: None,
                        },
                    );
                }
            }
            Declaration::Constant { name, type_, .. } => {
                symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::CONSTANT,
                        range,
                        selection_range: range,
                        detail: type_.as_ref().map(format_type),
                        documentation: None,
                        type_info: type_.clone(),
                        definition_uri: None,
                        references: Vec::new(),
                        enum_variants: None,
                        params: None,
                    },
                );
            }
            Declaration::TraitImplementation(impl_block) => {
                for method in &impl_block.methods {
                    let method_name = format!("{}.{}", impl_block.type_name, method.name);
                    let detail = format!(
                        "{}.{} = ({}) {}",
                        impl_block.type_name,
                        method.name,
                        method
                            .args
                            .iter()
                            .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                            .collect::<Vec<_>>()
                            .join(", "),
                        format_type(&method.return_type)
                    );

                    symbols.insert(
                        method_name.clone(),
                        SymbolInfo {
                            name: method.name.clone(),
                            kind: SymbolKind::METHOD,
                            range,
                            selection_range: range,
                            detail: Some(detail),
                            documentation: Some(format!(
                                "Method from {}.implements({})",
                                impl_block.type_name, impl_block.trait_name
                            )),
                            type_info: Some(method.return_type.clone()),
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: None,
                            params: Some(method.args.clone()),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    symbols
}

/// Extract symbols with path context and reference tracking
pub fn extract_symbols_with_path(
    parse_with_path: impl Fn(&str, Option<&str>) -> Option<Vec<Declaration>>,
    find_declaration_position: impl Fn(&str, &Declaration, usize) -> (usize, usize),
    find_references_in_statements: impl Fn(&[Statement], &mut HashMap<String, SymbolInfo>),
    extract_variables_from_statements: impl Fn(&[Statement], &str, &mut HashMap<String, SymbolInfo>),
    content: &str,
    file_path: Option<&str>,
) -> HashMap<String, SymbolInfo> {
    let mut symbols = HashMap::new();

    if let Some(ast) = parse_with_path(content, file_path) {
        // First pass: Extract symbol definitions
        for (decl_index, decl) in ast.iter().enumerate() {
            let (line, char_pos) = find_declaration_position(content, decl, decl_index);
            let symbol_name = match decl {
                Declaration::Function(f) => &f.name,
                Declaration::Struct(s) => &s.name,
                Declaration::Enum(e) => &e.name,
                Declaration::Constant { name, .. } => name,
                _ => continue,
            };
            let name_end = char_pos + symbol_name.len();
            let range = Range {
                start: Position {
                    line: line as u32,
                    character: char_pos as u32,
                },
                end: Position {
                    line: line as u32,
                    character: name_end as u32,
                },
            };

            match decl {
                Declaration::Function(func) => {
                    let detail = format!(
                        "{} = ({}) {}",
                        func.name,
                        func.args
                            .iter()
                            .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                            .collect::<Vec<_>>()
                            .join(", "),
                        format_type(&func.return_type)
                    );

                    symbols.insert(
                        func.name.clone(),
                        SymbolInfo {
                            name: func.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            range,
                            selection_range: range,
                            detail: Some(detail),
                            documentation: None,
                            type_info: Some(func.return_type.clone()),
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: None,
                            params: Some(func.args.clone()),
                        },
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
                        SymbolInfo {
                            name: struct_def.name.clone(),
                            kind: SymbolKind::STRUCT,
                            range,
                            selection_range: range,
                            detail: Some(detail),
                            documentation: None,
                            type_info: None,
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: None,
                            params: None,
                        },
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
                        SymbolInfo {
                            name: enum_def.name.clone(),
                            kind: SymbolKind::ENUM,
                            range,
                            selection_range: range,
                            detail: Some(detail),
                            documentation: None,
                            type_info: None,
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: Some(variant_names),
                            params: None,
                        },
                    );

                    // Add enum variants as symbols
                    for variant in &enum_def.variants {
                        let variant_name = format!("{}::{}", enum_def.name, variant.name);
                        symbols.insert(
                            variant_name.clone(),
                            SymbolInfo {
                                name: variant.name.clone(),
                                kind: SymbolKind::ENUM_MEMBER,
                                range,
                                selection_range: range,
                                detail: Some(format!("{}::{}", enum_def.name, variant.name)),
                                documentation: None,
                                type_info: None,
                                definition_uri: None,
                                references: Vec::new(),
                                enum_variants: None,
                                params: None,
                            },
                        );
                    }
                }
                Declaration::Constant { name, type_, .. } => {
                    symbols.insert(
                        name.clone(),
                        SymbolInfo {
                            name: name.clone(),
                            kind: SymbolKind::CONSTANT,
                            range,
                            selection_range: range,
                            detail: type_.as_ref().map(format_type),
                            documentation: None,
                            type_info: type_.clone(),
                            definition_uri: None,
                            references: Vec::new(),
                            enum_variants: None,
                            params: None,
                        },
                    );
                }
                Declaration::TraitImplementation(impl_block) => {
                    for method in &impl_block.methods {
                        let method_name = format!("{}.{}", impl_block.type_name, method.name);
                        let detail = format!(
                            "{}.{} = ({}) {}",
                            impl_block.type_name,
                            method.name,
                            method
                                .args
                                .iter()
                                .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                                .collect::<Vec<_>>()
                                .join(", "),
                            format_type(&method.return_type)
                        );

                        symbols.insert(
                            method_name.clone(),
                            SymbolInfo {
                                name: method.name.clone(),
                                kind: SymbolKind::METHOD,
                                range,
                                selection_range: range,
                                detail: Some(detail),
                                documentation: Some(format!(
                                    "Method from {}.implements({})",
                                    impl_block.type_name, impl_block.trait_name
                                )),
                                type_info: Some(method.return_type.clone()),
                                definition_uri: None,
                                references: Vec::new(),
                                enum_variants: None,
                                params: Some(method.args.clone()),
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        // Second pass: Find references to symbols and extract variables
        for decl in ast {
            if let Declaration::Function(func) = decl {
                find_references_in_statements(&func.body, &mut symbols);
                // Extract variables from function body
                extract_variables_from_statements(&func.body, content, &mut symbols);
            }
        }
    }

    symbols
}
