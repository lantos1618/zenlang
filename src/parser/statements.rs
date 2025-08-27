// Loop syntax is simplified - only conditional and infinite loops are supported.
// Range and iterator loops have been removed in favor of functional iteration.
use super::core::Parser;
use crate::ast::{Program, Declaration, Statement, VariableDeclarationType, Expression};
use crate::error::{CompileError, Result};
use crate::lexer::{self, Token};

impl<'a> Parser<'a> {
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = vec![];
        while self.current_token != Token::Eof {
            // Parse top-level declarations
            if let Token::Identifier(_) = &self.current_token {
                // Could be a function definition: name :: (params) -> returnType { ... } or name = ...
                if self.peek_token == Token::Operator("::".to_string()) {
                    // Function with type annotation: name :: (params) -> returnType { ... }
                    declarations.push(Declaration::Function(self.parse_function()?));
                } else if self.peek_token == Token::Operator("=".to_string()) || self.peek_token == Token::Operator("<".to_string()) {
                    // Check if it's a struct, enum, or function definition
                    let _name = if let Token::Identifier(name) = &self.current_token {
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
                    
                    // If generics, need to look ahead to determine struct vs function
                    if self.peek_token == Token::Operator("<".to_string()) {
                        // Look ahead to see if it's a struct or a function with generics
                        let saved_position = self.lexer.position;
                        let saved_read_position = self.lexer.read_position;
                        let saved_current_char = self.lexer.current_char;
                        let saved_current = self.current_token.clone();
                        let saved_peek = self.peek_token.clone();
                        let saved_span = self.current_span.clone();
                        let saved_peek_span = self.peek_span.clone();
                        
                        // Skip past the generics to see what follows
                        self.next_token(); // Move to <
                        self.next_token(); // Move past <
                        let mut depth = 1;
                        while depth > 0 && self.current_token != Token::Eof {
                            if self.current_token == Token::Operator("<".to_string()) {
                                depth += 1;
                            } else if self.current_token == Token::Operator(">".to_string()) {
                                depth -= 1;
                            }
                            if depth > 0 {
                                self.next_token();
                            }
                        }
                        
                        if depth == 0 {
                            self.next_token(); // Move past >
                            
                            // Check what comes after the generics
                            let is_struct = self.current_token == Token::Operator("=".to_string()) 
                                && self.peek_token == Token::Symbol('{');
                            let is_enum = self.current_token == Token::Operator("=".to_string())
                                && self.peek_token == Token::Symbol('|');
                            let is_function = self.current_token == Token::Operator("=".to_string()) 
                                && self.peek_token == Token::Symbol('(');
                            let is_behavior = self.current_token == Token::Operator("=".to_string()) 
                                && self.peek_token == Token::Keyword(lexer::Keyword::Behavior);
                            
                            // Restore lexer state
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.current_token = saved_current;
                            self.peek_token = saved_peek;
                            self.current_span = saved_span;
                            self.peek_span = saved_peek_span;
                            
                            if is_behavior {
                                declarations.push(Declaration::Behavior(self.parse_behavior()?));
                            } else if is_enum {
                                declarations.push(Declaration::Enum(self.parse_enum()?));
                            } else if is_function {
                                declarations.push(Declaration::Function(self.parse_function()?));
                            } else if is_struct {
                                declarations.push(Declaration::Struct(self.parse_struct()?));
                            } else {
                                // Default to struct for backward compatibility
                                declarations.push(Declaration::Struct(self.parse_struct()?));
                            }
                        } else {
                            // Malformed generics, restore and try to parse as struct
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.current_token = saved_current;
                            self.peek_token = saved_peek;
                            self.current_span = saved_span;
                            self.peek_span = saved_peek_span;
                            declarations.push(Declaration::Struct(self.parse_struct()?));
                        }
                    } else {
                        // Need to look ahead to determine if it's a struct, enum, behavior, or function
                        self.next_token(); // Move to '='
                        self.next_token(); // Move past '=' to see what comes after
                        
                        // Check what comes after '='
                        let is_struct = matches!(&self.current_token, Token::Symbol('{'));
                        let is_enum = matches!(&self.current_token, Token::Symbol('|'));
                        let is_function = matches!(&self.current_token, Token::Symbol('('));
                        let is_behavior = matches!(&self.current_token, Token::Keyword(lexer::Keyword::Behavior));

                        // Restore lexer state
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.current_token = saved_current_token;
                        self.peek_token = saved_peek_token;

                        if is_behavior {
                            declarations.push(Declaration::Behavior(self.parse_behavior()?));
                        } else if is_struct {
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
                } else if self.peek_token == Token::Symbol('.') {
                    // Could be an impl block: Type.impl = { ... }
                    let type_name = if let Token::Identifier(name) = &self.current_token {
                        name.clone()
                    } else {
                        unreachable!()
                    };
                    
                    // Save state for potential backtrack
                    let saved_position = self.lexer.position;
                    let saved_read_position = self.lexer.read_position;
                    let saved_current_char = self.lexer.current_char;
                    let saved_current_token = self.current_token.clone();
                    let saved_peek_token = self.peek_token.clone();
                    
                    self.next_token(); // consume type name
                    self.next_token(); // consume '.'
                    
                    if let Token::Keyword(lexer::Keyword::Impl) = self.current_token {
                        // This is an impl block
                        self.next_token(); // consume 'impl'
                        declarations.push(Declaration::Impl(self.parse_impl_block_from_type(type_name)?));
                    } else {
                        // Not an impl block, restore and error
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.current_token = saved_current_token;
                        self.peek_token = saved_peek_token;
                        
                        return Err(CompileError::SyntaxError(
                            format!("Expected 'impl' after '{}.'", type_name),
                            Some(self.current_span.clone()),
                        ));
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
                if matches!(keyword, crate::lexer::Keyword::Type) {
                    // Parse type alias: type Name = Type or type Name<T> = Type<T>
                    declarations.push(Declaration::TypeAlias(self.parse_type_alias()?));
                } else if matches!(keyword, crate::lexer::Keyword::Comptime) {
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
                } else if matches!(keyword, crate::lexer::Keyword::Extern) {
                    // Parse external function declaration
                    self.next_token(); // consume 'extern'
                    declarations.push(Declaration::ExternalFunction(self.parse_external_function()?));
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
                    Token::Symbol('.') | Token::Symbol('[') => {
                        // Could be member access or array indexing followed by assignment
                        // Parse the left-hand side expression first
                        let lhs = self.parse_expression()?;
                        
                        // Check if it's followed by an assignment
                        if self.current_token == Token::Operator("=".to_string()) {
                            self.next_token(); // consume '='
                            let value = self.parse_expression()?;
                            if self.current_token == Token::Symbol(';') {
                                self.next_token();
                            }
                            // Use PointerAssignment for member field assignments and array element assignments
                            Ok(Statement::PointerAssignment {
                                pointer: lhs,
                                value,
                            })
                        } else {
                            // Just an expression statement
                            if self.current_token == Token::Symbol(';') {
                                self.next_token();
                            }
                            Ok(Statement::Expression(lhs))
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
            Token::Keyword(lexer::Keyword::Comptime) => {
                // Parse comptime block as statement
                self.next_token(); // consume 'comptime'
                if self.current_token != Token::Symbol('{') {
                    // It's a comptime expression, not a block
                    let expr = Expression::Comptime(Box::new(self.parse_expression()?));
                    if self.current_token == Token::Symbol(';') {
                        self.next_token();
                    }
                    return Ok(Statement::Expression(expr));
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
                Ok(Statement::ComptimeBlock(statements))
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
                Err(CompileError::SyntaxError(
                    format!("Unexpected token in statement: {:?}", self.current_token),
                    Some(self.current_span.clone()),
                ))
            }
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
        use crate::ast::LoopKind;
        
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
        
        // Determine the loop kind - only support infinite and condition loops now
        let kind = if self.current_token == Token::Symbol('{') {
            // No condition - infinite loop: loop { }
            LoopKind::Infinite
        } else {
            // Parse a general condition expression
            let condition = self.parse_expression()?;
            LoopKind::Condition(condition)
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
            kind,
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
