// Helper functions for reducing duplication
use super::super::types::SymbolInfo;
use crate::ast::AstType;
use lsp_types::*;

/// Create a Range from line/char position and symbol length
pub fn make_range(line: usize, char_pos: usize, symbol_len: usize) -> Range {
    Range {
        start: Position {
            line: line as u32,
            character: char_pos as u32,
        },
        end: Position {
            line: line as u32,
            character: (char_pos + symbol_len) as u32,
        },
    }
}

/// Create a dummy range for built-in types (no source location)
pub fn dummy_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

/// Create a SymbolInfo with common defaults
pub fn make_symbol(
    name: String,
    kind: SymbolKind,
    range: Range,
    detail: Option<String>,
    documentation: Option<String>,
    type_info: Option<AstType>,
) -> SymbolInfo {
    SymbolInfo {
        name,
        kind,
        range,
        selection_range: range,
        detail,
        documentation,
        type_info,
        definition_uri: None,
        references: Vec::new(),
        enum_variants: None,
    }
}

/// Create a SymbolInfo for an enum (with variants)
pub fn make_enum_symbol(
    name: String,
    range: Range,
    detail: String,
    variants: Vec<String>,
) -> SymbolInfo {
    SymbolInfo {
        name,
        kind: SymbolKind::ENUM,
        range,
        selection_range: range,
        detail: Some(detail),
        documentation: None,
        type_info: None,
        definition_uri: None,
        references: Vec::new(),
        enum_variants: Some(variants),
    }
}
