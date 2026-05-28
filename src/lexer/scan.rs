use super::{Lexer, Token};
use crate::error::{CompileError, Span};

impl Lexer {
    pub(super) fn lex_next(&mut self) -> Result<(Token, Span), CompileError> {
        self.skip_whitespace_and_comments()?;

        let start = self.byte_pos();

        let Some(ch) = self.peek() else {
            return Ok((Token::EOF, self.make_span(start, start)));
        };

        if ch == '\n' {
            self.advance();
            return Ok((Token::Newline, self.make_span(start, self.byte_pos())));
        }

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
            return Ok(self.lex_at_token());
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

    pub(super) fn lex_identifier(&mut self) -> (Token, Span) {
        let start = self.byte_pos();
        let word = self.lex_identifier_tail();
        let span = self.make_span(start, self.byte_pos());
        let tok = Token::from_keyword(&word).unwrap_or(Token::Identifier(word));
        (tok, span)
    }

    pub(super) fn lex_at_token(&mut self) -> (Token, Span) {
        let start = self.byte_pos();
        self.advance();
        let word = self.lex_identifier_tail();
        let span = self.make_span(start, self.byte_pos());
        let tok =
            Token::from_at_name(&word).unwrap_or_else(|| Token::Identifier(format!("@{word}")));
        (tok, span)
    }

    fn lex_identifier_tail(&mut self) -> String {
        let id_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.source[id_start..self.pos].iter().collect()
    }
}
