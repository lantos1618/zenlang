use super::core::Parser;
use crate::ast::{Expression, BinaryOperator, Pattern};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, precedence: u8) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;
        
        while let Token::Operator(op) = &self.current_token {
            let op_clone = op.clone();
            let next_prec = self.get_precedence(&op_clone);
            if next_prec > precedence {
                self.next_token(); // advance past the operator
                
                // Handle range expressions specially
                if op_clone == ".." || op_clone == "..=" {
                    let right = self.parse_binary_expression(next_prec)?;
                    left = Expression::Range {
                        start: Box::new(left),
                        end: Box::new(right),
                        inclusive: op_clone == "..=",
                    };
                } else {
                    let right = self.parse_binary_expression(next_prec)?;
                    left = Expression::BinaryOp {
                        left: Box::new(left),
                        op: self.token_to_binary_operator(&op_clone)?,
                        right: Box::new(right),
                    };
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Operator(op) if op == "-" => {
                self.next_token();
                let expr = self.parse_unary_expression()?;
                // For now, represent unary minus as BinaryOp with 0 - expr
                Ok(Expression::BinaryOp {
                    left: Box::new(Expression::Integer32(0)),
                    op: BinaryOperator::Subtract,
                    right: Box::new(expr),
                })
            }
            Token::Operator(op) if op == "!" => {
                self.next_token();
                let expr = self.parse_unary_expression()?;
                // For now, represent logical not as a function call
                Ok(Expression::FunctionCall {
                    name: "not".to_string(),
                    args: vec![expr],
                })
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary_expression()?;
        
        // Handle postfix operators
        loop {
            match &self.current_token {
                Token::Symbol('?') => {
                    // Pattern matching: scrutinee ? | pattern => expression
                    self.next_token(); // consume '?'
                    expr = self.parse_pattern_match(expr)?;
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Keyword(crate::lexer::Keyword::Comptime) => {
                self.next_token(); // consume 'comptime'
                let expr = self.parse_expression()?;
                Ok(Expression::Comptime(Box::new(expr)))
            }
            Token::Integer(value_str) => {
                let value = value_str.parse::<i64>().map_err(|_| {
                    CompileError::SyntaxError(
                        format!("Invalid integer: {}", value_str),
                        Some(self.current_span.clone()),
                    )
                })?;
                self.next_token();
                // Default to Integer32 unless out of range
                if value <= i32::MAX as i64 && value >= i32::MIN as i64 {
                    Ok(Expression::Integer32(value as i32))
                } else {
                    Ok(Expression::Integer64(value))
                }
            }
            Token::Float(value_str) => {
                let value = value_str.parse::<f64>().map_err(|_| {
                    CompileError::SyntaxError(
                        format!("Invalid float: {}", value_str),
                        Some(self.current_span.clone()),
                    )
                })?;
                self.next_token();
                Ok(Expression::Float64(value))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                self.next_token();
                
                // Check if the string contains interpolation syntax
                if value.contains("$(") {
                    self.parse_interpolated_string(value)
                } else {
                    Ok(Expression::String(value))
                }
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                // Check for boolean literals
                if name == "true" {
                    return Ok(Expression::Boolean(true));
                } else if name == "false" {
                    return Ok(Expression::Boolean(false));
                }
                
                // Check for enum variant syntax: EnumName::VariantName
                if self.current_token == Token::Operator("::".to_string()) {
                    self.next_token(); // consume '::'
                    
                    let variant = match &self.current_token {
                        Token::Identifier(v) => v.clone(),
                        _ => {
                            return Err(CompileError::SyntaxError(
                                "Expected variant name after '::'".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                    };
                    self.next_token();
                    
                    // Check for variant payload
                    let payload = if self.current_token == Token::Symbol('(') {
                        self.next_token(); // consume '('
                        let expr = self.parse_expression()?;
                        if self.current_token != Token::Symbol(')') {
                            return Err(CompileError::SyntaxError(
                                "Expected ')' after enum variant payload".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                        self.next_token(); // consume ')'
                        Some(Box::new(expr))
                    } else {
                        None
                    };
                    
                    return Ok(Expression::EnumVariant {
                        enum_name: name,
                        variant,
                        payload,
                    });
                }
                
                // Check for generic type parameters before struct literal
                // e.g., Vec<T> { ... } or Result<T,E> { ... }
                let struct_name = if self.current_token == Token::Operator("<".to_string()) {
                    // This is a generic struct instantiation
                    // We need to consume the type parameters but for now we'll just 
                    // skip to the closing '>' and use the base name
                    let mut depth = 1;
                    self.next_token(); // consume '<'
                    while depth > 0 && self.current_token != Token::Eof {
                        match &self.current_token {
                            Token::Operator(op) if op == "<" => depth += 1,
                            Token::Operator(op) if op == ">" => depth -= 1,
                            _ => {}
                        }
                        self.next_token();
                    }
                    // For now, just use the base name without type params for struct literals
                    // In the future, we might want to preserve the full generic type
                    name.clone()
                } else {
                    name.clone()
                };
                
                // Check for struct literal syntax: Name { field: value, ... }
                if self.current_token == Token::Symbol('{') {
                    return self.parse_struct_literal(struct_name);
                }
                
                let mut expr = Expression::Identifier(name);
                
                // Handle member access, array indexing, and function calls
                loop {
                    match &self.current_token {
                        Token::Symbol('.') => {
                            self.next_token(); // consume '.'
                            
                            let member = match &self.current_token {
                                Token::Identifier(name) => name.clone(),
                                Token::Keyword(kw) => {
                                    // Allow keywords as member names (e.g., .loop, .await, etc.)
                                    format!("{:?}", kw).to_lowercase()
                                }
                                _ => {
                                    return Err(CompileError::SyntaxError(
                                        "Expected identifier after '.'".to_string(),
                                        Some(self.current_span.clone()),
                                    ));
                                }
                            };
                            self.next_token();
                            expr = Expression::MemberAccess {
                                object: Box::new(expr),
                                member,
                            };
                        }
                        Token::Symbol('[') => {
                            // Array indexing
                            self.next_token(); // consume '['
                            let index = self.parse_expression()?;
                            if self.current_token != Token::Symbol(']') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ']' after array index".to_string(),
                                    Some(self.current_span.clone()),
                                ));
                            }
                            self.next_token(); // consume ']'
                            expr = Expression::ArrayIndex {
                                array: Box::new(expr),
                                index: Box::new(index),
                            };
                        }
                        Token::Symbol('(') => {
                            // Function call
                            if let Expression::MemberAccess { object, member } = expr {
                                return self.parse_call_expression_with_object(*object, member);
                            } else if let Expression::Identifier(name) = expr {
                                return self.parse_call_expression(name);
                            } else {
                                return Err(CompileError::SyntaxError(
                                    "Unexpected expression type for function call".to_string(),
                                    Some(self.current_span.clone()),
                                ));
                            }
                        }
                        _ => break,
                    }
                }
                
                Ok(expr)
            }
            Token::Symbol('(') => {
                self.next_token();
                let mut expr = self.parse_expression()?;
                if self.current_token != Token::Symbol(')') {
                    return Err(CompileError::SyntaxError(
                        "Expected closing parenthesis".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
                
                // Handle member access, array indexing, and function calls after parenthesized expression
                loop {
                    match &self.current_token {
                        Token::Symbol('.') => {
                            self.next_token(); // consume '.'
                            
                            let member = match &self.current_token {
                                Token::Identifier(name) => name.clone(),
                                Token::Keyword(kw) => {
                                    // Allow keywords as member names (e.g., .loop, .await, etc.)
                                    format!("{:?}", kw).to_lowercase()
                                }
                                _ => {
                                    return Err(CompileError::SyntaxError(
                                        "Expected identifier after '.'".to_string(),
                                        Some(self.current_span.clone()),
                                    ));
                                }
                            };
                            self.next_token();
                            expr = Expression::MemberAccess {
                                object: Box::new(expr),
                                member,
                            };
                        }
                        Token::Symbol('[') => {
                            // Array indexing
                            self.next_token(); // consume '['
                            let index = self.parse_expression()?;
                            if self.current_token != Token::Symbol(']') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ']' after array index".to_string(),
                                    Some(self.current_span.clone()),
                                ));
                            }
                            self.next_token(); // consume ']'
                            expr = Expression::ArrayIndex {
                                array: Box::new(expr),
                                index: Box::new(index),
                            };
                        }
                        Token::Symbol('(') => {
                            // Function call
                            if let Expression::MemberAccess { object, member } = expr {
                                return self.parse_call_expression_with_object(*object, member);
                            } else {
                                return Err(CompileError::SyntaxError(
                                    "Cannot call non-identifier expression".to_string(),
                                    Some(self.current_span.clone()),
                                ));
                            }
                        }
                        _ => break,
                    }
                }
                
                Ok(expr)
            }
            Token::Symbol('[') => {
                // Array literal: [expr, expr, ...]
                self.parse_array_literal()
            }
            Token::Symbol('{') => {
                // Block expression: { statements... }
                self.parse_block_expression()
            }
            _ => Err(CompileError::SyntaxError(
                format!("Unexpected token: {:?}", self.current_token),
                Some(self.current_span.clone()),
            )),
        }
    }

    fn parse_call_expression(&mut self, function_name: String) -> Result<Expression> {
        self.next_token(); // consume '('
        let mut arguments = vec![];
        if self.current_token != Token::Symbol(')') {
            loop {
                arguments.push(self.parse_expression()?);
                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if self.current_token != Token::Symbol(',') {
                    return Err(CompileError::SyntaxError(
                        "Expected ',' or ')' in function call".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
            }
        }
        self.next_token(); // consume ')'
        Ok(Expression::FunctionCall {
            name: function_name,
            args: arguments,
        })
    }

    fn parse_call_expression_with_object(&mut self, object: Expression, method_name: String) -> Result<Expression> {
        self.next_token(); // consume '('
        let mut arguments = vec![];
        if self.current_token != Token::Symbol(')') {
            loop {
                arguments.push(self.parse_expression()?);
                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if self.current_token != Token::Symbol(',') {
                    return Err(CompileError::SyntaxError(
                        "Expected ',' or ')' in function call".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
            }
        }
        self.next_token(); // consume ')'
        Ok(Expression::FunctionCall {
            name: format!("{}.{}", match &object {
                Expression::Identifier(name) => name.clone(),
                _ => return Err(CompileError::SyntaxError(
                    "Expected identifier for object in method call".to_string(),
                    Some(self.current_span.clone()),
                )),
            }, method_name),
            args: arguments,
        })
    }
    
    fn parse_struct_literal(&mut self, name: String) -> Result<Expression> {
        self.next_token(); // consume '{'
        let mut fields = vec![];
        
        while self.current_token != Token::Symbol('}') {
            // Parse field name
            let field_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => {
                    return Err(CompileError::SyntaxError(
                        "Expected field name in struct literal".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            };
            self.next_token();
            
            // Expect ':'
            if self.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError(
                    "Expected ':' after field name in struct literal".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            
            // Parse field value
            let field_value = self.parse_expression()?;
            fields.push((field_name, field_value));
            
            // Check for comma or end of struct
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            } else if self.current_token != Token::Symbol('}') {
                return Err(CompileError::SyntaxError(
                    "Expected ',' or '}' in struct literal".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
        }
        
        self.next_token(); // consume '}'
        Ok(Expression::StructLiteral { name, fields })
    }

    fn parse_pattern_match(&mut self, scrutinee: Expression) -> Result<Expression> {
        // Parse: scrutinee ? | pattern => expr | pattern => expr ...
        let scrutinee = Box::new(scrutinee);
        // Expect first arm to start with |
        if self.current_token != Token::Symbol('|') {
            return Err(CompileError::SyntaxError(
                "Expected '|' to start pattern matching arms".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        
        let mut arms = vec![];
        
        // Parse arms: | pattern => expr | pattern => expr ...
        while self.current_token == Token::Symbol('|') {
            self.next_token(); // consume '|'
            
            // Parse pattern - could be single or multiple (or patterns)
            let mut patterns = vec![self.parse_pattern()?];
            
            // Check for additional patterns separated by |
            while self.current_token == Token::Symbol('|') && 
                  self.peek_token != Token::Symbol('|') && // Not start of next arm
                  self.peek_token != Token::Eof {
                // This is an or pattern - consume the | and parse the next pattern
                self.next_token();
                patterns.push(self.parse_pattern()?);
            }
            
            // Create the final pattern
            let pattern = if patterns.len() == 1 {
                patterns.remove(0)
            } else {
                Pattern::Or(patterns)
            };
            
            // Check for destructuring/guard with ->
            let guard = if self.current_token == Token::Operator("->".to_string()) {
                self.next_token();
                // TODO: Properly handle destructuring vs guards
                // For now, treat it as a guard
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            // Expect =>
            if self.current_token != Token::Operator("=>".to_string()) {
                return Err(CompileError::SyntaxError(
                    "Expected '=>' after pattern in match arm".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token(); // consume '=>'
            
            // Parse the result expression
            let body = self.parse_expression()?;
            
            arms.push(crate::ast::ConditionalArm {
                pattern,
                guard,
                body,
            });
            
            // Check if there are more arms
            if self.current_token != Token::Symbol('|') {
                break;
            }
        }
        Ok(Expression::Conditional {
            scrutinee,
            arms,
        })
    }

    fn parse_array_literal(&mut self) -> Result<Expression> {
        // Consume '['
        self.next_token();
        
        let mut elements = Vec::new();
        
        // Handle empty array
        if self.current_token == Token::Symbol(']') {
            self.next_token();
            return Ok(Expression::ArrayLiteral(elements));
        }
        
        // Parse first element
        elements.push(self.parse_expression()?);
        
        // Parse remaining elements
        while self.current_token == Token::Symbol(',') {
            self.next_token(); // consume ','
            
            // Allow trailing comma
            if self.current_token == Token::Symbol(']') {
                break;
            }
            
            elements.push(self.parse_expression()?);
        }
        
        // Expect closing ']'
        if self.current_token != Token::Symbol(']') {
            return Err(CompileError::SyntaxError(
                format!("Expected ']' in array literal, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        Ok(Expression::ArrayLiteral(elements))
    }
    
    fn parse_block_expression(&mut self) -> Result<Expression> {
        self.next_token(); // consume '{'
        let mut statements = vec![];
        
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
        }
        
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close block expression".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token(); // consume '}'
        
        Ok(Expression::Block(statements))
    }

    pub(crate) fn token_to_binary_operator(&self, op: &str) -> Result<BinaryOperator> {
        match op {
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
            _ => Err(CompileError::SyntaxError(
                format!("Unknown binary operator: {}", op),
                Some(self.current_span.clone()),
            )),
        }
    }

    fn get_precedence(&self, op: &str) -> u8 {
        match op {
            ".." | "..=" => 1,  // Range has lowest precedence
            "==" | "!=" => 2,
            "<" | "<=" | ">" | ">=" => 3,
            "+" | "-" => 4,
            "*" | "/" | "%" => 5,
            _ => 0,
        }
    }

    fn parse_interpolated_string(&mut self, input: String) -> Result<Expression> {
        use crate::ast::StringPart;
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'(') {
                // Found interpolation start
                chars.next(); // consume '('
                
                // Save any literal part before this interpolation
                if !current.is_empty() {
                    parts.push(StringPart::Literal(current.clone()));
                    current.clear();
                }
                
                // Parse the interpolated expression
                let mut expr_str = String::new();
                let mut paren_count = 1;
                
                while let Some(ch) = chars.next() {
                    if ch == '(' {
                        paren_count += 1;
                        expr_str.push(ch);
                    } else if ch == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            break;
                        }
                        expr_str.push(ch);
                    } else {
                        expr_str.push(ch);
                    }
                }
                
                if paren_count != 0 {
                    return Err(CompileError::SyntaxError(
                        "Unmatched parentheses in string interpolation".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                
                // Parse the expression string
                // We need to create a temporary parser for this expression
                let lexer = crate::lexer::Lexer::new(&expr_str);
                let mut temp_parser = crate::parser::Parser::new(lexer);
                let expr = temp_parser.parse_expression()?;
                parts.push(StringPart::Interpolation(expr));
            } else {
                current.push(ch);
            }
        }
        
        // Add any remaining literal part
        if !current.is_empty() {
            parts.push(StringPart::Literal(current));
        }
        
        // If we have interpolation parts, create an interpolated string expression
        if parts.iter().any(|p| matches!(p, StringPart::Interpolation(_))) {
            Ok(Expression::StringInterpolation { parts })
        } else {
            // No interpolation found, return a simple string
            Ok(Expression::String(input))
        }
    }
}
