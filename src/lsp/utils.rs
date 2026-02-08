// LSP Utility Functions

use crate::ast::AstType;
use crate::error::{CompileError, Span};
use crate::lexer::{Lexer, Token};
use crate::stdlib_types::StdlibTypeRegistry;
use lsp_types::*;

// Re-export shared line analysis utilities from formatting module
pub use crate::formatting::{analyze_line_tokens, is_pattern_arm_line, LineTokenInfo};

/// Tokenize content and return tokens with their line numbers.
/// Used for accurate folding range detection.
pub struct TokenWithLine {
    pub token: Token,
    pub line: u32,
    pub column: u32,
}

/// Tokenize entire content and return tokens with line information.
/// This is useful for folding ranges where we need to track brace positions.
pub fn tokenize_with_lines(content: &str) -> Vec<TokenWithLine> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(content);

    loop {
        let token_with_span = lexer.next_token_with_span();
        if token_with_span.token == Token::Eof {
            break;
        }

        tokens.push(TokenWithLine {
            token: token_with_span.token,
            line: token_with_span.span.line as u32 - 1, // Convert to 0-based
            column: token_with_span.span.column as u32,
        });
    }

    tokens
}

/// Check if a line contains a pattern match scrutinee (ends with `?` token).
/// Returns the position of the `?` if found.
pub fn find_pattern_match_question(line: &str) -> Option<usize> {
    let info = analyze_line_tokens(line);
    if info.ends_with_question {
        // Find the position of the last '?' that's actually a token
        // We need to search from the end
        line.rfind('?')
    } else {
        None
    }
}

/// Convert a byte offset to LSP Position (line and character)
/// Returns (line, character) where line is 0-based and character is UTF-16 code unit offset
pub fn byte_offset_to_lsp_position(content: &str, byte_offset: usize) -> Position {
    let mut line = 0u32;
    let mut line_start_offset = 0usize;

    for (idx, ch) in content.char_indices() {
        if idx >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start_offset = idx + 1;
        }
    }

    // Calculate character offset within the line (in UTF-16 code units for LSP)
    let char_offset = if byte_offset >= line_start_offset {
        content[line_start_offset..byte_offset.min(content.len())]
            .chars()
            .map(|c| c.len_utf16() as u32)
            .sum()
    } else {
        0
    };

    Position {
        line,
        character: char_offset,
    }
}

/// Convert a compiler Span to LSP Range using source content for accurate multi-line handling
pub fn span_to_lsp_range(span: &Span, content: Option<&str>) -> Range {
    if let Some(content) = content {
        // Use accurate byte offset conversion
        let start = byte_offset_to_lsp_position(content, span.start);
        let end = byte_offset_to_lsp_position(content, span.end);
        Range { start, end }
    } else {
        // Fallback: use span's line/column for start, estimate end on same line
        let start = Position {
            line: if span.line > 0 {
                span.line as u32 - 1
            } else {
                0
            },
            character: span.column as u32,
        };
        let end = Position {
            line: start.line,
            character: (span.column + (span.end.saturating_sub(span.start)).max(1)) as u32,
        };
        Range { start, end }
    }
}

// Convert CompileError to LSP Diagnostic (without source context)
pub fn compile_error_to_diagnostic(error: CompileError) -> Diagnostic {
    compile_error_to_diagnostic_with_content(error, None)
}

