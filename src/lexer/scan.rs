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
}
