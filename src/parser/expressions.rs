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
            _ => self.parse_primary_expression(),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Keyword(crate::lexer::Keyword::Match) => {
                self.next_token();
                self.parse_match_expression()
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
                Ok(Expression::String(value))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                let mut expr = Expression::Identifier(name);
                
                // Handle member access and function calls
                while let Token::Symbol('.') = &self.current_token {
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
                
                // Handle function call if present
                if self.current_token == Token::Symbol('(') {
                    if let Expression::MemberAccess { object, member } = expr {
                        self.parse_call_expression_with_object(*object, member)
                    } else if let Expression::Identifier(name) = expr {
                        self.parse_call_expression(name)
                    } else {
                        Err(CompileError::SyntaxError(
                            "Unexpected expression type for function call".to_string(),
                            Some(self.current_span.clone()),
                        ))
                    }
                } else {
                    Ok(expr)
                }
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
                
                // Handle member access after parenthesized expression
                while let Token::Symbol('.') = &self.current_token {
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
                
                // Handle function call if present
                if self.current_token == Token::Symbol('(') {
                    if let Expression::MemberAccess { object, member } = expr {
                        self.parse_call_expression_with_object(*object, member)
                    } else {
                        Err(CompileError::SyntaxError(
                            "Cannot call non-identifier expression".to_string(),
                            Some(self.current_span.clone()),
                        ))
                    }
                } else {
                    Ok(expr)
                }
            }
            Token::Operator(op) if op == "?" => {
                // Conditional expression: ? expr -> pattern { ... }
                self.next_token();
                self.parse_conditional_expression()
            }
            Token::Symbol('?') => {
                // Conditional expression: ? expr -> pattern { ... }
                self.next_token();
                self.parse_conditional_expression()
            }
            _ => Err(CompileError::SyntaxError(
                format!("Unexpected token: {:?}", self.current_token),
                Some(self.current_span.clone()),
            )),
        }
    }
    
    fn parse_conditional_expression(&mut self) -> Result<Expression> {
        // Parse conditional expression: expr -> binding_pattern { | pattern => expr | pattern => expr }
        let scrutinee = Box::new(self.parse_expression()?);
        

        
        if self.current_token != Token::Operator("->".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '->' in conditional expression, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse binding pattern (the identifier after ->)
        let binding_name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected identifier for binding pattern".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        let binding_pattern = Pattern::Binding {
            name: binding_name.clone(),
            pattern: Box::new(Pattern::Identifier(binding_name)),
        };
        
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' after binding pattern".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut arms = vec![];
        
        while self.current_token != Token::Symbol('}') {
            if self.current_token == Token::Eof {
                return Err(CompileError::SyntaxError(
                    "Unexpected end of file in conditional expression".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            
            // Each arm starts with |
            if self.current_token != Token::Symbol('|') {
                return Err(CompileError::SyntaxError(
                    "Expected '|' at start of conditional arm".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            
            // Parse pattern
            let pattern = self.parse_pattern()?;
            
            // Optional guard condition
            let guard = if self.current_token == Token::Operator("->".to_string()) {
                self.next_token();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            // =>
            if self.current_token != Token::Operator("=>".to_string()) {
                return Err(CompileError::SyntaxError(
                    "Expected '=>' in conditional arm".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            
            // Parse body expression
            let body = self.parse_expression()?;
            
            arms.push(crate::ast::ConditionalArm {
                pattern,
                guard,
                body,
            });
        }
        
        self.next_token(); // consume '}'
        
        Ok(Expression::Conditional {
            scrutinee,
            arms,
        })
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

    fn parse_match_expression(&mut self) -> Result<Expression> {
        // Parse: match expr { | pattern => expr ... }
        let scrutinee = Box::new(self.parse_expression()?);
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' after match scrutinee".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        let mut arms = vec![];
        while self.current_token != Token::Symbol('}') {
            if self.current_token == Token::Eof {
                return Err(CompileError::SyntaxError(
                    "Unexpected end of file in match expression".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            // Each arm starts with |
            if self.current_token != Token::Symbol('|') {
                return Err(CompileError::SyntaxError(
                    "Expected '|' at start of match arm".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            // Parse pattern
            let pattern = self.parse_pattern()?;
            // Optional guard condition
            let guard = if self.current_token == Token::Operator("->".to_string()) {
                self.next_token();
                Some(self.parse_expression()?)
            } else {
                None
            };
            // =>
            if self.current_token != Token::Operator("=>".to_string()) {
                return Err(CompileError::SyntaxError(
                    "Expected '=>' in match arm".to_string(),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            // Parse body expression
            let body = self.parse_expression()?;
            arms.push(crate::ast::PatternArm {
                pattern,
                guard,
                body,
            });
        }
        self.next_token(); // consume '}'
        Ok(Expression::PatternMatch {
            scrutinee,
            arms,
        })
    }

    fn token_to_binary_operator(&self, op: &str) -> Result<BinaryOperator> {
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
}