// Convert CompileError to LSP Diagnostic with source content for position inference
pub fn compile_error_to_diagnostic_with_content(
    error: CompileError,
    content: Option<&str>,
) -> Diagnostic {
    // Extract span and determine severity
    let (span, severity, code) = match &error {
        CompileError::ParseError(_, span) => {
            (span.clone(), DiagnosticSeverity::ERROR, Some("parse-error"))
        }
        CompileError::SyntaxError(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("syntax-error"),
        ),
        CompileError::TypeError(_, span) => {
            (span.clone(), DiagnosticSeverity::ERROR, Some("type-error"))
        }
        CompileError::TypeMismatch { span, .. } => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("type-mismatch"),
        ),
        CompileError::UndeclaredVariable(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("undeclared-variable"),
        ),
        CompileError::UndeclaredFunction(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("undeclared-function"),
        ),
        CompileError::UnexpectedToken { span, .. } => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("unexpected-token"),
        ),
        CompileError::InvalidPattern(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("invalid-pattern"),
        ),
        CompileError::InvalidSyntax { span, .. } => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("invalid-syntax"),
        ),
        CompileError::MissingTypeAnnotation(_, span) => (
            span.clone(),
            DiagnosticSeverity::WARNING,
            Some("missing-type"),
        ),
        CompileError::DuplicateDeclaration {
            duplicate_location, ..
        } => (
            duplicate_location.clone(),
            DiagnosticSeverity::ERROR,
            Some("duplicate-declaration"),
        ),
        CompileError::ImportError(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("import-error"),
        ),
        CompileError::FFIError(_, span) => {
            (span.clone(), DiagnosticSeverity::ERROR, Some("ffi-error"))
        }
        CompileError::InvalidLoopCondition(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("invalid-loop"),
        ),
        CompileError::MissingReturnStatement(_, span) => (
            span.clone(),
            DiagnosticSeverity::WARNING,
            Some("missing-return"),
        ),
        CompileError::InternalError(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("internal-error"),
        ),
        CompileError::UnsupportedFeature(_, span) => (
            span.clone(),
            DiagnosticSeverity::WARNING,
            Some("unsupported-feature"),
        ),
        CompileError::FileNotFound(_, _) => {
            (None, DiagnosticSeverity::ERROR, Some("file-not-found"))
        }
        CompileError::ComptimeError(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("comptime-error"),
        ),
        CompileError::BuildError(_, span) => {
            (span.clone(), DiagnosticSeverity::ERROR, Some("build-error"))
        }
        CompileError::FileError(_, span) => {
            (span.clone(), DiagnosticSeverity::ERROR, Some("file-error"))
        }
        CompileError::CyclicDependency(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            Some("cyclic-dependency"),
        ),
    };

    // Convert span to LSP range, or try to infer from error message and content
    let (start_pos, end_pos) = if let Some(span) = span {
        let range = span_to_lsp_range(&span, content);
        (range.start, range.end)
    } else if let Some(content) = content {
        infer_error_position(&error, content)
    } else {
        (
            Position {
                line: 0,
                character: 0,
            },
            Position {
                line: 0,
                character: 1,
            },
        )
    };

    Diagnostic {
        range: Range {
            start: start_pos,
            end: end_pos,
        },
        severity: Some(severity),
        code: code.map(|c| lsp_types::NumberOrString::String(c.to_string())),
        code_description: None,
        source: Some("zen-compiler".to_string()),
        message: format!("{}", error),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Try to infer error position from error message and source content
fn infer_error_position(error: &CompileError, content: &str) -> (Position, Position) {
    let search_terms = extract_search_terms(error);

    for term in search_terms {
        if let Some((line, col, len)) = find_in_content(&term, content) {
            return (
                Position {
                    line: line as u32,
                    character: col as u32,
                },
                Position {
                    line: line as u32,
                    character: (col + len) as u32,
                },
            );
        }
    }

    // Default to first line if nothing found
    (
        Position {
            line: 0,
            character: 0,
        },
        Position {
            line: 0,
            character: 1,
        },
    )
}

/// Extract searchable terms from error message
fn extract_search_terms(error: &CompileError) -> Vec<String> {
    let mut terms = Vec::new();

    match error {
        CompileError::TypeError(msg, _) => {
            // Try to extract function/variable names from error message
            // "Unknown function: foo" -> search for "foo"
            if let Some(idx) = msg.find("Unknown function: ") {
                let name = msg[idx + 18..].trim();
                // Remove any trailing punctuation
                let name = name.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !name.is_empty() {
                    terms.push(format!("{}(", name)); // Function call
                    terms.push(name.to_string());
                }
            }
            // "'foo' is not a function" -> search for "foo("
            if let Some(after_quote) = msg.strip_prefix('\'') {
                if let Some(end_quote) = after_quote.find('\'') {
                    let name = &after_quote[..end_quote];
                    if !name.is_empty() {
                        terms.push(format!("{}(", name)); // Function call
                        terms.push(name.to_string());
                    }
                }
            }
            // "Undeclared variable: 'foo'" -> search for "foo"
            if let Some(idx) = msg.find("Undeclared variable: '") {
                let start = idx + 22;
                if let Some(end) = msg[start..].find('\'') {
                    let name = &msg[start..start + end];
                    if !name.is_empty() {
                        terms.push(name.to_string());
                    }
                }
            }
        }
        CompileError::UndeclaredVariable(name, _) => {
            terms.push(name.clone());
        }
        CompileError::UndeclaredFunction(name, _) => {
            terms.push(format!("{}(", name));
            terms.push(name.clone());
        }
        _ => {}
    }

    terms
}

/// Find a term in content and return (line, column, length)
fn find_in_content(term: &str, content: &str) -> Option<(usize, usize, usize)> {
    for (line_num, line) in content.lines().enumerate() {
        // Skip comment lines
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if let Some(col) = line.find(term) {
            return Some((line_num, col, term.len()));
        }
    }
    None
}

pub fn format_symbol_kind(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::FILE => "File",
        SymbolKind::MODULE => "Module",
        SymbolKind::NAMESPACE => "Namespace",
        SymbolKind::PACKAGE => "Package",
        SymbolKind::CLASS => "Class",
        SymbolKind::METHOD => "Method",
        SymbolKind::PROPERTY => "Property",
        SymbolKind::FIELD => "Field",
        SymbolKind::CONSTRUCTOR => "Constructor",
        SymbolKind::ENUM => "Enum",
        SymbolKind::INTERFACE => "Interface",
        SymbolKind::FUNCTION => "Function",
        SymbolKind::VARIABLE => "Variable",
        SymbolKind::CONSTANT => "Constant",
        SymbolKind::STRING => "String",
        SymbolKind::NUMBER => "Number",
        SymbolKind::BOOLEAN => "Boolean",
        SymbolKind::ARRAY => "Array",
        SymbolKind::OBJECT => "Object",
        SymbolKind::KEY => "Key",
        SymbolKind::NULL => "Null",
        SymbolKind::ENUM_MEMBER => "Enum Member",
        SymbolKind::STRUCT => "Struct",
        SymbolKind::EVENT => "Event",
        SymbolKind::OPERATOR => "Operator",
        SymbolKind::TYPE_PARAMETER => "Type Parameter",
        _ => "Unknown",
    }
}

