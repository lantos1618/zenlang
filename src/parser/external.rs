use super::core::Parser;
use crate::ast::ExternalFunction;
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_external_function(&mut self) -> Result<ExternalFunction> {
        // External function name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected external function name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Skip the '=' operator
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after external function name".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parameters
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError(
                "Expected '(' for external function parameters".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut args = vec![];
        let mut is_varargs = false;
        
        if self.current_token != Token::Symbol(')') {
            loop {
                if self.current_token == Token::Operator("...".to_string()) {
                    is_varargs = true;
                    self.next_token();
                    break;
                }
                
                // Check if we have a parameter name (optional)
                if let Token::Identifier(_param_name) = &self.current_token {
                    // Check if the next token is ':'
                    if self.peek_token == Token::Symbol(':') {
                        // Skip the parameter name and ':'
                        self.next_token(); // skip param name
                        self.next_token(); // skip ':'
                    }
                }
                
                let arg_type = self.parse_type()?;
                args.push(arg_type);
                
                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if self.current_token != Token::Symbol(',') {
                    return Err(CompileError::SyntaxError(
                        "Expected ',' or ')' in external function parameter list".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
            }
        }
        self.next_token(); // consume ')'
        
        // Return type
        let return_type = self.parse_type()?;
        
        Ok(ExternalFunction {
            name,
            args,
            return_type,
            is_varargs,
        })
    }
}
