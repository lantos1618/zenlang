// Tokenization and parsing logic
use super::DocumentStore;
use crate::ast::Declaration;
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;

impl DocumentStore {
    pub(super) fn tokenize(&self, content: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(content);
        let mut tokens = Vec::new();

        // Collect all tokens
        loop {
            let token = lexer.next_token();
            if matches!(token, Token::Eof) {
                break;
            }
            tokens.push(token);
        }

        tokens
    }

    pub(super) fn parse_with_path(
        &self,
        content: &str,
        file_path: Option<&str>,
    ) -> Option<Vec<Declaration>> {
        let lexer = Lexer::new(content);
        let mut parser = Parser::new(lexer);

        // Try normal parsing first
        match parser.parse_program() {
            Ok(program) => Some(program.declarations),
            Err(e) => {
                if let Some(path) = file_path {
                    log::debug!("[LSP] Parse error in {}: {:?}", path, e);
                } else {
                    log::debug!("[LSP] Parse error: {:?}", e);
                }

                // Use error recovery to get partial AST
                // Re-create parser since we consumed it
                let lexer = Lexer::new(content);
                let mut parser = Parser::new(lexer);
                let (program, errors) = parser.parse_program_with_recovery();

                if !errors.is_empty() {
                    log::debug!("[LSP] Recovery parsing found {} errors", errors.len());
                }

                // Return partial AST if we got any declarations
                if program.declarations.is_empty() {
                    None
                } else {
                    log::debug!(
                        "[LSP] Recovery parsing extracted {} declarations",
                        program.declarations.len()
                    );
                    Some(program.declarations)
                }
            }
        }
    }

    pub(super) fn parse(&self, content: &str) -> Option<Vec<Declaration>> {
        self.parse_with_path(content, None)
    }
}
