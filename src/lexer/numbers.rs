use super::{Lexer, Token};
use crate::error::{CompileError, Span};

impl Lexer {
    pub(super) fn lex_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();

        if self.peek() == Some('0') {
            match self.peek_ahead(1) {
                Some('x') | Some('X') => return self.lex_prefixed_int(start, 16),
                Some('b') | Some('B') => return self.lex_prefixed_int(start, 2),
                Some('o') | Some('O') => return self.lex_prefixed_int(start, 8),
                _ => {}
            }
        }

        let digits_start = self.pos;
        self.eat_digits(|c| c.is_ascii_digit());

        if self.peek() == Some('.') && self.peek_ahead(1).is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            self.eat_digits(|c| c.is_ascii_digit());
            let text: String = self.source[digits_start..self.pos]
                .iter()
                .filter(|c| **c != '_')
                .collect();
            let val: f64 = text.parse().map_err(|_| {
                CompileError::Syntax(
                    format!("invalid float literal: {text}"),
                    Some(self.make_span(start, self.byte_pos())),
                )
            })?;
            return Ok((
                Token::FloatLiteral(val),
                self.make_span(start, self.byte_pos()),
            ));
        }

        let text: String = self.source[digits_start..self.pos]
            .iter()
            .filter(|c| **c != '_')
            .collect();
        let val: i64 = text.parse().map_err(|_| {
            CompileError::Syntax(
                format!("invalid integer literal: {text}"),
                Some(self.make_span(start, self.byte_pos())),
            )
        })?;
        Ok((
            Token::IntLiteral(val),
            self.make_span(start, self.byte_pos()),
        ))
    }

    fn lex_prefixed_int(&mut self, start: u32, radix: u32) -> Result<(Token, Span), CompileError> {
        self.advance_n(2);
        let digits_start = self.pos;

        let valid_digit = match radix {
            16 => (|c: char| c.is_ascii_hexdigit()) as fn(char) -> bool,
            2 => |c: char| c == '0' || c == '1',
            8 => |c: char| ('0'..='7').contains(&c),
            _ => unreachable!(),
        };
        self.eat_digits(valid_digit);

        let text: String = self.source[digits_start..self.pos]
            .iter()
            .filter(|c| **c != '_')
            .collect();
        if text.is_empty() {
            let prefix = match radix {
                16 => "0x",
                2 => "0b",
                8 => "0o",
                _ => unreachable!(),
            };
            return Err(CompileError::Syntax(
                format!("expected digits after {prefix}"),
                Some(self.make_span(start, self.byte_pos())),
            ));
        }
        let val = i64::from_str_radix(&text, radix).map_err(|_| {
            CompileError::Syntax(
                "integer literal out of range".to_string(),
                Some(self.make_span(start, self.byte_pos())),
            )
        })?;
        Ok((
            Token::IntLiteral(val),
            self.make_span(start, self.byte_pos()),
        ))
    }

    fn eat_digits(&mut self, is_valid: fn(char) -> bool) {
        while let Some(ch) = self.peek() {
            if is_valid(ch) || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
    }
}
