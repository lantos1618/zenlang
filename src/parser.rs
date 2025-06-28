use super::lexer::{Lexer, Token};
use crate::error::Span;
use crate::ast::{Program, Expression, Statement, Declaration, Function, ExternalFunction, VariableDeclarationType, Pattern, PatternArm, AstType, BinaryOperator};
use crate::error::{CompileError, Result};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    current_span: Span,
    peek_span: Span,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token_with_span = lexer.next_token_with_span();
        let peek_token_with_span = lexer.next_token_with_span();
        Parser {
            lexer,
            current_token: current_token_with_span.token,
            peek_token: peek_token_with_span.token,
            current_span: current_token_with_span.span,
            peek_span: peek_token_with_span.span,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = vec![];
        
        while self.current_token != Token::Eof {
            // Parse top-level declarations
            if let Token::Identifier(_) = &self.current_token {
                // Could be a function definition: name = (params) returnType { ... }
                if self.peek_token == Token::Operator("=".to_string()) {
                    // Check if it's a struct or enum definition
                    let name = if let Token::Identifier(name) = &self.current_token {
                        name.clone()
                    } else {
                        return Err(CompileError::SyntaxError("Expected identifier".to_string(), Some(self.current_span.clone())));
                    };
                    
                    // Look ahead to see if it's a struct or enum
                    let saved_position = self.lexer.position;
                    let saved_read_position = self.lexer.read_position;
                    let saved_current_char = self.lexer.current_char;
                    let saved_line = self.lexer.line;
                    let saved_column = self.lexer.column;
                    
                    // Skip the identifier and '='
                    self.next_token(); // skip identifier
                    self.next_token(); // skip '='
                    
                    match self.current_token {
                        Token::Symbol('{') => {
                            // Restore lexer state
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.lexer.line = saved_line;
                            self.lexer.column = saved_column;
                            self.current_token = Token::Identifier(name.clone());
                            self.peek_token = Token::Operator("=".to_string());
                            
                            // Struct definition: Name = { field: type, ... }
                            match self.parse_struct_definition() {
                                Ok(struct_def) => {
                                    declarations.push(Declaration::Struct(struct_def));
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        Token::Symbol('|') => {
                            // Restore lexer state
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.lexer.line = saved_line;
                            self.lexer.column = saved_column;
                            self.current_token = Token::Identifier(name.clone());
                            self.peek_token = Token::Operator("=".to_string());
                            
                            // Enum definition: Name = | Variant1 | Variant2(data: type) | ...
                            match self.parse_enum_definition() {
                                Ok(enum_def) => {
                                    declarations.push(Declaration::Enum(enum_def));
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        Token::Symbol('(') => {
                            // Restore lexer state
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.lexer.line = saved_line;
                            self.lexer.column = saved_column;
                            self.current_token = Token::Identifier(name.clone());
                            self.peek_token = Token::Operator("=".to_string());
                            
                            // Function definition: name = (params) returnType { ... }
                            match self.parse_function() {
                                Ok(function) => {
                                    declarations.push(Declaration::Function(function));
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        _ => {
                            // Restore lexer state
                            self.lexer.position = saved_position;
                            self.lexer.read_position = saved_read_position;
                            self.lexer.current_char = saved_current_char;
                            self.lexer.line = saved_line;
                            self.lexer.column = saved_column;
                            self.current_token = Token::Identifier(name.clone());
                            self.peek_token = Token::Operator("=".to_string());
                            
                            // Try parsing as a function anyway
                            match self.parse_function() {
                                Ok(function) => {
                                    declarations.push(Declaration::Function(function));
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
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
            } else if let Token::Keyword(ref keyword) = &self.current_token {
                match keyword.as_str() {
                    "extern" => {
                        // External function declaration
                        match self.parse_external_function() {
                            Ok(ext_func) => {
                                declarations.push(Declaration::ExternalFunction(ext_func));
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    "comptime" => {
                        // Comptime block
                        match self.parse_comptime_block() {
                            Ok(_comptime_block) => {
                                declarations.push(Declaration::ModuleImport {
                                    alias: "comptime".to_string(),
                                    module_path: "comptime_block".to_string(),
                                });
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        self.next_token();
                    }
                }
            } else {
                self.next_token();
            }
        }
        
        Ok(Program { declarations })
    }

    fn parse_function(&mut self) -> Result<Function> {
        // Parse function name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected function name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        // Skip '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError("Expected '='".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Skip '('
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError("Expected '('".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse arguments
        let mut args = vec![];
        while self.current_token != Token::Symbol(')') && self.current_token != Token::Eof {
            // Parse parameter name
            let param_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError("Expected parameter name".to_string(), Some(self.current_span.clone())));
            };
            self.next_token();
            
            // Skip ':'
            if self.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError("Expected ':'".to_string(), Some(self.current_span.clone())));
            }
            self.next_token();
            
            // Parse parameter type
            let param_type = self.parse_type()?;
            args.push((param_name, param_type));
            
            // Skip ',' if there are more parameters
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            }
        }
        
        // Skip ')'
        if self.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse return type
        let return_type = self.parse_type()?;
        
        // Skip '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError("Expected '{'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse function body
        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        
        // Skip '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError("Expected '}'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        Ok(Function {
            name,
            args,
            return_type,
            body,
            is_async: false, // TODO: Support async functions
        })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
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
                        let saved_line = self.lexer.line;
                        let saved_column = self.lexer.column;
                        
                        self.next_token(); // skip identifier
                        self.next_token(); // skip ':'
                        
                        if self.current_token == Token::Symbol(':') {
                            // :: T = (mutable with explicit type)
                            self.next_token(); // skip second ':'
                            let type_ = self.parse_type()?;
                            
                            if self.current_token != Token::Operator("=".to_string()) {
                                // Restore and try as expression
                                self.lexer.position = saved_position;
                                self.lexer.read_position = saved_read_position;
                                self.lexer.current_char = saved_current_char;
                                self.lexer.line = saved_line;
                                self.lexer.column = saved_column;
                                self.current_token = Token::Identifier(name_clone.clone());
                                self.peek_token = Token::Symbol(':');
                                
                                let expr = self.parse_expression()?;
                                Ok(Statement::Expression(expr))
                            } else {
                                self.next_token(); // skip '='
                                let initializer = Some(self.parse_expression()?);
                                
                                Ok(Statement::VariableDeclaration {
                                    name: name_clone,
                                    type_: Some(type_),
                                    initializer,
                                    is_mutable: true,
                                    declaration_type: VariableDeclarationType::ExplicitMutable,
                                })
                            }
                        } else {
                            // : T = (immutable with explicit type)
                            let type_ = self.parse_type()?;
                            
                            if self.current_token != Token::Operator("=".to_string()) {
                                // Restore and try as expression
                                self.lexer.position = saved_position;
                                self.lexer.read_position = saved_read_position;
                                self.lexer.current_char = saved_current_char;
                                self.lexer.line = saved_line;
                                self.lexer.column = saved_column;
                                self.current_token = Token::Identifier(name_clone.clone());
                                self.peek_token = Token::Symbol(':');
                                
                                let expr = self.parse_expression()?;
                                Ok(Statement::Expression(expr))
                            } else {
                                self.next_token(); // skip '='
                                let initializer = Some(self.parse_expression()?);
                                
                                Ok(Statement::VariableDeclaration {
                                    name: name_clone,
                                    type_: Some(type_),
                                    initializer,
                                    is_mutable: false,
                                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                                })
                            }
                        }
                    }
                    _ => {
                        // Regular expression
                        let expr = self.parse_expression()?;
                        Ok(Statement::Expression(expr))
                    }
                }
            }
            Token::Keyword(keyword) => match keyword.as_str() {
                "loop" => self.parse_loop_statement(),
                "break" => {
                    self.next_token();
                    let label = if let Token::Identifier(_) = &self.current_token {
                        let label = if let Token::Identifier(name) = &self.current_token {
                            name.clone()
                        } else {
                            unreachable!()
                        };
                        self.next_token();
                        Some(label)
                    } else {
                        None
                    };
                    Ok(Statement::Break { label })
                }
                "continue" => {
                    self.next_token();
                    let label = if let Token::Identifier(_) = &self.current_token {
                        let label = if let Token::Identifier(name) = &self.current_token {
                            name.clone()
                        } else {
                            unreachable!()
                        };
                        self.next_token();
                        Some(label)
                    } else {
                        None
                    };
                    Ok(Statement::Continue { label })
                }
                "return" => {
                    self.next_token();
                    let expr = self.parse_expression()?;
                    Ok(Statement::Return(expr))
                }
                _ => Err(CompileError::SyntaxError(format!("Unexpected keyword: {}", keyword), Some(self.current_span.clone()))),
            },
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
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
        
        let (is_mutable, declaration_type) = match &self.current_token {
            Token::Operator(op) if op == ":=" => (false, VariableDeclarationType::InferredImmutable),
            Token::Operator(op) if op == "::=" => (true, VariableDeclarationType::InferredMutable),
            _ => return Err(CompileError::SyntaxError("Expected ':=' or '::='".to_string(), Some(self.current_span.clone()))),
        };
        self.next_token();
        
        let initializer = Some(self.parse_expression()?);
        
        Ok(Statement::VariableDeclaration {
            name,
            type_: None, // Inferred type
            initializer,
            is_mutable,
            declaration_type,
        })
    }

    fn parse_loop_statement(&mut self) -> Result<Statement> {
        self.next_token(); // skip 'loop'
        
        let label = if matches!(&self.current_token, Token::Identifier(_)) && self.peek_token == Token::Symbol(':') {
            let label = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                unreachable!()
            };
            self.next_token(); // skip identifier
            self.next_token(); // skip ':'
            Some(label)
        } else {
            None
        };
        
        // Check for iterator loop: loop x in collection
        let iterator = if matches!(&self.current_token, Token::Identifier(_)) && self.peek_token == Token::Keyword("in".to_string()) {
            let variable = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                unreachable!()
            };
            self.next_token(); // skip variable
            self.next_token(); // skip 'in'
            let collection = self.parse_expression()?;
            Some(crate::ast::LoopIterator { variable, collection })
        } else {
            None
        };
        
        let condition = if iterator.is_none() && self.current_token != Token::Symbol('{') {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        // Parse loop body
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError("Expected '{' for loop body".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError("Expected '}'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        Ok(Statement::Loop {
            condition,
            iterator,
            label,
            body,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, precedence: u8) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;
        
        while precedence < self.get_operator_precedence(&self.current_token) {
            let op = self.parse_binary_operator(&self.current_token)?;
            self.next_token();
            let right = self.parse_binary_expression(self.get_operator_precedence(&self.current_token))?;
            
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Symbol('&') => {
                self.next_token();
                let expr = self.parse_unary_expression()?;
                Ok(Expression::AddressOf(Box::new(expr)))
            }
            Token::Symbol('*') => {
                self.next_token();
                let expr = self.parse_unary_expression()?;
                Ok(Expression::Dereference(Box::new(expr)))
            }
            Token::Symbol('?') => {
                self.next_token();
                let scrutinee = Box::new(self.parse_expression()?);
                let arms = self.parse_pattern_arms()?;
                Ok(Expression::PatternMatch { scrutinee, arms })
            }
            _ => self.parse_primary_expression(),
        }
    }

    fn parse_pattern_arms(&mut self) -> Result<Vec<PatternArm>> {
        let mut arms = vec![];
        
        while self.current_token == Token::Symbol('|') {
            self.next_token(); // skip '|'
            
            let pattern = self.parse_pattern()?;
            
            // Check for guard condition (->)
            let guard = if self.current_token == Token::Operator("->".to_string()) {
                self.next_token(); // skip '->'
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            // Check for => separator
            if self.current_token != Token::Operator("=>".to_string()) {
                return Err(CompileError::SyntaxError("Expected '=>'".to_string(), Some(self.current_span.clone())));
            }
            self.next_token(); // skip '=>'
            
            let body = self.parse_expression()?;
            
            arms.push(PatternArm { pattern, guard, body });
        }
        
        Ok(arms)
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.current_token {
            Token::Symbol('_') => {
                self.next_token();
                Ok(Pattern::Wildcard)
            }
            Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) => {
                let expr = self.parse_primary_expression()?;
                Ok(Pattern::Literal(expr))
            }
            Token::Identifier(name) => {
                // Check for enum variant pattern
                if self.peek_token == Token::Symbol('.') {
                    let enum_name = name.clone();
                    self.next_token(); // skip enum name
                    self.next_token(); // skip '.'
                    
                    let variant = if let Token::Identifier(name) = &self.current_token {
                        name.clone()
                    } else {
                        return Err(CompileError::SyntaxError("Expected variant name".to_string(), Some(self.current_span.clone())));
                    };
                    self.next_token();
                    
                    let payload = if self.current_token == Token::Symbol('(') {
                        self.next_token(); // skip '('
                        let pattern = Some(Box::new(self.parse_pattern()?));
                        if self.current_token != Token::Symbol(')') {
                            return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
                        }
                        self.next_token(); // skip ')'
                        pattern
                    } else {
                        None
                    };
                    
                    Ok(Pattern::EnumVariant {
                        enum_name,
                        variant,
                        payload,
                    })
                } else {
                    // Check for binding pattern (->)
                    if self.peek_token == Token::Operator("->".to_string()) {
                        let name = name.clone();
                        self.next_token(); // skip identifier
                        self.next_token(); // skip '->'
                        let pattern = Box::new(self.parse_pattern()?);
                        Ok(Pattern::Binding { name, pattern })
                    } else {
                        // Simple identifier pattern
                        let name = name.clone();
                        self.next_token();
                        Ok(Pattern::Identifier(name))
                    }
                }
            }
            _ => Err(CompileError::SyntaxError("Invalid pattern".to_string(), Some(self.current_span.clone()))),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Integer(value) => {
                let value = value.parse::<i64>().map_err(|_| {
                    CompileError::SyntaxError("Invalid integer".to_string(), Some(self.current_span.clone()))
                })?;
                self.next_token();
                Ok(Expression::Integer64(value))
            }
            Token::Float(value) => {
                let value = value.parse::<f64>().map_err(|_| {
                    CompileError::SyntaxError("Invalid float".to_string(), Some(self.current_span.clone()))
                })?;
                self.next_token();
                Ok(Expression::Float64(value))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                self.next_token();
                Ok(Expression::String(value))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                // Check for function call
                if self.current_token == Token::Symbol('(') {
                    self.next_token();
                    let mut args = vec![];
                    
                    while self.current_token != Token::Symbol(')') && self.current_token != Token::Eof {
                        args.push(self.parse_expression()?);
                        
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        }
                    }
                    
                    if self.current_token != Token::Symbol(')') {
                        return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
                    }
                    self.next_token();
                    
                    Ok(Expression::FunctionCall { name, args })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::Symbol('(') => {
                self.next_token();
                let expr = self.parse_expression()?;
                
                if self.current_token != Token::Symbol(')') {
                    return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
                }
                self.next_token();
                
                Ok(expr)
            }
            _ => Err(CompileError::SyntaxError("Unexpected token".to_string(), Some(self.current_span.clone()))),
        }
    }

    fn parse_type(&mut self) -> Result<AstType> {
        match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                // Check for generic types
                if self.current_token == Token::Symbol('<') {
                    self.next_token();
                    let mut type_args = vec![];
                    
                    while self.current_token != Token::Symbol('>') && self.current_token != Token::Eof {
                        type_args.push(self.parse_type()?);
                        
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        }
                    }
                    
                    if self.current_token != Token::Symbol('>') {
                        return Err(CompileError::SyntaxError("Expected '>'".to_string(), Some(self.current_span.clone())));
                    }
                    self.next_token();
                    
                    Ok(AstType::Generic { name, type_args })
                } else {
                    // Simple type name
                    match name.as_str() {
                        "i8" => Ok(AstType::I8),
                        "i16" => Ok(AstType::I16),
                        "i32" => Ok(AstType::I32),
                        "i64" => Ok(AstType::I64),
                        "u8" => Ok(AstType::U8),
                        "u16" => Ok(AstType::U16),
                        "u32" => Ok(AstType::U32),
                        "u64" => Ok(AstType::U64),
                        "f32" => Ok(AstType::F32),
                        "f64" => Ok(AstType::F64),
                        "bool" => Ok(AstType::Bool),
                        "string" => Ok(AstType::String),
                        "void" => Ok(AstType::Void),
                        _ => Ok(AstType::Struct { name, fields: vec![] }), // Assume it's a struct type
                    }
                }
            }
            _ => Err(CompileError::SyntaxError("Expected type".to_string(), Some(self.current_span.clone()))),
        }
    }

    fn parse_binary_operator(&self, token: &Token) -> Result<BinaryOperator> {
        match token {
            Token::Operator(op) => match op.as_str() {
                "+" => Ok(BinaryOperator::Add),
                "-" => Ok(BinaryOperator::Subtract),
                "*" => Ok(BinaryOperator::Multiply),
                "/" => Ok(BinaryOperator::Divide),
                "%" => Ok(BinaryOperator::Modulo),
                "==" => Ok(BinaryOperator::Equals),
                "!=" => Ok(BinaryOperator::NotEquals),
                "<" => Ok(BinaryOperator::LessThan),
                ">" => Ok(BinaryOperator::GreaterThan),
                "<=" => Ok(BinaryOperator::LessThanEquals),
                ">=" => Ok(BinaryOperator::GreaterThanEquals),
                "&&" => Ok(BinaryOperator::And),
                "||" => Ok(BinaryOperator::Or),
                _ => Err(CompileError::SyntaxError(format!("Unknown operator: {}", op), Some(self.current_span.clone()))),
            },
            _ => Err(CompileError::SyntaxError("Expected operator".to_string(), Some(self.current_span.clone()))),
        }
    }

    fn get_operator_precedence(&self, token: &Token) -> u8 {
        match token {
            Token::Operator(op) => match op.as_str() {
                "||" => 1,
                "&&" => 2,
                "==" | "!=" => 3,
                "<" | ">" | "<=" | ">=" => 4,
                "+" | "-" => 5,
                "*" | "/" | "%" => 6,
                _ => 0,
            },
            _ => 0,
        }
    }

    fn next_token(&mut self) {
        let token_with_span = self.lexer.next_token_with_span();
        self.current_token = self.peek_token.clone();
        self.current_span = self.peek_span.clone();
        self.peek_token = token_with_span.token;
        self.peek_span = token_with_span.span;
    }

    fn parse_struct_definition(&mut self) -> Result<crate::ast::StructDefinition> {
        // Parse struct name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected struct name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        // Skip '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError("Expected '='".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Skip '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError("Expected '{'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        let mut fields = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            // Parse field name
            let field_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError("Expected field name".to_string(), Some(self.current_span.clone())));
            };
            self.next_token();
            
            // Skip ':'
            if self.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError("Expected ':'".to_string(), Some(self.current_span.clone())));
            }
            self.next_token();
            
            // Parse field type
            let field_type = self.parse_type()?;
            // Only advance if next token is not a delimiter
            if !matches!(self.current_token, Token::Symbol(',') | Token::Symbol('}')) {
                self.next_token();
            }
            
            // Check for default value
            let default_value = if self.current_token == Token::Operator("=".to_string()) {
                self.next_token();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            fields.push(crate::ast::StructField {
                name: field_name,
                type_: field_type,
                is_mutable: false, // Default to immutable
                default_value,
            });
            
            // Check for comma or closing brace
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol('}') {
                return Err(CompileError::SyntaxError("Expected ',' or '}'".to_string(), Some(self.current_span.clone())));
            }
        }
        
        // Skip '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError("Expected '}'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        Ok(crate::ast::StructDefinition {
            name,
            fields,
        })
    }

    fn parse_enum_definition(&mut self) -> Result<crate::ast::EnumDefinition> {
        // Parse enum name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected enum name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        // Skip '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError("Expected '='".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Skip '|'
        if self.current_token != Token::Symbol('|') {
            return Err(CompileError::SyntaxError("Expected '|'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        let mut variants = vec![];
        while self.current_token != Token::Eof {
            // Parse variant name
            let variant_name = if let Token::Identifier(name) = &self.current_token {
                name.clone()
            } else {
                return Err(CompileError::SyntaxError("Expected variant name".to_string(), Some(self.current_span.clone())));
            };
            self.next_token();
            
            // Check for payload
            let payload = if self.current_token == Token::Symbol('(') {
                self.next_token();
                // Check if it's a named payload (name: type) or just type
                let payload_type = if let Token::Identifier(_) = &self.current_token {
                    // Named payload: name: type
                    self.next_token(); // skip the name
                    if self.current_token != Token::Symbol(':') {
                        return Err(CompileError::SyntaxError("Expected ':' after payload name".to_string(), Some(self.current_span.clone())));
                    }
                    self.next_token(); // skip ':'
                    let t = self.parse_type()?;
                    // Only advance if next token is not a delimiter
                    if !matches!(self.current_token, Token::Symbol(')')) {
                        self.next_token();
                    }
                    t
                } else {
                    // Just type
                    let t = self.parse_type()?;
                    // Only advance if next token is not a delimiter
                    if !matches!(self.current_token, Token::Symbol(')')) {
                        self.next_token();
                    }
                    t
                };
                if self.current_token != Token::Symbol(')') {
                    return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
                }
                self.next_token();
                Some(payload_type)
            } else {
                None
            };
            
            variants.push(crate::ast::EnumVariant {
                name: variant_name,
                payload,
            });
            
            // Check for next variant or end
            if self.current_token == Token::Symbol('|') {
                self.next_token();
            } else {
                break;
            }
        }
        
        Ok(crate::ast::EnumDefinition {
            name,
            variants,
        })
    }

    fn parse_external_function(&mut self) -> Result<crate::ast::ExternalFunction> {
        // Skip 'extern'
        if self.current_token != Token::Keyword("extern".to_string()) {
            return Err(CompileError::SyntaxError("Expected 'extern'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse function name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError("Expected function name".to_string(), Some(self.current_span.clone())));
        };
        self.next_token();
        
        // Skip '('
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError("Expected '('".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Parse arguments
        let mut args = vec![];
        while self.current_token != Token::Symbol(')') && self.current_token != Token::Eof {
            let arg_type = self.parse_type()?;
            // Only advance if next token is not a delimiter
            if !matches!(self.current_token, Token::Symbol(',') | Token::Symbol(')')) {
                self.next_token();
            }
            args.push(arg_type);
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol(')') {
                return Err(CompileError::SyntaxError("Expected ',' or ')'".to_string(), Some(self.current_span.clone())));
            }
        }
        // Skip ')'
        if self.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError("Expected ')'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        // Parse return type
        let return_type = self.parse_type()?;
        // Only advance if next token is not a delimiter (should be EOF for external functions)
        if self.current_token != Token::Eof {
            self.next_token();
        }
        
        Ok(crate::ast::ExternalFunction {
            name,
            args,
            return_type,
            is_varargs: false, // Default to false
        })
    }

    fn parse_comptime_block(&mut self) -> Result<Vec<Statement>> {
        // Skip 'comptime'
        if self.current_token != Token::Keyword("comptime".to_string()) {
            return Err(CompileError::SyntaxError("Expected 'comptime'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        // Skip '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError("Expected '{'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        let mut statements = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
        }
        
        // Skip '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError("Expected '}'".to_string(), Some(self.current_span.clone())));
        }
        self.next_token();
        
        Ok(statements)
    }
} 