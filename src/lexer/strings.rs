use super::{Lexer, Token};
use crate::error::{CompileError, Span};

impl Lexer {
    pub(super) fn lex_string(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();
        self.advance();

        let mut buf: Vec<(Token, Span)> = Vec::new();
        let mut text = String::new();
        let mut chunk_start = self.byte_pos();

        loop {
            match self.peek() {
                None => {
                    return Err(CompileError::Syntax(
                        "unterminated string literal".into(),
                        Some(self.make_span(start, self.byte_pos())),
                    ));
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => self.push_escaped_string_char(&mut text, start)?,
                Some('$') if self.peek_ahead(1) == Some('{') => {
                    if !text.is_empty() {
                        buf.push((
                            Token::StringChunk(std::mem::take(&mut text)),
                            self.make_span(chunk_start, self.byte_pos()),
                        ));
                    }

                    let interp_start = self.byte_pos();
                    self.advance_n(2);
                    buf.push((
                        Token::InterpolationStart,
                        self.make_span(interp_start, self.byte_pos()),
                    ));

                    self.lex_interpolation_body(&mut buf)?;

                    chunk_start = self.byte_pos();
                }
                Some(ch) => {
                    self.advance();
                    text.push(ch);
                }
            }
        }

        if buf.is_empty() {
            Ok((
                Token::StringLiteral(text),
                self.make_span(start, self.byte_pos()),
            ))
        } else {
            if !text.is_empty() {
                buf.push((
                    Token::StringChunk(text),
                    self.make_span(chunk_start, self.byte_pos()),
                ));
            }
            let first = buf.remove(0);
            self.pending = buf;
            Ok(first)
        }
    }

    fn push_escaped_string_char(
        &mut self,
        text: &mut String,
        string_start: u32,
    ) -> Result<(), CompileError> {
        self.advance();
        let Some(ch) = self.peek() else {
            return Err(CompileError::Syntax(
                "unterminated escape sequence".into(),
                Some(self.make_span(string_start, self.byte_pos())),
            ));
        };
        match ch {
            'n' => text.push('\n'),
            't' => text.push('\t'),
            'r' => text.push('\r'),
            '\\' => text.push('\\'),
            '"' => text.push('"'),
            '0' => text.push('\0'),
            '$' => text.push('$'),
            'x' => {
                self.advance();
                let hex = format!(
                    "{}{}",
                    self.advance().unwrap_or('0'),
                    self.advance().unwrap_or('0')
                );
                text.push(u8::from_str_radix(&hex, 16).unwrap_or(0) as char);
                return Ok(());
            }
            other => {
                text.push('\\');
                text.push(other);
            }
        }
        self.advance();
        Ok(())
    }
}
