mod scan;
mod strings;
mod tokens;
mod whitespace;

pub use tokens::Token;

use crate::error::{CompileError, FileId, Span};

// ── Lexer ─────────────────────────────────────────────────────────

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    file_id: FileId,
    /// byte_offsets[i] = byte offset of source char i.
    /// Sentinel at [source.len()] = total byte length.
    byte_offsets: Vec<u32>,
    /// Buffered tokens from string interpolation.
    pending: Vec<(Token, Span)>,
}

impl Lexer {
    pub fn new(source: &str, file_id: FileId) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let byte_offsets = Self::build_byte_offsets(source, chars.len());
        Self {
            source: chars,
            pos: 0,
            file_id,
            byte_offsets,
            pending: Vec::new(),
        }
    }

    /// Tokenise the entire source, returning tokens paired with spans.
    pub fn tokenize(&mut self) -> Result<Vec<(Token, Span)>, CompileError> {
        let mut tokens = Vec::new();
        loop {
            let (tok, span) = self.next_token()?;
            let done = tok.is_eof();
            tokens.push((tok, span));
            if done {
                break;
            }
        }
        Ok(tokens)
    }

    /// Return the next token (drains pending buffer first).
    pub fn next_token(&mut self) -> Result<(Token, Span), CompileError> {
        // Drain buffered tokens from string interpolation
        if !self.pending.is_empty() {
            return Ok(self.pending.remove(0));
        }
        self.lex_next()
    }

    // ── Character helpers ────────────────────────────────────────

    fn build_byte_offsets(source: &str, char_count: usize) -> Vec<u32> {
        let mut offsets = Vec::with_capacity(char_count + 1);
        for (byte_idx, _) in source.char_indices() {
            offsets.push(byte_idx as u32);
        }
        offsets.push(source.len() as u32); // sentinel for EOF
        offsets
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn advance_n(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.source.len());
    }

    fn byte_pos(&self) -> u32 {
        self.byte_offsets[self.pos.min(self.source.len())]
    }

    fn make_span(&self, start: u32, end: u32) -> Span {
        Span::new(self.file_id, start, end)
    }

    /// Check whether the source at `self.pos` starts with `s`.
    fn matches(&self, s: &str) -> bool {
        for (i, expected) in s.chars().enumerate() {
            match self.source.get(self.pos + i) {
                Some(&ch) if ch == expected => {}
                _ => return false,
            }
        }
        true
    }
}

// ── Convenience function ──────────────────────────────────────────

/// Tokenise source code into a list of (Token, Span) pairs.
pub fn tokenize(source: &str, file_id: FileId) -> Result<Vec<(Token, Span)>, CompileError> {
    Lexer::new(source, file_id).tokenize()
}

#[cfg(test)]
mod tests;
