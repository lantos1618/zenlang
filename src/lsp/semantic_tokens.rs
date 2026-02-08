// Semantic Tokens Module for Zen LSP
// Handles textDocument/semanticTokens/full requests

use lsp_server::{ErrorCode, Request, Response};
use lsp_types::*;

use super::document_store::DocumentStore;
use crate::ast::primitives;
use crate::lexer::{Lexer, Token};

// Token type indices (must match server capabilities legend)
const TYPE_NAMESPACE: u32 = 0;
const TYPE_TYPE: u32 = 1;
const TYPE_CLASS: u32 = 2;
const TYPE_ENUM: u32 = 3;
const TYPE_INTERFACE: u32 = 4;
const TYPE_VARIABLE: u32 = 8;
const TYPE_ENUM_MEMBER: u32 = 10;
const TYPE_FUNCTION: u32 = 12;
const TYPE_METHOD: u32 = 13;
const TYPE_KEYWORD: u32 = 15;
const TYPE_COMMENT: u32 = 17;
const TYPE_STRING: u32 = 18;
const TYPE_NUMBER: u32 = 19;
const TYPE_OPERATOR: u32 = 21;

// Token modifiers
const MOD_DECLARATION: u32 = 0b1;
const MOD_DEFINITION: u32 = 0b10;
const MOD_DEFAULT_LIBRARY: u32 = 0b1000000000;

// ============================================================================
// PUBLIC HANDLER
// ============================================================================

pub fn handle_semantic_tokens(
    req: Request,
    store: &std::sync::Arc<std::sync::RwLock<DocumentStore>>,
) -> Response {
    let params: SemanticTokensParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => return error_response(req.id, "Invalid parameters"),
    };

    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return empty_response(req.id),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(doc) => doc,
        None => return null_response(req.id),
    };

    let tokens = generate_semantic_tokens(&doc.content);
    let result = SemanticTokens {
        result_id: None,
        data: tokens,
    };

    crate::lsp::helpers::success_response_id(req.id, result)
}

fn error_response(id: lsp_server::RequestId, msg: &str) -> Response {
    crate::lsp::helpers::error_response_id(id, ErrorCode::InvalidParams, msg)
}

fn empty_response(id: lsp_server::RequestId) -> Response {
    crate::lsp::helpers::success_response_id(id, SemanticTokens::default())
}

fn null_response(id: lsp_server::RequestId) -> Response {
    crate::lsp::helpers::null_response_id(id)
}

// ============================================================================
// TOKEN GENERATION
// ============================================================================

#[derive(Debug)]
struct RawToken {
    line: u32,
    column: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

fn generate_semantic_tokens(content: &str) -> Vec<SemanticToken> {
    let mut raw_tokens: Vec<RawToken> = Vec::new();

    // Extract comments (lexer skips them)
    extract_comments(content, &mut raw_tokens);

    // Use compiler's lexer for all other tokens
    let mut lexer = Lexer::new(content);
    let mut after_fn = false;
    let mut after_dot = false;

    loop {
        let tok = lexer.next_token_with_span();
        if tok.token == Token::Eof {
            break;
        }

        let span = &tok.span;
        let line = span.line.saturating_sub(1) as u32;
        let column = span.column as u32;
        let length = (span.end - span.start) as u32;

        let (token_type, modifiers) = match &tok.token {
            Token::StringLiteral(_) => (TYPE_STRING, 0),
            Token::Integer(_) | Token::Float(_) => (TYPE_NUMBER, 0),
            Token::Question | Token::Pipe => (TYPE_OPERATOR, 0),
            Token::Underscore => (TYPE_VARIABLE, 0),
            Token::Pub => (TYPE_KEYWORD, 0),

            Token::Operator(op) if op == "." => {
                after_dot = true;
                (TYPE_OPERATOR, 0)
            }
            // Skip single < and > - let TextMate grammar handle the distinction
            // between generic type brackets (MutPtr<T>) and comparison operators (a < b).
            // Multi-char operators like <=, >=, <<, >> are always comparison/bitwise ops.
            Token::Operator(op) if op == "<" || op == ">" => continue,
            Token::Operator(_) => (TYPE_OPERATOR, 0),

            Token::Symbol('.') => {
                after_dot = true;
                (TYPE_OPERATOR, 0)
            }
            Token::Symbol(_) => continue,

            Token::AtStd | Token::AtThis | Token::AtMeta | Token::AtExport | Token::AtBuiltin => {
                (TYPE_NAMESPACE, MOD_DEFAULT_LIBRARY)
            }

            Token::Identifier(name) => {
                let result = classify_identifier(name, after_fn, after_dot);
                after_fn = name == "fn";
                after_dot = false;
                result
            }

            Token::InterpolationStart | Token::InterpolationEnd | Token::Eof => continue,
        };

        raw_tokens.push(RawToken {
            line,
            column,
            length,
            token_type,
            modifiers,
        });
    }

    // Sort by position and encode
    raw_tokens.sort_by(|a, b| (a.line, a.column).cmp(&(b.line, b.column)));
    encode_tokens(&raw_tokens)
}

// ============================================================================
// COMMENT EXTRACTION
// ============================================================================

fn extract_comments(content: &str, tokens: &mut Vec<RawToken>) {
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    let mut line: u32 = 0;
    let mut col: u32 = 0;

    while i < chars.len() {
        match chars[i] {
            '\n' => {
                line += 1;
                col = 0;
                i += 1;
            }
            '/' if matches!(chars.get(i + 1), Some('/')) => {
                let start_col = col;
                let start_i = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                    col += 1;
                }
                tokens.push(RawToken {
                    line,
                    column: start_col,
                    length: (i - start_i) as u32,
                    token_type: TYPE_COMMENT,
                    modifiers: 0,
                });
            }
            '/' if matches!(chars.get(i + 1), Some('*')) => {
                extract_block_comment(&chars, &mut i, &mut line, &mut col, tokens);
            }
            _ => {
                col += 1;
                i += 1;
            }
        }
    }
}