pub fn symbol_kind_to_completion_kind(kind: SymbolKind) -> CompletionItemKind {
    match kind {
        SymbolKind::FUNCTION | SymbolKind::METHOD => CompletionItemKind::FUNCTION,
        SymbolKind::STRUCT | SymbolKind::CLASS => CompletionItemKind::STRUCT,
        SymbolKind::ENUM => CompletionItemKind::ENUM,
        SymbolKind::ENUM_MEMBER => CompletionItemKind::ENUM_MEMBER,
        SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
        SymbolKind::CONSTANT => CompletionItemKind::CONSTANT,
        SymbolKind::FIELD | SymbolKind::PROPERTY => CompletionItemKind::FIELD,
        SymbolKind::INTERFACE => CompletionItemKind::INTERFACE,
        SymbolKind::MODULE | SymbolKind::NAMESPACE => CompletionItemKind::MODULE,
        SymbolKind::TYPE_PARAMETER => CompletionItemKind::TYPE_PARAMETER,
        SymbolKind::CONSTRUCTOR => CompletionItemKind::CONSTRUCTOR,
        SymbolKind::EVENT => CompletionItemKind::EVENT,
        SymbolKind::OPERATOR => CompletionItemKind::OPERATOR,
        _ => CompletionItemKind::TEXT,
    }
}

