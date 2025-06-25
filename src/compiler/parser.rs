use super::lexer::{Lexer, Token};
use crate::ast::{Program, Declaration, Function, Statement, Expression, AstType, BinaryOperator};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut declarations = vec![];
        
        while self.current_token != Token::Eof {
            // Parse top-level declarations
            if let Token::Identifier(_) = &self.current_token {
                // Could be a function definition: name = (params) returnType { ... }
                if self.peek_token == Token::Operator("=".to_string()) {
                    match self.parse_function() {
                        Ok(function) => {
                            declarations.push(Declaration::Function(function));
                            // Do not advance token here; parse_function already advances as needed
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else {
                    // Could be a variable declaration or other statement
                    match self.parse_statement() {
                        Ok(_statement) => {
                            // For now, skip non-function declarations
                            // Do not advance token here; parse_statement already advances as needed
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            } else {
                self.next_token();
            }
        }
        
        Ok(Program { declarations })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        // Parse function name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err("Expected function name".to_string());
        };
        self.next_token();
        
        // Skip '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err("Expected '='".to_string());
        }
        self.next_token();
        
        // Skip '('
        if self.current_token != Token::Symbol('(') {
            return Err("Expected '('".to_string());
        }
        self.next_token();
        
        // Parse arguments
        let mut args = vec![];
        while self.current_token != Token::Symbol(')') && self.current_token != Token::Eof {
            // Parse parameter name
            let param_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err("Expected parameter name".to_string());
            };
            self.next_token();
            
            // Skip ':'
            if self.current_token != Token::Symbol(':') {
                return Err("Expected ':'".to_string());
            }
            self.next_token();
            
            // Parse parameter type
            let param_type = self.parse_type()?;
            
            args.push((param_name, param_type));
            
            // Check for comma or closing parenthesis
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol(')') {
                return Err("Expected ',' or ')'".to_string());
            }
        }
        
        // Skip ')'
        if self.current_token != Token::Symbol(')') {
            return Err("Expected ')'".to_string());
        }
        self.next_token();
        
        // Parse return type
        let return_type = self.parse_type()?;
        
        // Skip '{'
        if self.current_token != Token::Symbol('{') {
            return Err("Expected '{'".to_string());
        }
        self.next_token();
        
        // Parse function body
        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            // Save current lexer state for backtracking
            let saved_position = self.lexer.position;
            let saved_read_position = self.lexer.read_position;
            let saved_current_char = self.lexer.current_char;
            let saved_token = self.current_token.clone();
            let saved_peek = self.peek_token.clone();
            
            // Try to parse a statement first
            match self.parse_statement() {
                Ok(stmt) => {
                    body.push(stmt);
                },
                Err(e) => {
                    // Check if this is a binary expression error
                    if e.contains("Binary expression, not statement") {
                        // Restore lexer state and parse as expression
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.current_token = saved_token;
                        self.peek_token = saved_peek;
                        
                        if let Ok(expr) = self.parse_expression() {
                            body.push(Statement::Expression(expr));
                            
                            // Consume semicolon if present
                            if self.current_token == Token::Symbol(';') {
                                self.next_token();
                            }
                        } else {
                            break;
                        }
                    } else {
                        // Restore lexer state and try to parse as trailing expression
                        self.lexer.position = saved_position;
                        self.lexer.read_position = saved_read_position;
                        self.lexer.current_char = saved_current_char;
                        self.current_token = saved_token;
                        self.peek_token = saved_peek;
                        
                        // Check if we're at a token that could be a trailing expression
                        let could_be_expression = matches!(
                            self.current_token,
                            Token::Identifier(_) | Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) | Token::Symbol('(')
                        );
                        
                        if could_be_expression {
                            if let Ok(expr) = self.parse_expression() {
                                body.push(Statement::Expression(expr));
                                
                                // Consume semicolon if present
                                if self.current_token == Token::Symbol(';') {
                                    self.next_token();
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        
        // Skip '}'
        if self.current_token == Token::Symbol('}') {
            self.next_token();
        }
        
        Ok(Function {
            name,
            args,
            return_type,
            body,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.current_token {
            Token::Identifier(_) => {
                // Could be variable declaration or assignment
                let name = if let Token::Identifier(name) = &self.current_token {
                    name.clone()
                } else {
                    return Err("Expected identifier".to_string());
                };
                self.next_token();
                
                match &self.current_token {
                    Token::Operator(op) => {
                        match op.as_str() {
                            ":=" | "::=" => {
                                // Variable declaration: name := value or name ::= value
                                let _is_mutable = op == "::=";
                                self.next_token();
                                let initializer = self.parse_expression()?;
                                
                                if self.current_token == Token::Symbol(';') {
                                    self.next_token();
                                }
                                
                                // For now, we'll use a placeholder type
                                // In a real implementation, we'd infer the type from the initializer
                                Ok(Statement::VariableDeclaration {
                                    name,
                                    type_: AstType::Int32, // Placeholder
                                    initializer: Some(initializer),
                                })
                            }
                            ":" | "::" => {
                                // Typed variable declaration: name: Type = value or name:: Type = value
                                let _is_mutable = op == "::";
                                self.next_token();
                                
                                let type_ = self.parse_type()?;
                                
                                if self.current_token != Token::Operator("=".to_string()) {
                                    return Err("Expected '='".to_string());
                                }
                                self.next_token();
                                
                                let initializer = self.parse_expression()?;
                                
                                if self.current_token == Token::Symbol(';') {
                                    self.next_token();
                                }
                                
                                Ok(Statement::VariableDeclaration {
                                    name,
                                    type_,
                                    initializer: Some(initializer),
                                })
                            }
                            "=" => {
                                // Variable assignment: name = value
                                self.next_token();
                                let value = self.parse_expression()?;
                                
                                if self.current_token == Token::Symbol(';') {
                                    self.next_token();
                                }
                                
                                Ok(Statement::VariableAssignment {
                                    name,
                                    value,
                                })
                            }
                            _ => {
                                // Check if this is a binary expression (e.g., x + y)
                                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">=") {
                                    // This is a binary expression, not a statement
                                    // We need to restore the identifier and parse as expression
                                    return Err("Binary expression, not statement".to_string());
                                }
                                
                                // Could be a function call or other expression
                                let expr = self.parse_expression()?;
                                if self.current_token == Token::Symbol(';') {
                                    self.next_token();
                                }
                                Ok(Statement::Expression(expr))
                            }
                        }
                    }
                    _ => {
                        // Could be a function call or other expression
                        let expr = self.parse_expression()?;
                        if self.current_token == Token::Symbol(';') {
                            self.next_token();
                        }
                        Ok(Statement::Expression(expr))
                    }
                }
            }
            Token::Keyword(ref keyword) => {
                match keyword.as_str() {
                    "loop" => {
                        self.next_token();
                        // Parse loop condition or iterable
                        let condition = self.parse_expression()?;
                        
                        if self.current_token != Token::Symbol('{') {
                            return Err("Expected '{'".to_string());
                        }
                        self.next_token();
                        
                        let mut body = vec![];
                        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                            body.push(self.parse_statement()?);
                        }
                        
                        if self.current_token == Token::Symbol('}') {
                            self.next_token();
                        }
                        
                        Ok(Statement::Loop {
                            condition,
                            body,
                        })
                    }
                    _ => Err(format!("Unknown keyword: {}", keyword)),
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                }
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let result = self.parse_binary_expression(0);
        result
    }

    fn parse_binary_expression(&mut self, precedence: u8) -> Result<Expression, String> {
        let mut left = self.parse_primary_expression()?;
        
        while let Token::Operator(ref op) = self.current_token {
            let op_precedence = self.get_operator_precedence(op);
            if op_precedence < precedence {
                break;
            }
            
            let op_token = op.clone();
            self.next_token();
            
            let right = self.parse_binary_expression(op_precedence + 1)?;
            
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: self.parse_binary_operator(&op_token)?,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match &self.current_token {
            Token::Integer(value) => {
                let value = value.parse::<i32>().map_err(|_| "Invalid integer".to_string())?;
                self.next_token();
                Ok(Expression::Integer32(value))
            }
            Token::Float(value) => {
                let value = value.parse::<f64>().map_err(|_| "Invalid float".to_string())?;
                self.next_token();
                Ok(Expression::Float(value))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                self.next_token();
                Ok(Expression::String(value))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                // Check for function call (e.g., add(21, 21))
                if self.current_token == Token::Symbol('(') {
                    self.next_token();
                    let mut args = vec![];
                    
                    // Parse arguments
                    while self.current_token != Token::Symbol(')') && self.current_token != Token::Eof {
                        args.push(self.parse_expression()?);
                        
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        } else if self.current_token != Token::Symbol(')') {
                            return Err("Expected ',' or ')'".to_string());
                        }
                    }
                    
                    if self.current_token != Token::Symbol(')') {
                        return Err("Expected ')'".to_string());
                    }
                    self.next_token();
                    
                    Ok(Expression::FunctionCall {
                        name,
                        args,
                    })
                }
                // Check for member access (e.g., io.print)
                else if self.current_token == Token::Symbol('.') {
                    self.next_token();
                    if let Token::Identifier(member) = &self.current_token {
                        let member = member.clone();
                        self.next_token();
                        // For now, we'll just return the member name as a string
                        // In a real implementation, we'd create a proper member access expression
                        Ok(Expression::Identifier(format!("{}.{}", name, member)))
                    } else {
                        Err("Expected identifier after '.'".to_string())
                    }
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::Symbol('(') => {
                self.next_token();
                let expr = self.parse_expression()?;
                if self.current_token != Token::Symbol(')') {
                    return Err("Expected ')'".to_string());
                }
                self.next_token();
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }

    fn parse_type(&mut self) -> Result<AstType, String> {
        match &self.current_token {
            Token::Identifier(ref name) => {
                let type_ = match name.as_str() {
                    "int8" => AstType::Int8,
                    "int16" => AstType::Int16,
                    "int32" => AstType::Int32,
                    "int64" => AstType::Int64,
                    "uint8" => AstType::Int8, // Placeholder
                    "uint16" => AstType::Int16, // Placeholder
                    "uint32" => AstType::Int32, // Placeholder
                    "uint64" => AstType::Int64, // Placeholder
                    "usize" => AstType::Int64, // Placeholder
                    "float32" => AstType::Float, // Placeholder
                    "float64" => AstType::Float,
                    "bool" => AstType::Int8, // Placeholder
                    "string" => AstType::String,
                    "void" => AstType::Void,
                    _ => return Err(format!("Unknown type: {}", name)),
                };
                self.next_token();
                Ok(type_)
            }
            _ => Err(format!("Expected type, got: {:?}", self.current_token)),
        }
    }

    fn parse_binary_operator(&self, op: &str) -> Result<BinaryOperator, String> {
        match op {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            "*" => Ok(BinaryOperator::Multiply),
            "/" => Ok(BinaryOperator::Divide),
            "==" => Ok(BinaryOperator::Equals),
            "!=" => Ok(BinaryOperator::NotEquals),
            "<" => Ok(BinaryOperator::LessThan),
            ">" => Ok(BinaryOperator::GreaterThan),
            "<=" => Ok(BinaryOperator::LessThanEquals),
            ">=" => Ok(BinaryOperator::GreaterThanEquals),
            _ => Err(format!("Unknown operator: {}", op)),
        }
    }

    fn get_operator_precedence(&self, op: &str) -> u8 {
        match op {
            "*" | "/" => 2,
            "+" | "-" => 1,
            "==" | "!=" | "<" | ">" | "<=" | ">=" => 0,
            _ => 0,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
} 