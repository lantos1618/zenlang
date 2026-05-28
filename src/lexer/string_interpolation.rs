use super::{Lexer, Token};
use crate::error::{CompileError, Span};

impl Lexer {
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
                        self.push_current_char_token(buf, Token::InterpolationEnd);
                        return Ok(());
                    }
                    self.push_current_char_token(buf, Token::RBrace);
                }
                Some('{') => {
                    depth += 1;
                    self.push_current_char_token(buf, Token::LBrace);
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

    fn lex_next_no_skip(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.byte_pos();

        let Some(ch) = self.peek() else {
            return Ok((Token::EOF, self.make_span(start, start)));
        };

        self.lex_non_string_token(start, ch)
    }

    fn push_current_char_token(&mut self, buf: &mut Vec<(Token, Span)>, token: Token) {
        let start = self.byte_pos();
        self.advance();
        buf.push((token, self.make_span(start, self.byte_pos())));
    }
}
