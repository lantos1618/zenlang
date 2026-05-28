use super::*;

pub(super) struct Parser {
    pub(super) tokens: Vec<(Token, Span)>,
    pub(super) pos: usize,
    pub(super) errors: Vec<CompileError>,
    pub(super) loop_controls: Vec<(String, String)>,
    next_loop_control_id: usize,
}

pub(super) struct ParserCheckpoint {
    pos: usize,
    tokens: Vec<(Token, Span)>,
}

impl Parser {
    pub(super) fn new(tokens: Vec<(Token, Span)>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            loop_controls: Vec::new(),
            next_loop_control_id: 0,
        }
    }

    pub(super) fn loop_control_label(&self, name: &str) -> Option<String> {
        self.loop_controls
            .iter()
            .rev()
            .find_map(|(control_name, label)| (control_name == name).then(|| label.clone()))
    }

    pub(super) fn fresh_loop_control_label(&mut self) -> String {
        let id = self.next_loop_control_id;
        self.next_loop_control_id += 1;
        format!("__zen_loop_{}", id)
    }

    pub(super) fn checkpoint(&self) -> ParserCheckpoint {
        ParserCheckpoint {
            pos: self.pos,
            tokens: self.tokens.clone(),
        }
    }

    pub(super) fn restore(&mut self, checkpoint: ParserCheckpoint) {
        self.pos = checkpoint.pos;
        self.tokens = checkpoint.tokens;
    }

    pub(super) fn generic_close_has_attached_suffix_from(&self, start: usize) -> bool {
        let mut depth = 0usize;
        for index in start..self.tokens.len() {
            let (token, span) = &self.tokens[index];
            match token {
                Token::Lt => depth += 1,
                Token::Gt | Token::ShiftRight => {
                    let close_count = if matches!(token, Token::Gt) {
                        1
                    } else {
                        2
                    };
                    depth = depth.saturating_sub(close_count);
                    if depth == 0 {
                        return self.token_has_attached_generic_suffix(index, *span);
                    }
                }
                Token::EOF | Token::Newline | Token::RParen | Token::RBrace => return false,
                _ => {}
            }
        }
        false
    }

    fn token_has_attached_generic_suffix(&self, index: usize, close_span: Span) -> bool {
        let Some((next, next_span)) = self.tokens.get(index + 1) else {
            return false;
        };
        match next {
            Token::LParen | Token::Dot => next_span.start == close_span.end,
            Token::LBrace => true,
            _ => false,
        }
    }

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
}

pub(super) enum StmtOrExpr {
    Stmt(Statement),
    Expr(Expression),
}
