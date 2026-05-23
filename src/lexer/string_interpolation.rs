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

        self.lex_non_string_token(start, ch)
    }
}
