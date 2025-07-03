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
                    // Check if it's a struct, enum, or function definition
                    let name = if let Token::Identifier(name) = &self.current_token {
                        name.clone()
                    } else {
                        unreachable!()
                    };
                    
                    // Look ahead to see what type of declaration this is
                    let saved_position = self.lexer.position;
                    let saved_read_position = self.lexer.read_position;
                    let saved_current_char = self.lexer.current_char;
                    let saved_current_token = self.current_token.clone();
                    let saved_peek_token = self.peek_token.clone();
                    
                    self.next_token(); // consume '='
                    
                    // Check what comes after '='
                    let is_struct = matches!(&self.current_token, Token::Symbol('{'));
                    let is_enum = matches!(&self.current_token, Token::Operator(op) if op == "|");
                    let is_function = matches!(&self.current_token, Token::Symbol('('));
                    
                    // Restore lexer state
                    self.lexer.position = saved_position;
                    self.lexer.read_position = saved_read_position;
                    self.lexer.current_char = saved_current_char;
                    self.current_token = saved_current_token;
                    self.peek_token = saved_peek_token;
                    
                    if is_struct {
                        declarations.push(Declaration::Struct(self.parse_struct()?));
                    } else if is_enum {
                        declarations.push(Declaration::Enum(self.parse_enum()?));
                    } else if is_function {
                        declarations.push(Declaration::Function(self.parse_function()?));
                    } else {
                        // Try to parse as function (fallback)
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
                        // Look ahead to see if this is "name : type = value"
                        let saved_position = self.lexer.position;
                        let saved_read_position = self.lexer.read_position;
                        let saved_current_char = self.lexer.current_char;
                        let saved_line = self.lexer.line;
                        let saved_column = self.lexer.column;
                        
                        // Advance past the ':'
                        self.next_token();
                        
                        // Try to parse a type
                        let type_result = self.parse_type();
                        
                        // Check if next token is '='
                        let has_equals = if type_result.is_ok() {
                            self.current_token == Token::Operator("=".to_string())
                        } else {
                            false
                        };
                        
                        // Restore lexer state
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.lexer.line = saved_line;
                        self.lexer.column = saved_column;
                        
                        // Re-read the current token
                        let token_with_span = self.lexer.next_token_with_span();
                        self.current_token = token_with_span.token;
                        self.current_span = token_with_span.span;
                        
                        // Re-read the peek token
                        let peek_token_with_span = self.lexer.next_token_with_span();
                        self.peek_token = peek_token_with_span.token;
                        self.peek_span = peek_token_with_span.span;
                        
                        if has_equals {
                            println!("DEBUG: Found explicit type declaration, calling parse_variable_declaration");
                            self.parse_variable_declaration()
                        } else {
                            println!("DEBUG: Not a variable declaration, treating as expression");
                            // Not a variable declaration, treat as expression
                            let expr = self.parse_expression()?;
                            if self.current_token == Token::Symbol(';') {
                                self.next_token();
                            }
                            Ok(Statement::Expression(expr))
                        }
                    }
                    Token::Operator(op) if op == "::" => {
                        // Check for explicit mutable declarations
                        self.parse_variable_declaration()
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
            Token::Keyword(keyword) if keyword == "loop" => {
                self.parse_loop_statement()
            }
            Token::Keyword(keyword) if keyword == "break" => {
                self.next_token();
                let label = if let Token::Identifier(label_name) = &self.current_token {
                    let label_name = label_name.clone();
                    self.next_token();
                    Some(label_name)
                } else {
                    None
                };
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Break { label })
            }
            Token::Keyword(keyword) if keyword == "continue" => {
                self.next_token();
                let label = if let Token::Identifier(label_name) = &self.current_token {
                    let label_name = label_name.clone();
                    self.next_token();
                    Some(label_name)
                } else {
                    None
                };
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Continue { label })
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
        println!("DEBUG: parse_variable_declaration called with token: {:?}", self.current_token);
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected variable name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        let (is_mutable, declaration_type, type_) = match &self.current_token {
            Token::Operator(op) if op == ":=" => {
                println!("DEBUG: Found := operator");
                // Inferred immutable: name := value
                self.next_token();
                (false, VariableDeclarationType::InferredImmutable, None)
            }
            Token::Operator(op) if op == "::=" => {
                println!("DEBUG: Found ::= operator");
                // Inferred mutable: name ::= value
                self.next_token();
                (true, VariableDeclarationType::InferredMutable, None)
            }
            Token::Symbol(':') => {
                println!("DEBUG: Found : symbol for explicit type");
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
                println!("DEBUG: Found :: operator");
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
                println!("DEBUG: Unexpected token in variable declaration: {:?}", self.current_token);
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
    
    fn parse_loop_statement(&mut self) -> Result<Statement> {
        // Skip 'loop' keyword
        self.next_token();
        
        // Check for optional label
        let label = if self.current_token == Token::Symbol(':') {
            self.next_token();
            if let Token::Identifier(label_name) = &self.current_token {
                let label_name = label_name.clone();
                self.next_token();
                Some(label_name)
            } else {
                return Err(CompileError::SyntaxError(
                    "Expected label name after ':'".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
        } else {
            None
        };
        
        // Check if this is a "loop x in collection" or "loop condition"
        let (condition, iterator) = if let Token::Identifier(_) = &self.current_token {
            // Could be "loop x in collection" or "loop condition"
            let first_identifier = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                unreachable!()
            };
            self.next_token();
            
            if self.current_token == Token::Keyword("in".to_string()) {
                // This is "loop x in collection"
                self.next_token();
                let collection = self.parse_expression()?;
                (None, Some(crate::ast::LoopIterator {
                    variable: first_identifier,
                    collection,
                }))
            } else {
                // This is "loop condition" - for now, just use the identifier as condition
                // TODO: Implement proper condition parsing
                (Some(Expression::Identifier(first_identifier)), None)
            }
        } else {
            // No identifier, must be a condition expression
            let condition = self.parse_expression()?;
            (Some(condition), None)
        };
        
        // Opening brace
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' for loop body".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse loop body
        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            body.push(self.parse_statement()?);
        }
        
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close loop body".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        Ok(Statement::Loop {
            condition,
            iterator,
            label,
            body,
        })
    }
}
