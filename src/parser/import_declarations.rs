use super::*;
use crate::root_spelling::{AT_BUILTIN_ROOT, AT_STD_ROOT};

impl Parser {
    pub(super) fn parse_import(&mut self) -> Result<Declaration, CompileError> {
        let start = self.peek_span();

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
            self.consume_comma();
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();

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
                path.push(AT_STD_ROOT.to_string());
            }
            Token::AtBuiltin => {
                self.advance();
                path.push(AT_BUILTIN_ROOT.to_string());
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
