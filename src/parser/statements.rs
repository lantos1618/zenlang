// DO NOT EVER implement 'in'-based loop syntax (e.g., 'for i in ...', 'loop ... in ...').
// Use explicit, non-dangling, non-tertiary forms only. Prefer 'loop 0..10 { ... }' or C-style loops.
use super::core::Parser;
use crate::ast::{Program, Declaration, Statement, VariableDeclarationType, Expression, AstType};
use crate::error::{CompileError, Result};
use crate::lexer::{self, Token};

impl<'a> Parser<'a> {
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = vec![];
        while self.current_token != Token::Eof {
            // Parse top-level declarations
            if let Token::Identifier(_) = &self.current_token {
                // Could be a function definition: name = (params) returnType { ... }
                if self.peek_token == Token::Operator("=".to_string()) || self.peek_token == Token::Symbol('<') {
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
                    
                    // If generics, advance to parse_struct
                    if self.peek_token == Token::Symbol('<') {
                        // Do NOT advance the token; let parse_struct handle the name and generics
                        declarations.push(Declaration::Struct(self.parse_struct()?));
                    } else {
                        // Need to look ahead to determine if it's a struct, enum, or function
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
            } else if let Token::Keyword(keyword) = &self.current_token {
                if matches!(keyword, crate::lexer::Keyword::Comptime) {
                    // Parse comptime block
                    self.next_token(); // consume 'comptime'
                    if self.current_token != Token::Symbol('{') {
                        return Err(CompileError::SyntaxError(
                            "Expected '{' after comptime".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                    self.next_token(); // consume '{'
                    
                    let mut statements = vec![];
                    while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                        statements.push(self.parse_statement()?);
                    }
                    
                    if self.current_token != Token::Symbol('}') {
                        return Err(CompileError::SyntaxError(
                            "Expected '}' to close comptime block".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                    self.next_token(); // consume '}'
                    
                    // Add comptime block to declarations
                    declarations.push(Declaration::ComptimeBlock(statements));
                } else {
                    return Err(CompileError::SyntaxError(
                        format!("Unexpected keyword at top level: {:?}", keyword),
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
            Token::Identifier(_name) => {
                // Check for variable declarations using peek tokens
                match &self.peek_token {
                    Token::Operator(op) if op == ":=" || op == "::=" => {
                        self.parse_variable_declaration()
                    }
                    Token::Symbol(':') => {
                        self.parse_variable_declaration()
                    }
                    Token::Operator(op) if op == "::" => {
                        self.parse_variable_declaration()
                    }
                    Token::Operator(op) if op == "=" => {
                        self.parse_variable_assignment()
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
            Token::Symbol('?') => {
                // Parse conditional expression
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
            Token::Symbol('(') => {
                // Parse parenthesized expression as statement
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
            Token::Keyword(lexer::Keyword::Return) => {
                self.next_token();
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Return(expr))
            }
            Token::Keyword(lexer::Keyword::Loop) => {
                self.parse_loop_statement()
            }
            Token::Keyword(lexer::Keyword::Break) => {
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
            Token::Keyword(lexer::Keyword::Continue) => {
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
            Token::Keyword(lexer::Keyword::Match) => {
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
            // Handle literal expressions as valid statements
            Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) => {
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
            _ => {
                println!("DEBUG: Unexpected token in statement: {:?} at position {:?}", self.current_token, self.current_span);
                Err(CompileError::SyntaxError(
                    format!("Unexpected token in statement: {:?}", self.current_token),
                    Some(self.current_span.clone()),
                ))
            }
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
        
        // For inferred declarations (:= and ::=), leave type_ as None
        // For explicit declarations (: T = and :: T =), use the parsed type
        let final_type = type_;
        
        if self.current_token == Token::Symbol(';') {
            self.next_token();
        }
        
        Ok(Statement::VariableDeclaration {
            name,
            type_: final_type,
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
        
        // Parse loop condition (required for non-infinite loops)
        let condition = if let Token::Identifier(_) = &self.current_token {
            // Parse a condition expression starting with an identifier
            let condition = self.parse_expression()?;
            Some(condition)
        } else {
            // Parse a condition expression
            let condition = self.parse_expression()?;
            Some(condition)
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
            label,
            body,
        })
    }
    
    fn parse_variable_assignment(&mut self) -> Result<Statement> {
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected variable name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token(); // consume identifier
        
        // Consume the '=' operator
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError("Expected '=' for assignment".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse the value expression
        let value = self.parse_expression()?;
        
        // Consume semicolon if present
        if self.current_token == Token::Symbol(';') {
            self.next_token();
        }
        
        Ok(Statement::VariableAssignment {
            name,
            value,
        })
    }
}