fn extract_block_comment(
    chars: &[char],
    i: &mut usize,
    line: &mut u32,
    col: &mut u32,
    tokens: &mut Vec<RawToken>,
) {
    let start_line = *line;
    let start_col = *col;
    *i += 2;
    *col += 2;

    let mut seg_start = *i;
    let mut seg_line = start_line;

    while *i < chars.len() {
        if chars[*i] == '*' && matches!(chars.get(*i + 1), Some('/')) {
            // End of comment
            let len = (*i + 2 - seg_start) as u32;
            let (c, extra) = if seg_line == start_line {
                (start_col, 2)
            } else {
                (0, 0)
            };
            tokens.push(RawToken {
                line: seg_line,
                column: c,
                length: len + extra,
                token_type: TYPE_COMMENT,
                modifiers: 0,
            });
            *i += 2;
            *col += 2;
            return;
        } else if chars[*i] == '\n' {
            // Emit line segment
            let len = (*i - seg_start) as u32;
            if seg_line == start_line {
                tokens.push(RawToken {
                    line: seg_line,
                    column: start_col,
                    length: len + 2,
                    token_type: TYPE_COMMENT,
                    modifiers: 0,
                });
            } else if len > 0 {
                tokens.push(RawToken {
                    line: seg_line,
                    column: 0,
                    length: len,
                    token_type: TYPE_COMMENT,
                    modifiers: 0,
                });
            }
            *i += 1;
            *line += 1;
            seg_line = *line;
            seg_start = *i;
            *col = 0;
        } else {
            *i += 1;
            *col += 1;
        }
    }
}

// ============================================================================
// IDENTIFIER CLASSIFICATION
// ============================================================================

fn classify_identifier(name: &str, after_fn: bool, after_dot: bool) -> (u32, u32) {
    // Keywords
    if is_keyword(name) {
        return (TYPE_KEYWORD, 0);
    }

    // Context-based classification
    if after_fn {
        return (TYPE_FUNCTION, MOD_DECLARATION | MOD_DEFINITION);
    }
    if after_dot {
        return (TYPE_METHOD, 0);
    }

    // Type classification
    if let Some(result) = classify_type(name) {
        return result;
    }

    // Uppercase = likely type
    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
        return (TYPE_TYPE, 0);
    }

    (TYPE_VARIABLE, 0)
}

/// Check if an identifier should be highlighted as a keyword for IDE purposes.
/// Uses the centralized definition from ast::primitives.
fn is_keyword(name: &str) -> bool {
    primitives::is_keyword_like(name)
}

fn classify_type(name: &str) -> Option<(u32, u32)> {
    // Use shared primitive definitions
    if primitives::is_primitive_name(name) {
        return Some((TYPE_TYPE, MOD_DEFAULT_LIBRARY));
    }

    // Collection types (dynamically discovered from stdlib)
    if crate::stdlib_types::stdlib_types().is_collection_type(name) {
        return Some((TYPE_CLASS, MOD_DEFAULT_LIBRARY));
    }

    // Pointer types
    if primitives::is_pointer_type(name) {
        return Some((TYPE_TYPE, MOD_DEFAULT_LIBRARY));
    }

    // Other well-known types
    Some(match name {
        // String types
        "String" => (TYPE_TYPE, MOD_DEFAULT_LIBRARY),
        // Sum types (use well_known in future)
        "Option" | "Result" => (TYPE_ENUM, MOD_DEFAULT_LIBRARY),
        // Enum members (use well_known in future)
        "Some" | "None" | "Ok" | "Err" => (TYPE_ENUM_MEMBER, 0),
        // Interfaces
        "Allocator" => (TYPE_INTERFACE, MOD_DEFAULT_LIBRARY),
        _ => return None,
    })
}

// ============================================================================
// DELTA ENCODING
// ============================================================================

fn encode_tokens(raw_tokens: &[RawToken]) -> Vec<SemanticToken> {
    let mut result = Vec::with_capacity(raw_tokens.len());
    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;

    for tok in raw_tokens {
        let delta_line = tok.line - prev_line;
        let delta_start = if delta_line == 0 {
            tok.column - prev_col
        } else {
            tok.column
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: tok.length,
            token_type: tok.token_type,
            token_modifiers_bitset: tok.modifiers,
        });

        prev_line = tok.line;
        prev_col = tok.column;
    }

    result
}
