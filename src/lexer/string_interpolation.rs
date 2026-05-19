use super::{Lexer, Token};
use crate::error::{CompileError, Span};

impl Lexer {
    /// Lex tokens inside `${...}`, tracking brace depth.
    /// Pushes all tokens, including the closing `InterpolationEnd`, into `buf`.
    pub(super) fn lex_interpolation_body(
        &mut self,
        buf: &mut Vec<(Token, Span)>,
    ) -> Result<(), CompileError> {
        let mut depth = 1u32;

        loop {
            self.skip_all_whitespace_and_comments()?;

            match self.peek() {
                None => {
                    return Err(CompileError::Syntax(
                        "unterminated string interpolation".into(),
                        Some(self.make_span(self.byte_pos(), self.byte_pos())),
                    ));
                }
                Some('}') => {
                    depth -= 1;
                    if depth == 0 {
                        let s = self.byte_pos();
                        self.advance();
                        buf.push((Token::InterpolationEnd, self.make_span(s, self.byte_pos())));
                        return Ok(());
                    }
                    let s = self.byte_pos();
                    self.advance();
                    buf.push((Token::RBrace, self.make_span(s, self.byte_pos())));
                }
                Some('{') => {
                    depth += 1;
                    let s = self.byte_pos();
                    self.advance();
                    buf.push((Token::LBrace, self.make_span(s, self.byte_pos())));
                }
                Some('"') => {
                    let saved_pending = std::mem::take(&mut self.pending);
                    let first = self.lex_string()?;
                    buf.push(first);
                    buf.append(&mut self.pending);
                    self.pending = saved_pending;
                }
                _ => {
                    let (tok, span) = self.lex_next_no_skip()?;
                    if tok.is_eof() {
                        return Err(CompileError::Syntax(
                            "unterminated string interpolation".into(),
                            Some(span),
                        ));
                    }
                    buf.push((tok, span));
                }
            }
        }
    }

    /// Lex a single token without calling skip_whitespace first.
    /// Used inside interpolation bodies where whitespace has already been skipped.
    fn lex_next_no_skip(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok((Token::EOF, self.make_span(start, start))),
        };

        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.lex_identifier());
        }
        if ch.is_ascii_digit() {
            return self.lex_number();
        }
        if ch == '@' {
            return self.lex_at_token();
        }

        if let Some(tok) = self.lex_no_skip_multi_char_operator(start) {
            return Ok(tok);
        }

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

    fn lex_no_skip_multi_char_operator(&mut self, start: u32) -> Option<(Token, Span)> {
        for (spelling, token, width) in [
            ("::=", Token::DeclareAssign, 3),
            (":=", Token::ConstAssign, 2),
            ("..=", Token::DotDotEq, 3),
            ("..", Token::DotDot, 2),
            ("=>", Token::FatArrow, 2),
            ("->", Token::Arrow, 2),
            ("==", Token::Eq, 2),
            ("!=", Token::NotEq, 2),
            ("<=", Token::LtEq, 2),
            (">=", Token::GtEq, 2),
            ("&&", Token::And, 2),
            ("||", Token::Or, 2),
            ("<<", Token::ShiftLeft, 2),
            (">>", Token::ShiftRight, 2),
        ] {
            if self.matches(spelling) {
                self.advance_n(width);
                return Some((token, self.make_span(start, self.byte_pos())));
            }
        }
        None
    }
}
