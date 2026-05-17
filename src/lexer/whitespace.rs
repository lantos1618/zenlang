use super::Lexer;
use crate::error::CompileError;

impl Lexer {
    /// Skip spaces, tabs, carriage returns and comments.
    /// Newlines are NOT consumed — they become Newline tokens.
    pub(super) fn skip_whitespace_and_comments(&mut self) -> Result<(), CompileError> {
        loop {
            // Skip horizontal whitespace
            while let Some(ch) = self.peek() {
                if ch == ' ' || ch == '\t' || ch == '\r' {
                    self.advance();
                } else {
                    break;
                }
            }

            // Line comment //
            if self.matches("//") {
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Block comment /* ... */ (nested)
            if self.matches("/*") {
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
            while let Some(ch) = self.peek() {
                if ch.is_ascii_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.matches("//") {
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            if self.matches("/*") {
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
                continue;
            }
            break;
        }
        Ok(())
    }
}
