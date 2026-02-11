use super::{Lexer, Token};
use crate::error::{CompileError, Span};

// ── String literal and interpolation scanning ────────────────────

impl Lexer {
    pub(super) fn lex_string(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();
        self.advance(); // consume opening "

        let mut buf: Vec<(Token, Span)> = Vec::new();
        let mut text = String::new();
        let mut chunk_start = self.byte_pos();
        let mut has_interpolation = false;

        loop {
            match self.peek() {
                None => {
                    return Err(CompileError::Syntax(
                        "unterminated string literal".into(),
                        Some(self.make_span(start, self.byte_pos())),
                    ));
                }
                Some('"') => {
                    self.advance(); // closing "
                    break;
                }
                Some('\\') => {
                    self.advance(); // consume backslash
                    match self.peek() {
                        None => {
                            return Err(CompileError::Syntax(
                                "unterminated escape sequence".into(),
                                Some(self.make_span(start, self.byte_pos())),
                            ));
                        }
                        Some('n') => {
                            self.advance();
                            text.push('\n');
                        }
                        Some('t') => {
                            self.advance();
                            text.push('\t');
                        }
                        Some('r') => {
                            self.advance();
                            text.push('\r');
                        }
                        Some('\\') => {
                            self.advance();
                            text.push('\\');
                        }
                        Some('"') => {
                            self.advance();
                            text.push('"');
                        }
                        Some('0') => {
                            self.advance();
                            text.push('\0');
                        }
                        Some('$') => {
                            self.advance();
                            text.push('$');
                        }
                        Some('x') => {
                            self.advance(); // consume 'x'
                            let h1 = self.advance().unwrap_or('0');
                            let h2 = self.advance().unwrap_or('0');
                            let hex = format!("{h1}{h2}");
                            let code = u8::from_str_radix(&hex, 16).unwrap_or(0);
                            text.push(code as char);
                        }
                        Some(other) => {
                            self.advance();
                            text.push('\\');
                            text.push(other);
                        }
                    }
                }
                Some('$') if self.peek_ahead(1) == Some('{') => {
                    has_interpolation = true;

                    // Emit accumulated text as StringChunk
                    if !text.is_empty() {
                        buf.push((
                            Token::StringChunk(std::mem::take(&mut text)),
                            self.make_span(chunk_start, self.byte_pos()),
                        ));
                    }

                    // Emit InterpolationStart
                    let interp_start = self.byte_pos();
                    self.advance_n(2); // skip ${
                    buf.push((
                        Token::InterpolationStart,
                        self.make_span(interp_start, self.byte_pos()),
                    ));

                    // Lex tokens inside interpolation until matching }
                    self.lex_interpolation_body(&mut buf)?;

                    chunk_start = self.byte_pos();
                }
                Some(ch) => {
                    self.advance();
                    text.push(ch);
                }
            }
        }

        if !has_interpolation {
            // Simple string — single StringLiteral token
            Ok((
                Token::StringLiteral(text),
                self.make_span(start, self.byte_pos()),
            ))
        } else {
            // Has interpolation — push remaining text chunk and queue all via pending
            if !text.is_empty() {
                buf.push((
                    Token::StringChunk(text),
                    self.make_span(chunk_start, self.byte_pos()),
                ));
            }
            // Return first token, store rest in pending buffer
            let first = buf.remove(0);
            self.pending = buf;
            Ok(first)
        }
    }

    /// Lex tokens inside `${...}`, tracking brace depth.
    /// Pushes all tokens (including the closing InterpolationEnd) into `buf`.
    fn lex_interpolation_body(&mut self, buf: &mut Vec<(Token, Span)>) -> Result<(), CompileError> {
        let mut depth = 1u32;

        loop {
            // Inside interpolation, newlines are insignificant
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
                    // Nested } — emit as RBrace
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
                    // Nested string inside interpolation — lex it recursively.
                    // lex_string may set self.pending, so we save and restore.
                    let saved_pending = std::mem::take(&mut self.pending);
                    let first = self.lex_string()?;
                    buf.push(first);
                    buf.append(&mut self.pending);
                    self.pending = saved_pending;
                }
                _ => {
                    // Use lex_next for a regular token (skip_whitespace already done)
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
    /// Used inside interpolation body where we already skipped whitespace.
    fn lex_next_no_skip(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok((Token::EOF, self.make_span(start, start))),
        };

        // Identifiers / keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.lex_identifier());
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.lex_number();
        }

        // @ tokens
        if ch == '@' {
            return self.lex_at_token();
        }

        // Multi-char operators
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

        // Single-char
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
}
