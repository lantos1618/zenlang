use super::super::lexer::{Lexer, Token};
use crate::error::Span;
use crate::ast::Program;
use crate::error::{CompileError, Result};

pub struct Parser<'a> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) current_token: Token,
    pub(crate) peek_token: Token,
    pub(crate) current_span: Span,
    pub(crate) peek_span: Span,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token_with_span = lexer.next_token_with_span();
        let peek_token_with_span = lexer.next_token_with_span();
        Parser {
            lexer,
            current_token: current_token_with_span.token,
            peek_token: peek_token_with_span.token,
            current_span: current_token_with_span.span,
            peek_span: peek_token_with_span.span,
        }
    }

    pub fn next_token(&mut self) {
        let token_with_span = self.lexer.next_token_with_span();
        self.current_token = self.peek_token.clone();
        self.peek_token = token_with_span.token;
        self.current_span = self.peek_span.clone();
        self.peek_span = token_with_span.span;
    }

    pub fn debug_current_token(&self) -> &Token {
        &self.current_token
    }

    pub fn debug_peek_token(&self) -> &Token {
        &self.peek_token
    }
}
