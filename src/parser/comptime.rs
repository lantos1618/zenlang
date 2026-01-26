use super::core::Parser;
use crate::ast::{Expression, Statement};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    /// Parse a comptime block: comptime { ... }
    #[allow(dead_code)]
    pub fn parse_comptime_block(&mut self) -> Result<Statement> {
        let span = Some(self.current_span.clone());
        // Expect 'comptime' keyword (already consumed)

        // Expect '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' after 'comptime'".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        // Parse statements inside the comptime block
        let mut statements = Vec::new();

        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            // Check for import attempts inside comptime blocks
            if let Token::Identifier(_name) = &self.current_token {
                if self.peek_token == Token::Operator(":=".to_string()) {
                    // Look ahead to see if this is an import
                    let saved_current = self.current_token.clone();
                    let saved_peek = self.peek_token.clone();
                    let saved_state = self.lexer.save_state();

                    self.next_token(); // Move to :=
                    self.next_token(); // Move past :=

                    let is_import = if let Token::Identifier(id) = &self.current_token {
                        // Check if it's an import (@std, @compiler, build)
                        if id.starts_with("@") {
                            // Any @ import is not allowed in comptime
                            true
                        } else if id == "build" {
                            // Need to check if it's build.import
                            let next_saved_current = self.current_token.clone();
                            let next_saved_peek = self.peek_token.clone();
                            self.next_token();
                            let is_build_import = self.current_token == Token::Symbol('.')
                                && {
                                    self.next_token();
                                    matches!(&self.current_token, Token::Identifier(name) if name == "import")
                                };
                            // Restore for proper error handling
                            self.current_token = next_saved_current;
                            self.peek_token = next_saved_peek;
                            is_build_import
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Restore state
                    self.current_token = saved_current;
                    self.peek_token = saved_peek;
                    self.lexer.restore_state(saved_state);

                    if is_import {
                        return Err(CompileError::SyntaxError(
                            "Import statements are not allowed inside comptime blocks. Move imports to module level.".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                }
            }

            // Parse regular statement
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Skip optional semicolons
            if self.current_token == Token::Symbol(';') {
                self.next_token();
            }
        }

        // Expect '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close comptime block".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        Ok(Statement::ComptimeBlock { statements, span })
    }

    /// Parse a comptime expression: comptime expr
    #[allow(dead_code)]
    pub fn parse_comptime_expression(&mut self) -> Result<Expression> {
        // 'comptime' keyword already consumed
        let expr = self.parse_expression()?;
        Ok(Expression::Comptime(Box::new(expr)))
    }

    /// Check if we're in a comptime context and handle accordingly
    #[allow(dead_code)]
    pub fn parse_comptime(&mut self) -> Result<Statement> {
        // Check what follows 'comptime'
        if self.peek_token == Token::Symbol('{') {
            self.next_token(); // consume 'comptime'
            self.parse_comptime_block()
        } else {
            // It's a comptime expression, wrap it in a statement
            let span = self.current_span.clone();
            self.next_token(); // consume 'comptime'
            let expr = self.parse_comptime_expression()?;
            Ok(Statement::Expression {
                expr,
                span: Some(span),
            })
        }
    }
}
