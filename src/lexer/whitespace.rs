use super::Lexer;
use crate::error::CompileError;

impl Lexer {
    /// Skip spaces, tabs, carriage returns and comments.
    /// Newlines are NOT consumed — they become Newline tokens.
    pub(super) fn skip_whitespace_and_comments(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_chars_while(|ch| matches!(ch, ' ' | '\t' | '\r'));
            if self.skip_comment()? {
                continue;
            }
            break;
        }
        Ok(())
    }

    /// Skip ALL whitespace (including newlines) and comments.
    /// Used inside string interpolation where newlines are insignificant.
    pub(super) fn skip_all_whitespace_and_comments(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_chars_while(char::is_ascii_whitespace);
            if self.skip_comment()? {
                continue;
            }
            break;
        }
        Ok(())
    }

    fn skip_chars_while(&mut self, mut should_skip: impl FnMut(&char) -> bool) {
        while let Some(ch) = self.peek() {
            if should_skip(&ch) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) -> Result<bool, CompileError> {
        if self.matches("//") {
            self.skip_line_comment();
            return Ok(true);
        }

        if self.matches("/*") {
            self.skip_block_comment()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), CompileError> {
        let comment_start = self.byte_pos();
        self.advance_n(2);
        let mut depth = 1u32;

        while depth > 0 {
            match self.peek() {
                None => {
                    return Err(CompileError::Syntax(
                        "unterminated block comment".into(),
                        Some(self.make_span(comment_start, self.byte_pos())),
                    ));
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    self.advance_n(2);
                    depth += 1;
                }
                Some('*') if self.peek_ahead(1) == Some('/') => {
                    self.advance_n(2);
                    depth -= 1;
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(())
    }
}
