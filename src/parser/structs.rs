use super::core::Parser;
use crate::ast::{StructDefinition, StructField, AstType, Expression};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_struct(&mut self) -> Result<StructDefinition> {
        // Struct name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected struct name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Skip the '=' operator
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after struct name".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Opening brace
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' for struct fields".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut fields = vec![];
        
        // Parse fields
        while self.current_token != Token::Symbol('}') {
            if self.current_token == Token::Eof {
                return Err(CompileError::SyntaxError(
                    "Unexpected end of file in struct definition".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            
            // Field name
            let field_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError(
                    "Expected field name".to_string(),
                    Some(self.current_span.clone()),
                ));
            };
            self.next_token();
            
            // Colon
            if self.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError(
                    "Expected ':' after field name".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            
            // Field type
            let field_type = self.parse_type()?;
            
            // Check for mutability modifier (:: for mutable)
            let is_mutable = if self.current_token == Token::Operator("::".to_string()) {
                self.next_token();
                true
            } else {
                false
            };
            
            // Optional default value
            let default_value = if self.current_token == Token::Operator("=".to_string()) {
                self.next_token();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            fields.push(StructField {
                name: field_name,
                type_: field_type,
                is_mutable,
                default_value,
            });
            
            // Comma separator (except for last field)
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol('}') {
                return Err(CompileError::SyntaxError(
                    "Expected ',' or '}' after field".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
        }
        
        // Closing brace
        self.next_token();
        
        Ok(StructDefinition { name, fields })
    }
}