pub fn format_type(ast_type: &AstType) -> String {
    // LSP-specific overrides that differ from AstType::Display
    match ast_type {
        AstType::StaticLiteral => return "str".to_string(),
        AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => {
            return "String".to_string();
        }
        AstType::Ref(inner) => return format!("&{}", format_type(inner)),
        AstType::StdModule => return "module".to_string(),
        AstType::FunctionPointer {
            param_types,
            return_type,
        } => {
            return format!(
                "fn({}) {}",
                param_types
                    .iter()
                    .map(format_type)
                    .collect::<Vec<_>>()
                    .join(", "),
                format_type(return_type)
            );
        }
        AstType::Range {
            start_type,
            end_type,
            inclusive,
        } => {
            if *inclusive {
                return format!("{}..={}", format_type(start_type), format_type(end_type));
            } else {
                return format!("{}..{}", format_type(start_type), format_type(end_type));
            }
        }
        _ => {}
    }

    // Delegate to AstType's Display impl for everything else
    format!("{}", ast_type)
}

#[cfg(test)]
mod tests {
    use crate::error::Span;
    use crate::lexer::Token;
    use crate::lsp::utils::{
        analyze_line_tokens, byte_offset_to_lsp_position, find_pattern_match_question,
        is_pattern_arm_line, span_to_lsp_range, tokenize_with_lines,
    };

    #[test]
    fn test_byte_offset_to_lsp_position_simple() {
        let content = "hello\nworld";

        let pos = byte_offset_to_lsp_position(content, 0);
        assert_eq!((pos.line, pos.character), (0, 0));

        let pos = byte_offset_to_lsp_position(content, 3);
        assert_eq!((pos.line, pos.character), (0, 3));

        let pos = byte_offset_to_lsp_position(content, 5);
        assert_eq!((pos.line, pos.character), (0, 5));

        let pos = byte_offset_to_lsp_position(content, 6);
        assert_eq!((pos.line, pos.character), (1, 0));

        let pos = byte_offset_to_lsp_position(content, 9);
        assert_eq!((pos.line, pos.character), (1, 3));
    }

    #[test]
    fn test_byte_offset_emoji_utf16_surrogate_pairs() {
        let content = "a😀b";

        let pos = byte_offset_to_lsp_position(content, 0);
        assert_eq!((pos.line, pos.character), (0, 0));

        let pos = byte_offset_to_lsp_position(content, 1);
        assert_eq!((pos.line, pos.character), (0, 1));

        let pos = byte_offset_to_lsp_position(content, 5);
        assert_eq!((pos.line, pos.character), (0, 3));
    }

    #[test]
    fn test_byte_offset_japanese_bmp_characters() {
        let content = "日本語";

        let pos = byte_offset_to_lsp_position(content, 0);
        assert_eq!((pos.line, pos.character), (0, 0));

        let pos = byte_offset_to_lsp_position(content, 3);
        assert_eq!((pos.line, pos.character), (0, 1));

        let pos = byte_offset_to_lsp_position(content, 6);
        assert_eq!((pos.line, pos.character), (0, 2));
    }

    #[test]
    fn test_byte_offset_multiline_mixed_unicode() {
        let content = "hello\n世界\n😀";

        let pos = byte_offset_to_lsp_position(content, 0);
        assert_eq!((pos.line, pos.character), (0, 0));

        let pos = byte_offset_to_lsp_position(content, 6);
        assert_eq!((pos.line, pos.character), (1, 0));

        let pos = byte_offset_to_lsp_position(content, 9);
        assert_eq!((pos.line, pos.character), (1, 1));

        let pos = byte_offset_to_lsp_position(content, 13);
        assert_eq!((pos.line, pos.character), (2, 0));
    }

