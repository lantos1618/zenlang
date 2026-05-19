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
}
