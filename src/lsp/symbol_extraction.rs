//! Symbol extraction from AST - shared core logic for both static indexing
//! and document-aware extraction.

use super::types::SymbolInfo;
use super::utils::format_type;
use crate::ast::{AstType, Declaration};
use crate::lexer::Lexer;
use crate::parser::Parser;
use lsp_types::*;
use std::collections::HashMap;

/// Options controlling symbol extraction behavior.
#[derive(Default)]
pub struct ExtractionOptions {
    /// Whether to populate the `params` field on function/method symbols.
    /// The static indexing path uses this; the document store path does not.
    pub include_params: bool,
    /// Whether to use `format_type()` (LSP-friendly) or `Display` for type formatting.
    /// Static indexing uses `format_type`; document store uses `Display`.
    pub use_format_type: bool,
}

/// Format an `AstType` according to the extraction options.
fn fmt_type(ty: &AstType, opts: &ExtractionOptions) -> String {
    if opts.use_format_type {
        format_type(ty)
    } else {
        ty.to_string()
    }
}

/// Shared core: extract symbol definitions from a slice of declarations.
///
/// `range_for_decl` is called for each declaration to compute the LSP range.
/// This handles Function, Struct, Enum, and Constant declarations. Trait
/// implementations are intentionally left to callers because the two extraction
/// paths compute their ranges differently (dummy range vs. `find_impl_block_range`).
pub fn extract_declaration_symbols(
    declarations: &[Declaration],
    range_for_decl: &dyn Fn(&Declaration, usize) -> Range,
    opts: &ExtractionOptions,
) -> HashMap<String, SymbolInfo> {
    let mut symbols = HashMap::new();

    for (decl_index, decl) in declarations.iter().enumerate() {
        let range = range_for_decl(decl, decl_index);

        match decl {
            Declaration::Function(func) => {
                let args_str = func
                    .args
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, fmt_type(ty, opts)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let detail = format!(
                    "{} = ({}) {}",
                    func.name,
                    args_str,
                    fmt_type(&func.return_type, opts)
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
                        params: if opts.include_params {
                            Some(func.args.clone())
                        } else {
                            None
                        },
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
                    let variant_key = format!("{}::{}", enum_def.name, variant.name);
                    symbols.insert(
                        variant_key.clone(),
                        SymbolInfo {
                            name: variant.name.clone(),
                            kind: SymbolKind::ENUM_MEMBER,
                            range,
                            selection_range: range,
                            detail: Some(variant_key),
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
                let detail = type_.as_ref().map(|t| fmt_type(t, opts));

                symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::CONSTANT,
                        range,
                        selection_range: range,
                        detail,
                        documentation: None,
                        type_info: type_.clone(),
                        definition_uri: None,
                        references: Vec::new(),
                        enum_variants: None,
                        params: None,
                    },
                );
            }
            _ => {}
        }
    }

    symbols
}

/// Insert trait implementation method symbols into an existing symbol map.
///
/// This is shared logic used by both extraction paths. The caller provides
/// the range to use for the methods.
pub fn insert_trait_impl_symbols(
    impl_block: &crate::ast::TraitImplementation,
    range: Range,
    opts: &ExtractionOptions,
    symbols: &mut HashMap<String, SymbolInfo>,
) {
    for method in &impl_block.methods {
        let method_name = format!("{}.{}", impl_block.type_name, method.name);
        let args_str = method
            .args
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, fmt_type(ty, opts)))
            .collect::<Vec<_>>()
            .join(", ");
        let detail = format!(
            "{}.{} = ({}) {}",
            impl_block.type_name,
            method.name,
            args_str,
            fmt_type(&method.return_type, opts)
        );

        symbols.insert(
            method_name,
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
                params: if opts.include_params {
                    Some(method.args.clone())
                } else {
                    None
                },
            },
        );
    }
}

/// Extract symbols from content (static version, no document context).
///
/// Used for workspace/stdlib indexing where we don't have a `DocumentStore`
/// and don't need reference tracking or accurate source positions.
pub fn extract_symbols_static(
    content: &str,
    file_path: Option<&str>,
) -> HashMap<String, SymbolInfo> {
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
            return HashMap::new();
        }
    };

    let dummy_range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 100,
        },
    };

    let opts = ExtractionOptions {
        include_params: true,
        use_format_type: true,
    };

    // Extract definitions (Function, Struct, Enum, Constant)
    let mut symbols = extract_declaration_symbols(&ast, &|_decl, _idx| dummy_range, &opts);

    // Also extract trait implementation methods with the dummy range
    for decl in &ast {
        if let Declaration::TraitImplementation(impl_block) = decl {
            insert_trait_impl_symbols(impl_block, dummy_range, &opts, &mut symbols);
        }
    }

    symbols
}
