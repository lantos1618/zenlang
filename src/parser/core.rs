use super::*;

pub(super) struct Parser {
    pub(super) tokens: Vec<(Token, Span)>,
    pub(super) pos: usize,
    #[allow(dead_code)]
    file_id: FileId,
    pub(super) errors: Vec<CompileError>,
    pub(super) loop_controls: Vec<(String, String)>,
    next_loop_control_id: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<(Token, Span)>, file_id: FileId) -> Self {
        Self {
            tokens,
            pos: 0,
            file_id,
            errors: Vec::new(),
            loop_controls: Vec::new(),
            next_loop_control_id: 0,
        }
    }

    pub(super) fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|(t, _)| t)
            .unwrap_or(&Token::EOF)
    }

    pub(super) fn loop_control_label(&self, name: &str) -> Option<String> {
        self.loop_controls
            .iter()
            .rev()
            .find(|(control_name, _)| control_name == name)
            .map(|(_, label)| label.clone())
    }

    pub(super) fn fresh_loop_control_label(&mut self) -> String {
        let id = self.next_loop_control_id;
        self.next_loop_control_id += 1;
        format!("__zen_loop_{}", id)
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

    /// Skip tokens until we find something that looks like a new declaration.
    pub(super) fn synchronize(&mut self) {
        loop {
            match self.peek() {
                Token::EOF => return,
                Token::Newline => {
                    self.advance();
                    match self.peek() {
                        Token::Identifier(_) | Token::Pub | Token::LBrace | Token::EOF => return,
                        _ => {}
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(super) fn parse_program(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_eof() {
                break;
            }
            match self.parse_declaration() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        decls
    }

    /// Check if current `{` starts an import: `{ name, ... } = module`.
    pub(super) fn is_import(&self) -> bool {
        let mut i = self.pos + 1;
        let mut depth = 1u32;
        loop {
            match self.tokens.get(i).map(|(t, _)| t) {
                Some(Token::LBrace) => {
                    depth += 1;
                    i += 1;
                }
                Some(Token::RBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                            i += 1;
                        }
                        return matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Assign));
                    }
                    i += 1;
                }
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    /// After seeing `Name:`, check if this is a struct def (next significant token is `{`).
    pub(super) fn is_struct_def(&self) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::LBrace))
    }

    /// After seeing `Name:`, check if this is an enum def (next significant token is an identifier).
    pub(super) fn is_enum_def(&self) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(_))
        )
    }

    pub(super) fn colon_is_followed_by_identifier(&self, expected: &str) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(name)) if name == expected
        )
    }

    /// Check if current `{` starts a struct destructuring pattern (not a block body).
    pub(super) fn is_struct_pattern(&self) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        match self.tokens.get(i).map(|(t, _)| t) {
            Some(Token::Identifier(_)) => {
                i += 1;
                while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                    i += 1;
                }
                matches!(
                    self.tokens.get(i).map(|(t, _)| t),
                    Some(Token::Comma) | Some(Token::Colon)
                )
            }
            _ => false,
        }
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

pub(super) enum StmtOrExpr {
    Stmt(Statement),
    Expr(Expression),
}
