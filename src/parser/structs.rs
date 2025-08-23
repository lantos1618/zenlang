use super::core::Parser;
use crate::ast::{StructDefinition, StructField, AstType, Function};
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

        // Parse generics if present: <T, U, ...>
        let mut generics = Vec::new();
        if self.current_token == Token::Operator("<".to_string()) {
            self.next_token();
            loop {
                if let Token::Identifier(gen) = &self.current_token {
                    generics.push(gen.clone());
                    self.next_token();
                    
                    if self.current_token == Token::Operator(">".to_string()) {
                        self.next_token();
                        break;
                    } else if self.current_token == Token::Symbol(',') {
                        self.next_token();
                    } else {
                        return Err(CompileError::SyntaxError(
                            "Expected ',' or '>' in generic parameters".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected generic parameter name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            }
        }

        // Expect and consume '='
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

        // Parse methods (zero or more fn ...)
        // TODO: Implement method parsing with correct syntax
        let methods = Vec::new();

        Ok(StructDefinition { name, generics, fields, methods })
    }

    fn parse_method(&mut self) -> Result<Function> {
        // Method name (after 'fn' keyword)
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected method name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();

        // Parameters in parentheses
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError(
                "Expected '(' for method parameters".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        let mut parameters = Vec::new();
        while self.current_token != Token::Symbol(')') {
            // Parameter name
            let param_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError(
                    "Expected parameter name".to_string(),
                    Some(self.current_span.clone()),
                ));
            };
            self.next_token();

            // Parameter type (optional - if next token is ')' or ',', skip type parsing)
            let param_type = if self.current_token == Token::Symbol(')') || self.current_token == Token::Symbol(',') {
                // No explicit type, use a default (could be inferred later)
                AstType::I32 // Default type for now
            } else {
                self.parse_type()?
            };
            parameters.push((param_name, param_type));

            // Comma or closing parenthesis
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol(')') {
                return Err(CompileError::SyntaxError(
                    "Expected ',' or ')' after parameter".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
        }
        self.next_token(); // consume ')'

        // Return type
        let return_type = self.parse_type()?;

        // Function body
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' for method body".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        let mut body = Vec::new();
        while self.current_token != Token::Symbol('}') {
            body.push(self.parse_statement()?);
        }
        self.next_token(); // consume '}'

        Ok(Function {
            name,
            args: parameters,
            return_type,
            body,
            is_async: false, // Methods are not async for now
        })
    }
}
