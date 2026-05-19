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
}

pub(super) enum StmtOrExpr {
    Stmt(Statement),
    Expr(Expression),
}
