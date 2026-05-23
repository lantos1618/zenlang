use super::{Lexer, Token};
use crate::error::{CompileError, Span};

// ── Main scanning logic ──────────────────────────────────────────

impl Lexer {
    /// Produce the next token from the source.
    pub(super) fn lex_next(&mut self) -> Result<(Token, Span), CompileError> {
        self.skip_whitespace_and_comments()?;

        let start = self.byte_pos();

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok((Token::EOF, self.make_span(start, start))),
        };

        // Newlines are significant tokens
        if ch == '\n' {
            self.advance();
            return Ok((Token::Newline, self.make_span(start, self.byte_pos())));
        }

        // Strings
        if ch == '"' {
            return self.lex_string();
        }

        self.lex_non_string_token(start, ch)
    }

    pub(in crate::lexer) fn lex_non_string_token(
        &mut self,
        start: u32,
        ch: char,
    ) -> Result<(Token, Span), CompileError> {
        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.lex_identifier());
        }
        if ch.is_ascii_digit() {
            return self.lex_number();
        }
        if ch == '@' {
            return self.lex_at_token();
        }

        if let Some(tok) = self.lex_multi_char_operator(start) {
            return Ok(tok);
        }

        self.lex_single_char_token(start, ch)
    }

    pub(in crate::lexer) fn lex_multi_char_operator(
        &mut self,
        start: u32,
    ) -> Option<(Token, Span)> {
        for (spelling, token) in Token::MULTI_CHAR_OPERATORS {
            if self.matches(spelling) {
                self.advance_n(spelling.len());
                return Some((token.clone(), self.make_span(start, self.byte_pos())));
            }
        }
        None
    }

    pub(in crate::lexer) fn lex_single_char_token(
        &mut self,
        start: u32,
        ch: char,
    ) -> Result<(Token, Span), CompileError> {
        self.advance();
        let end = self.byte_pos();
        let Some(tok) = Token::from_single_char(ch) else {
            return Err(CompileError::Syntax(
                format!("unexpected character '{ch}'"),
                Some(self.make_span(start, end)),
            ));
        };
        Ok((tok, self.make_span(start, end)))
    }

    // ── Identifiers / keywords ───────────────────────────────────

    pub(super) fn lex_identifier(&mut self) -> (Token, Span) {
        let start = self.byte_pos();
        let char_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let word: String = self.source[char_start..self.pos].iter().collect();
        let span = self.make_span(start, self.byte_pos());
        let tok = match word.as_str() {
            "pub" => Token::Pub,
            _ => Token::Identifier(word),
        };
        (tok, span)
    }

    // ── @ tokens ─────────────────────────────────────────────────

    pub(super) fn lex_at_token(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();
        self.advance(); // consume '@'
        let id_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let word: String = self.source[id_start..self.pos].iter().collect();
        let span = self.make_span(start, self.byte_pos());
        let tok = match word.as_str() {
            "std" => Token::AtStd,
            "builtin" => Token::AtBuiltin,
            "this" => Token::AtThis,
            "export" => Token::AtExport,
            _ => Token::Identifier(format!("@{word}")),
        };
        Ok((tok, span))
    }
}
