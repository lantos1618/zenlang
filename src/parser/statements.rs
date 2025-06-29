use super::core::Parser;
use crate::ast::{Program, Declaration, Statement, VariableDeclarationType, Expression, AstType};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = vec![];
        while self.current_token != Token::Eof {
            // Parse top-level declarations
            if let Token::Identifier(_) = &self.current_token {
                // Could be a function definition: name = (params) returnType { ... }
                if self.peek_token == Token::Operator("=".to_string()) {
                    // Check if it's a struct or enum definition
                    let _name = if let Token::Identifier(name) = &self.current_token {
                        name.clone()
                    } else {
                        unreachable!()
                    };
                    
                    // Look ahead to see if it's a struct/enum or function
                    let saved_position = self.lexer.position;
                    let saved_read_position = self.lexer.read_position;
                    let saved_current_char = self.lexer.current_char;
                    let saved_current_token = self.current_token.clone();
                    let saved_peek_token = self.peek_token.clone();
                    
                    self.next_token(); // consume '='
                    self.next_token(); // consume '('
                    
                    // If we see a type after '(', it's likely a function
                    // If we see a field name, it's likely a struct
                    let is_function = if let Token::Identifier(_) = &self.current_token {
                        // Look for ': type' pattern
                        let has_type_annotation = self.peek_token == Token::Symbol(':');
                        has_type_annotation
                    } else {
                        false
                    };
                    
                    // Restore lexer state
                    self.lexer.position = saved_position;
                    self.lexer.read_position = saved_read_position;
                    self.lexer.current_char = saved_current_char;
                    self.current_token = saved_current_token;
                    self.peek_token = saved_peek_token;
                    
                    if is_function {
                        declarations.push(Declaration::Function(self.parse_function()?));
                    } else {
                        // For now, assume it's a function
                        declarations.push(Declaration::Function(self.parse_function()?));
                    }
                } else if self.peek_token == Token::Symbol('(') {
                    // Could be an external function declaration
                    declarations.push(Declaration::ExternalFunction(self.parse_external_function()?));
                } else {
                    return Err(CompileError::SyntaxError(
                        format!("Unexpected token after identifier: {:?}", self.peek_token),
                        Some(self.current_span.clone()),
                    ));
                }
            } else {
                return Err(CompileError::SyntaxError(
                    format!("Unexpected token at top level: {:?}", self.current_token),
                    Some(self.current_span.clone()),
                ));
            }
        }
        
        Ok(Program { declarations })
    }

    pub fn parse_statement(&mut self) -> Result<Statement> {
        match &self.current_token {
            Token::Identifier(name) => {
                // Check for variable declarations
                match &self.peek_token {
                    Token::Operator(op) if op == ":=" || op == "::=" => {
                        self.parse_variable_declaration()
                    }
                    Token::Symbol(':') => {
                        // Check for explicit type declarations
                        let name_clone = name.clone();
                        let saved_position = self.lexer.position;
                        let saved_read_position = self.lexer.read_position;
                        let saved_current_char = self.lexer.current_char;
                        let saved_current_token = self.current_token.clone();
                        let saved_peek_token = self.peek_token.clone();
                        
                        self.next_token(); // consume ':'
                        
                        // Try to parse a type
                        let type_result = self.parse_type();
                        
                        // Restore lexer state
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.current_token = saved_current_token;
                        self.peek_token = saved_peek_token;
                        
                        if type_result.is_ok() && self.peek_token == Token::Operator("=".to_string()) {
                            self.parse_variable_declaration()
                        } else {
                            // Not a variable declaration, treat as expression
                            let expr = self.parse_expression()?;
                            if self.current_token == Token::Symbol(';') {
                                self.next_token();
                            }
                            Ok(Statement::Expression(expr))
                        }
                    }
                    _ => {
                        // Not a variable declaration, treat as expression
                        let expr = self.parse_expression()?;
                        if self.current_token == Token::Symbol(';') {
                            self.next_token();
                        }
                        Ok(Statement::Expression(expr))
                    }
                }
            }
            Token::Keyword(keyword) if keyword == "return" => {
                self.next_token();
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Return(expr))
            }
            // Handle literal expressions as valid statements
            Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) => {
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
            _ => Err(CompileError::SyntaxError(
                format!("Unexpected token in statement: {:?}", self.current_token),
                Some(self.current_span.clone()),
            )),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected variable name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        let (is_mutable, declaration_type, type_) = match &self.current_token {
            Token::Operator(op) if op == ":=" => {
                // Inferred immutable: name := value
                self.next_token();
                (false, VariableDeclarationType::InferredImmutable, None)
            }
            Token::Operator(op) if op == "::=" => {
                // Inferred mutable: name ::= value
                self.next_token();
                (true, VariableDeclarationType::InferredMutable, None)
            }
            Token::Symbol(':') => {
                // Explicit immutable: name : T = value
                self.next_token();
                let type_ = self.parse_type()?;
                if self.current_token != Token::Operator("=".to_string()) {
                    return Err(CompileError::SyntaxError("Expected '=' after type".to_string(), Some(self.current_span.clone())));
                }
                self.next_token();
                (false, VariableDeclarationType::ExplicitImmutable, Some(type_))
            }
            Token::Operator(op) if op == "::" => {
                // Explicit mutable: name :: T = value
                self.next_token();
                let type_ = self.parse_type()?;
                if self.current_token != Token::Operator("=".to_string()) {
                    return Err(CompileError::SyntaxError("Expected '=' after type".to_string(), Some(self.current_span.clone())));
                }
                self.next_token();
                (true, VariableDeclarationType::ExplicitMutable, Some(type_))
            }
            _ => {
                return Err(CompileError::SyntaxError(
                    format!("Expected variable declaration operator, got: {:?}", self.current_token),
                    Some(self.current_span.clone())
                ));
            }
        };
        
        let initializer = self.parse_expression()?;
        
        // Infer type from initializer if not explicitly specified
        let inferred_type = if type_.is_none() {
            match &initializer {
                Expression::Integer8(_) => Some(AstType::I8),
                Expression::Integer16(_) => Some(AstType::I16),
                Expression::Integer32(_) => Some(AstType::I32),
                Expression::Integer64(_) => Some(AstType::I64),
                Expression::Unsigned8(_) => Some(AstType::U8),
                Expression::Unsigned16(_) => Some(AstType::U16),
                Expression::Unsigned32(_) => Some(AstType::U32),
                Expression::Unsigned64(_) => Some(AstType::U64),
                Expression::Float32(_) => Some(AstType::F32),
                Expression::Float64(_) => Some(AstType::F64),
                Expression::Boolean(_) => Some(AstType::Bool),
                Expression::String(_) => Some(AstType::String),
                _ => None, // Can't infer type for other expressions
            }
        } else {
            type_
        };
        
        if self.current_token == Token::Symbol(';') {
            self.next_token();
        }
        
        Ok(Statement::VariableDeclaration {
            name,
            type_: inferred_type,
            initializer: Some(initializer),
            is_mutable,
            declaration_type,
        })
    }
}