    #[test]
    fn test_byte_offset_edge_cases() {
        let pos = byte_offset_to_lsp_position("", 0);
        assert_eq!((pos.line, pos.character), (0, 0));

        let pos = byte_offset_to_lsp_position("hi", 100);
        assert_eq!(pos.line, 0);

        let content = "a\n\nb";
        let pos = byte_offset_to_lsp_position(content, 2);
        assert_eq!((pos.line, pos.character), (1, 0));

        let pos = byte_offset_to_lsp_position(content, 3);
        assert_eq!((pos.line, pos.character), (2, 0));
    }

    #[test]
    fn test_span_to_lsp_range_with_content() {
        let content = "hello\nworld\ntest";
        let span = Span {
            start: 6,
            end: 11,
            line: 2,
            column: 0,
        };

        let range = span_to_lsp_range(&span, Some(content));
        assert_eq!((range.start.line, range.start.character), (1, 0));
        assert_eq!((range.end.line, range.end.character), (1, 5));
    }

    #[test]
    fn test_span_to_lsp_range_fallback_without_content() {
        let span = Span {
            start: 10,
            end: 15,
            line: 3,
            column: 5,
        };

        let range = span_to_lsp_range(&span, None);
        assert_eq!((range.start.line, range.start.character), (2, 5));
        assert_eq!((range.end.line, range.end.character), (2, 10));
    }

    #[test]
    fn test_span_to_lsp_range_multiline_span() {
        let content = "line1\nline2\nline3";
        let span = Span {
            start: 0,
            end: 11,
            line: 1,
            column: 0,
        };

        let range = span_to_lsp_range(&span, Some(content));
        assert_eq!((range.start.line, range.start.character), (0, 0));
        assert_eq!((range.end.line, range.end.character), (1, 5));
    }

    // Tests for lexer-based token analysis

    #[test]
    fn test_analyze_line_tokens_braces_in_string() {
        let info = analyze_line_tokens(r#"msg = "{ braces }""#);
        assert_eq!(info.open_braces, 0, "braces in string not counted");
        assert_eq!(info.close_braces, 0);
    }

    #[test]
    fn test_analyze_line_tokens_real_braces() {
        let info = analyze_line_tokens("foo = () {");
        assert_eq!(info.open_braces, 1);
        assert_eq!(info.close_braces, 0);
    }

    #[test]
    fn test_analyze_line_tokens_question_in_string() {
        let info = analyze_line_tokens(r#"msg = "What?""#);
        assert!(!info.ends_with_question);
    }

    #[test]
    fn test_analyze_line_tokens_real_question() {
        let info = analyze_line_tokens("result?");
        assert!(info.ends_with_question);
    }

    #[test]
    fn test_find_pattern_match_question() {
        assert!(find_pattern_match_question("result?").is_some());
        assert!(find_pattern_match_question(r#"msg = "What?""#).is_none());
    }

    #[test]
    fn test_is_pattern_arm_line_util() {
        assert!(is_pattern_arm_line("| Ok(x) { }"));
        assert!(!is_pattern_arm_line(r#"regex = "a|b""#));
    }

    #[test]
    fn test_tokenize_with_lines() {
        let content = "foo = {\n  bar\n}";
        let tokens = tokenize_with_lines(content);

        // Find the opening brace
        let open_brace = tokens
            .iter()
            .find(|t| matches!(t.token, Token::Symbol('{')));
        assert!(open_brace.is_some());
        assert_eq!(open_brace.unwrap().line, 0);

        // Find the closing brace
        let close_brace = tokens
            .iter()
            .find(|t| matches!(t.token, Token::Symbol('}')));
        assert!(close_brace.is_some());
        assert_eq!(close_brace.unwrap().line, 2);
    }
}
