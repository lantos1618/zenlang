use super::core::Parser;
use crate::ast::{Statement, Expression};
use crate::error::{CompileError, Result};
use crate::lexer::{Token, Keyword};

impl<'a> Parser<'a> {
    /// Parse a comptime block: comptime { ... }
    pub fn parse_comptime_block(&mut self) -> Result<Statement> {
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
        
        Ok(Statement::ComptimeBlock(statements))
    }
    
    /// Parse a comptime expression: comptime expr
    pub fn parse_comptime_expression(&mut self) -> Result<Expression> {
        // 'comptime' keyword already consumed
        let expr = self.parse_expression()?;
        Ok(Expression::Comptime(Box::new(expr)))
    }
    
    /// Check if we're in a comptime context and handle accordingly
    pub fn parse_comptime(&mut self) -> Result<Statement> {
        // Check what follows 'comptime'
        if self.peek_token() == Some(Token::Symbol('{')) {
            self.next_token(); // consume 'comptime'
            self.parse_comptime_block()
        } else {
            // It's a comptime expression, wrap it in a statement
            self.next_token(); // consume 'comptime'
            let expr = self.parse_comptime_expression()?;
            Ok(Statement::Expression(expr))
        }
    }
}
