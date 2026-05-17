use super::*;

impl Parser {
    pub(super) fn parse_import(&mut self) -> Result<Declaration, CompileError> {
        let start = self.peek_span();

        // { name1, name2 }
        self.expect(&Token::LBrace)?;
        let mut names = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let (name, _) = self.expect_identifier()?;
            names.push(name);
            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();

        // module path: std, std.io, @std.io, @builtin
        let module_path = self.parse_module_path()?;
        let span = start.merge(self.prev_span());

        Ok(Declaration::Import {
            names,
            module_path,
            span,
        })
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>, CompileError> {
        let mut path = Vec::new();

        match self.peek().clone() {
            Token::AtStd => {
                self.advance();
                path.push("@std".to_string());
            }
            Token::AtBuiltin => {
                self.advance();
                path.push("@builtin".to_string());
            }
            Token::Identifier(name) => {
                self.advance();
                path.push(name);
            }
            _ => {
                return Err(CompileError::Syntax(
                    format!("expected module path, found {:?}", self.peek()),
                    Some(self.peek_span()),
                ));
            }
        }

        while matches!(self.peek(), Token::Dot) {
            self.advance();
            let (seg, _) = self.expect_identifier()?;
            path.push(seg);
        }

        Ok(path)
    }
}
