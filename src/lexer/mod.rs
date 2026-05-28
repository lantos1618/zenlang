mod numbers;
mod scan;
mod string_interpolation;
mod strings;
mod tokens;
mod whitespace;

pub use tokens::Token;

use crate::error::{CompileError, FileId, Span};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    file_id: FileId,
    byte_offsets: Vec<u32>,
    pending: Vec<(Token, Span)>,
}

impl Lexer {
    pub fn new(source: &str, file_id: FileId) -> Self {
        let byte_offsets = source
            .char_indices()
            .map(|(byte_idx, _)| byte_idx as u32)
            .chain(std::iter::once(source.len() as u32))
            .collect();
        Self {
            source: source.chars().collect(),
            pos: 0,
            file_id,
            byte_offsets,
            pending: Vec::new(),
        }
    }

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

    pub fn next_token(&mut self) -> Result<(Token, Span), CompileError> {
        if !self.pending.is_empty() {
            return Ok(self.pending.remove(0));
        }
        self.lex_next()
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

pub fn tokenize(source: &str, file_id: FileId) -> Result<Vec<(Token, Span)>, CompileError> {
    Lexer::new(source, file_id).tokenize()
}
