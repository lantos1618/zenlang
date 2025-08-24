use super::core::Parser;
use crate::ast::{EnumDefinition, EnumVariant};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_enum(&mut self) -> Result<EnumDefinition> {
        // Enum name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected enum name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Skip the '=' operator
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after enum name".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut variants = vec![];
        
        // Parse variants
        while self.current_token != Token::Eof {
            // Each variant starts with |
            if self.current_token != Token::Symbol('|') {
                break;
            }
            self.next_token();
            
            // Variant name
            let variant_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError(
                    "Expected variant name".to_string(),
                    Some(self.current_span.clone()),
                ));
            };
            self.next_token();
            
            // Check for payload type
            let payload = if self.current_token == Token::Symbol('(') {
                self.next_token();
                
                // Check if this is a named field (field: type) or just a type
                if let Token::Identifier(_field_name) = &self.current_token {
                    // Look ahead to see if there's a colon
                    if self.peek_token == Token::Symbol(':') {
                        // Named field syntax - skip the field name for now
                        self.next_token(); // skip field name
                        self.next_token(); // skip ':'
                    }
                }
                
                // Parse payload type
                let payload_type = self.parse_type()?;
                
                // Closing parenthesis
                if self.current_token != Token::Symbol(')') {
                    return Err(CompileError::SyntaxError(
                        "Expected ')' after payload type".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
                
                Some(payload_type)
            } else {
                None
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                payload,
            });
        }
        
        if variants.is_empty() {
            return Err(CompileError::SyntaxError(
                "Enum must have at least one variant".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        
        Ok(EnumDefinition { name, type_params: Vec::new(), variants, methods: Vec::new() })
    }
}
