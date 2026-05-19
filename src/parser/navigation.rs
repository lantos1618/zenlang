use super::*;

impl Parser {
    pub(super) fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|(t, _)| t)
            .unwrap_or(&Token::EOF)
    }

    pub(super) fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| *s)
            .unwrap_or_else(Span::dummy)
    }

    /// Peek at the next non-newline token without consuming.
    pub(super) fn peek_skip_newlines(&self) -> &Token {
        let mut i = self.pos;
        loop {
            match self.tokens.get(i) {
                Some((Token::Newline, _)) => i += 1,
                Some((t, _)) => return t,
                None => return &Token::EOF,
            }
        }
    }

    /// Look ahead n significant (non-newline) tokens.
    pub(super) fn peek_ahead(&self, n: usize) -> &Token {
        let mut count = 0;
        let mut i = self.pos;
        loop {
            match self.tokens.get(i) {
                Some((Token::Newline, _)) => i += 1,
                Some((t, _)) => {
                    if count == n {
                        return t;
                    }
                    count += 1;
                    i += 1;
                }
                None => return &Token::EOF,
            }
        }
    }

    pub(super) fn advance(&mut self) -> (Token, Span) {
        let entry = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or((Token::EOF, Span::dummy()));
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        entry
    }

    pub(super) fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    pub(super) fn expect(&mut self, expected: &Token) -> Result<Span, CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(span)
        } else {
            Err(CompileError::Syntax(
                format!("expected {:?}, found {:?}", expected, tok),
                Some(span),
            ))
        }
    }

    /// Expect a `>` token, splitting `>>` (ShiftRight) if needed for nested generics.
    pub(super) fn expect_gt(&mut self) -> Result<Span, CompileError> {
        self.skip_newlines();
        let (tok, span) = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or((Token::EOF, Span::dummy()));
        match tok {
            Token::Gt => {
                self.pos += 1;
                Ok(span)
            }
            Token::ShiftRight => {
                // Split `>>` into `>` + `>`: consume first `>`, leave second in stream.
                let first_span = Span::new(span.file_id, span.start, span.start + 1);
                let second_span = Span::new(span.file_id, span.start + 1, span.end);
                self.tokens[self.pos] = (Token::Gt, second_span);
                Ok(first_span)
            }
            _ => Err(CompileError::Syntax(
                format!("expected `>`, found {:?}", tok),
                Some(span),
            )),
        }
    }

    pub(super) fn expect_identifier(&mut self) -> Result<(String, Span), CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        match tok {
            Token::Identifier(name) => Ok((name, span)),
            _ => Err(CompileError::Syntax(
                format!("expected identifier, found {:?}", tok),
                Some(span),
            )),
        }
    }

    pub(super) fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    pub(super) fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].1
        } else {
            Span::dummy()
        }
    }

    /// Skip newlines only if the next non-newline token is a continuation
    /// (operator, dot, etc.), preserving newlines that separate statements.
    pub(super) fn skip_newlines_if_continuation(&mut self) {
        if !matches!(self.peek(), Token::Newline) {
            return;
        }
        let next = self.peek_skip_newlines();
        match next {
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Eq
            | Token::NotEq
            | Token::Lt
            | Token::Gt
            | Token::LtEq
            | Token::GtEq
            | Token::And
            | Token::Or
            | Token::BitAnd
            | Token::BitXor
            | Token::ShiftLeft
            | Token::ShiftRight
            | Token::Dot
            | Token::Question
            | Token::Pipe
            | Token::DotDot
            | Token::DotDotEq
            | Token::LBracket => {
                self.skip_newlines();
            }
            _ => {}
        }
    }
}
