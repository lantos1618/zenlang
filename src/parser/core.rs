use super::super::lexer::{Lexer, LexerState, Token};
use crate::error::Span;

/// Saved parser state for backtracking during look-ahead
#[derive(Clone)]
pub struct ParserState {
    pub lexer_state: LexerState,
    pub current_token: Token,
    pub peek_token: Token,
    pub current_span: Span,
    pub peek_span: Span,
}

/// Maximum recursion depth for expression/statement parsing to prevent stack overflow
const MAX_PARSE_DEPTH: usize = 256;

pub struct Parser<'a> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) current_token: Token,
    pub(crate) peek_token: Token,
    pub(crate) current_span: Span,
    pub(crate) peek_span: Span,
    /// Current recursion depth for expression parsing
    pub(crate) parse_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token_with_span = lexer.next_token_with_span();
        let peek_token_with_span = lexer.next_token_with_span();
        Parser {
            lexer,
            current_token: current_token_with_span.token,
            peek_token: peek_token_with_span.token,
            current_span: current_token_with_span.span,
            peek_span: peek_token_with_span.span,
            parse_depth: 0,
        }
    }

    pub fn next_token(&mut self) {
        let token_with_span = self.lexer.next_token_with_span();
        self.current_token = self.peek_token.clone();
        self.peek_token = token_with_span.token;
        self.current_span = self.peek_span.clone();
        self.peek_span = token_with_span.span;
    }

    // ========================================================================
    // PARSER HELPER METHODS - Reduce duplication across parser modules
    // ========================================================================

    /// Create a syntax error with the current span
    #[inline]
    pub fn syntax_error(&self, message: impl Into<String>) -> crate::error::CompileError {
        crate::error::CompileError::SyntaxError(message.into(), Some(self.current_span.clone()))
    }

    /// Expect current token to be a specific symbol, return error if not
    pub fn expect_symbol(&mut self, expected: char) -> crate::error::Result<()> {
        if self.current_token == Token::Symbol(expected) {
            self.next_token();
            Ok(())
        } else {
            Err(self.syntax_error(format!(
                "Expected '{}', got {:?}",
                expected, self.current_token
            )))
        }
    }

    /// Try to split a `>>` or `>>=` token by consuming one `>`.
    /// Returns true if successfully consumed a `>` (either standalone or by splitting).
    /// Does NOT advance past a standalone `>` — that's done by the caller.
    fn try_split_right_angle(&mut self) -> bool {
        if self.current_token == Token::Operator(">>".to_string()) {
            self.current_token = Token::Operator(">".to_string());
            true
        } else if self.current_token == Token::Operator(">>=".to_string()) {
            self.current_token = Token::Operator(">=".to_string());
            true
        } else {
            false
        }
    }

    /// Expect current token to be a specific operator, return error if not
    /// Special handling for `>` when current token is `>>` or `>>=` (needed for nested generics)
    pub fn expect_operator(&mut self, expected: &str) -> crate::error::Result<()> {
        if self.current_token == Token::Operator(expected.to_string()) {
            self.next_token();
            Ok(())
        } else if expected == ">" && self.try_split_right_angle() {
            Ok(())
        } else {
            Err(self.syntax_error(format!(
                "Expected '{}', got {:?}",
                expected, self.current_token
            )))
        }
    }

    /// Try to consume a symbol if present, return true if consumed
    pub fn try_consume_symbol(&mut self, symbol: char) -> bool {
        if self.current_token == Token::Symbol(symbol) {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Try to consume an operator if present, return true if consumed
    /// Special handling for `>` when current token is `>>` or `>>=` (needed for nested generics like `Vec<Vec<T>>`)
    pub fn try_consume_operator(&mut self, op: &str) -> bool {
        if self.current_token == Token::Operator(op.to_string()) {
            self.next_token();
            true
        } else {
            op == ">" && self.try_split_right_angle()
        }
    }

    /// Extract identifier from current token, or return error
    pub fn expect_identifier(&mut self, context: &str) -> crate::error::Result<String> {
        if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.next_token();
            Ok(name)
        } else {
            Err(self.syntax_error(format!(
                "Expected {} (identifier), got {:?}",
                context, self.current_token
            )))
        }
    }

    /// Check if current token is a specific identifier keyword
    pub fn is_keyword(&self, keyword: &str) -> bool {
        matches!(&self.current_token, Token::Identifier(id) if id == keyword)
    }

    /// Consume a keyword if it matches, return true if consumed
    pub fn try_consume_keyword(&mut self, keyword: &str) -> bool {
        if self.is_keyword(keyword) {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Skip optional semicolon if present
    #[inline]
    pub fn skip_optional_semicolon(&mut self) {
        if self.current_token == Token::Symbol(';') {
            self.next_token();
        }
    }

    /// Parse generic type parameters like <T, U, V>
    /// Returns the type parameters as a string (e.g., "<T,U>")
    pub fn parse_generic_params_as_string(&mut self) -> String {
        if self.current_token != Token::Operator("<".to_string()) {
            return String::new();
        }

        let mut result = String::from("<");
        self.next_token(); // consume '<'

        while let Token::Identifier(param) = &self.current_token {
            result.push_str(param);
            self.next_token();

            if self.current_token == Token::Symbol(',') {
                result.push(',');
                self.next_token();
            } else if self.try_consume_operator(">") {
                result.push('>');
                break;
            } else {
                break;
            }
        }
        result
    }

    /// Parse type name with optional generic parameters
    /// Returns "TypeName" or "TypeName<T,U>"
    pub fn parse_type_name_with_generics(&mut self) -> crate::error::Result<String> {
        let name = self.expect_identifier("type name")?;
        let mut full_name = name;

        if self.current_token == Token::Operator("<".to_string()) {
            full_name.push_str(&self.parse_generic_params_as_string());
        }

        Ok(full_name)
    }

    /// Get current identifier without consuming, or None
    pub fn current_identifier(&self) -> Option<String> {
        if let Token::Identifier(name) = &self.current_token {
            Some(name.clone())
        } else {
            None
        }
    }

    /// Parse a dotted identifier path: `initial.member1.member2...`
    /// Consumes `.identifier` pairs as long as they're available.
    pub fn parse_dotted_path(&mut self, initial: String) -> String {
        let mut path = initial;
        while self.current_token == Token::Symbol('.') {
            self.next_token();
            if let Token::Identifier(member) = &self.current_token {
                path.push('.');
                path.push_str(member);
                self.next_token();
            } else {
                break;
            }
        }
        path
    }

    /// Parse a comma-separated list of identifiers enclosed by `close` char.
    /// Assumes the opening delimiter has already been consumed.
    /// Consumes the closing delimiter.
    pub fn parse_identifier_list(
        &mut self,
        close: char,
        context: &str,
    ) -> crate::error::Result<Vec<String>> {
        let mut names = vec![];
        while self.current_token != Token::Symbol(close) && self.current_token != Token::Eof {
            let name = self.expect_identifier(context)?;
            names.push(name);
            if !self.try_consume_symbol(',') && self.current_token != Token::Symbol(close) {
                return Err(
                    self.syntax_error(format!("Expected ',' or '{}' in {}", close, context))
                );
            }
        }
        if self.current_token != Token::Symbol(close) {
            return Err(self.syntax_error(format!("Expected '{}' to close {}", close, context)));
        }
        self.next_token(); // consume close
        Ok(names)
    }

    // ========================================================================
    // STATE SAVE/RESTORE FOR LOOK-AHEAD
    // ========================================================================

    /// Save the current parser state for potential backtracking
    pub fn save_state(&self) -> ParserState {
        ParserState {
            lexer_state: self.lexer.save_state(),
            current_token: self.current_token.clone(),
            peek_token: self.peek_token.clone(),
            current_span: self.current_span.clone(),
            peek_span: self.peek_span.clone(),
        }
    }

    /// Restore parser to a previously saved state
    pub fn restore_state(&mut self, state: ParserState) {
        self.lexer.restore_state(state.lexer_state);
        self.current_token = state.current_token;
        self.peek_token = state.peek_token;
        self.current_span = state.current_span;
        self.peek_span = state.peek_span;
    }

    /// Execute a closure with look-ahead, then restore state
    /// Returns whatever the closure returns
    pub fn with_lookahead<T, F: FnOnce(&mut Self) -> T>(&mut self, f: F) -> T {
        let saved = self.save_state();
        let result = f(self);
        self.restore_state(saved);
        result
    }

    /// Check and increment parse depth, returning error if too deep
    pub fn enter_recursion(&mut self) -> crate::error::Result<()> {
        self.parse_depth += 1;
        if self.parse_depth > MAX_PARSE_DEPTH {
            Err(crate::error::CompileError::SyntaxError(
                format!(
                    "Expression nesting too deep (exceeded {} levels)",
                    MAX_PARSE_DEPTH
                ),
                Some(self.current_span.clone()),
            ))
        } else {
            Ok(())
        }
    }

    /// Decrement parse depth
    pub fn exit_recursion(&mut self) {
        self.parse_depth = self.parse_depth.saturating_sub(1);
    }

    /// Skip past generic parameters <T, U, V> and return depth reached
    /// Returns 0 if not at '<', returns -1 if generics are malformed
    pub fn skip_generic_params(&mut self) -> i32 {
        if self.current_token != Token::Operator("<".to_string()) {
            return 0;
        }
        self.next_token(); // consume '<'
        let mut depth: i32 = 1;
        let mut iterations = 0;
        const MAX_GENERIC_TOKENS: usize = 1000;
        while depth > 0 && self.current_token != Token::Eof {
            iterations += 1;
            if iterations > MAX_GENERIC_TOKENS {
                return -1; // malformed - too many tokens in generic params
            }
            if self.current_token == Token::Operator("<".to_string()) {
                depth += 1;
            } else if self.current_token == Token::Operator(">".to_string()) {
                depth -= 1;
            } else if self.current_token == Token::Operator(">>".to_string()) {
                // Handle nested generics: `>>` counts as two `>` tokens
                depth -= 2;
                if depth == 0 {
                    // Consumed both `>` tokens, advance past `>>`
                    self.next_token();
                    return 1;
                } else if depth < 0 {
                    // One `>` was extra, leave it as current token
                    self.current_token = Token::Operator(">".to_string());
                    return 1;
                }
            }
            if depth > 0 {
                self.next_token();
            }
        }
        if depth == 0 {
            self.next_token(); // consume final '>'
            1
        } else {
            -1 // malformed
        }
    }
}
