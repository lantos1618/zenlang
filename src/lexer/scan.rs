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

        // Numbers
        if ch.is_ascii_digit() {
            return self.lex_number();
        }

        // @ tokens
        if ch == '@' {
            return self.lex_at_token();
        }

        // Identifiers / keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.lex_identifier());
        }

        // Multi-char operators (longest match first)
        if self.matches("::=") {
            self.advance_n(3);
            return Ok((Token::DeclareAssign, self.make_span(start, self.byte_pos())));
        }
        if self.matches(":=") {
            self.advance_n(2);
            return Ok((Token::ConstAssign, self.make_span(start, self.byte_pos())));
        }
        if self.matches("..=") {
            self.advance_n(3);
            return Ok((Token::DotDotEq, self.make_span(start, self.byte_pos())));
        }
        if self.matches("..") {
            self.advance_n(2);
            return Ok((Token::DotDot, self.make_span(start, self.byte_pos())));
        }
        if self.matches("=>") {
            self.advance_n(2);
            return Ok((Token::FatArrow, self.make_span(start, self.byte_pos())));
        }
        if self.matches("->") {
            self.advance_n(2);
            return Ok((Token::Arrow, self.make_span(start, self.byte_pos())));
        }
        if self.matches("==") {
            self.advance_n(2);
            return Ok((Token::Eq, self.make_span(start, self.byte_pos())));
        }
        if self.matches("!=") {
            self.advance_n(2);
            return Ok((Token::NotEq, self.make_span(start, self.byte_pos())));
        }
        if self.matches("<=") {
            self.advance_n(2);
            return Ok((Token::LtEq, self.make_span(start, self.byte_pos())));
        }
        if self.matches(">=") {
            self.advance_n(2);
            return Ok((Token::GtEq, self.make_span(start, self.byte_pos())));
        }
        if self.matches("&&") {
            self.advance_n(2);
            return Ok((Token::And, self.make_span(start, self.byte_pos())));
        }
        if self.matches("||") {
            self.advance_n(2);
            return Ok((Token::Or, self.make_span(start, self.byte_pos())));
        }
        if self.matches("<<") {
            self.advance_n(2);
            return Ok((Token::ShiftLeft, self.make_span(start, self.byte_pos())));
        }
        if self.matches(">>") {
            self.advance_n(2);
            return Ok((Token::ShiftRight, self.make_span(start, self.byte_pos())));
        }

        // Single-char tokens
        self.advance();
        let end = self.byte_pos();
        let tok = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '!' => Token::Not,
            '&' => Token::BitAnd,
            '|' => Token::Pipe,
            '^' => Token::BitXor,
            '~' => Token::Tilde,
            '=' => Token::Assign,
            '.' => Token::Dot,
            '?' => Token::Question,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            _ => {
                return Err(CompileError::Syntax(
                    format!("unexpected character '{ch}'"),
                    Some(self.make_span(start, end)),
                ));
            }
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

    // ── Numbers ──────────────────────────────────────────────────

    pub(super) fn lex_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();

        // Check 0x / 0b / 0o prefixes
        if self.peek() == Some('0') {
            match self.peek_ahead(1) {
                Some('x') | Some('X') => return self.lex_prefixed_int(start, 16),
                Some('b') | Some('B') => return self.lex_prefixed_int(start, 2),
                Some('o') | Some('O') => return self.lex_prefixed_int(start, 8),
                _ => {}
            }
        }

        // Decimal digits
        let digits_start = self.pos;
        self.eat_digits(|c| c.is_ascii_digit());

        // Float: digit(s) followed by '.' followed by digit (not '..' range)
        if self.peek() == Some('.') && self.peek_ahead(1).is_some_and(|c| c.is_ascii_digit()) {
            self.advance(); // consume '.'
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

        // Integer
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

    /// Lex a 0x / 0b / 0o prefixed integer.
    fn lex_prefixed_int(&mut self, start: u32, radix: u32) -> Result<(Token, Span), CompileError> {
        self.advance_n(2); // skip prefix (0x / 0b / 0o)
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
