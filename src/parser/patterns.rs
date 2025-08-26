use super::core::Parser;
use crate::ast::Pattern;
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.current_token {
            Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) => {
                // Literal pattern
                let expr = self.parse_expression()?;
                Ok(Pattern::Literal(expr))
            }
            Token::Symbol('.') => {
                // Shorthand enum variant pattern: .Variant or .Variant(payload)
                self.next_token();
                
                let variant_name = if let Token::Identifier(variant) = &self.current_token {
                    variant.clone()
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected variant name after '.'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                };
                self.next_token();
                
                // Check for payload pattern with arrow syntax: .Ok -> val
                let payload = if self.current_token == Token::Operator("->".to_string()) {
                    self.next_token();
                    Some(Box::new(self.parse_pattern()?))
                } else if self.current_token == Token::Symbol('(') {
                    // Alternative syntax: .Variant(pattern)
                    self.next_token();
                    let payload_pattern = self.parse_pattern()?;
                    if self.current_token != Token::Symbol(')') {
                        return Err(CompileError::SyntaxError(
                            "Expected ')' after variant payload".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                    self.next_token();
                    Some(Box::new(payload_pattern))
                } else {
                    None
                };
                
                return Ok(Pattern::EnumVariant {
                    enum_name: String::new(), // Will be inferred from context
                    variant: variant_name,
                    payload,
                });
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                // Check if it's a wildcard pattern
                if name == "_" {
                    return Ok(Pattern::Wildcard);
                }
                
                // Check if it's a struct pattern: StructName { field: pattern, ... }
                if self.current_token == Token::Symbol('{') {
                    self.next_token();
                    let mut fields = vec![];
                    
                    while self.current_token != Token::Symbol('}') {
                        if self.current_token == Token::Eof {
                            return Err(CompileError::SyntaxError(
                                "Unexpected end of file in struct pattern".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                        
                        // Field name
                        let field_name = if let Token::Identifier(field) = &self.current_token {
                            field.clone()
                        } else {
                            return Err(CompileError::SyntaxError(
                                "Expected field name in struct pattern".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        };
                        self.next_token();
                        
                        // Colon
                        if self.current_token != Token::Symbol(':') {
                            return Err(CompileError::SyntaxError(
                                "Expected ':' after field name in struct pattern".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                        self.next_token();
                        
                        // Field pattern
                        let field_pattern = self.parse_pattern()?;
                        fields.push((field_name, field_pattern));
                        
                        // Comma separator
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        } else if self.current_token != Token::Symbol('}') {
                            return Err(CompileError::SyntaxError(
                                "Expected ',' or '}' in struct pattern".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                    }
                    
                    self.next_token(); // consume '}'
                    return Ok(Pattern::Struct { name, fields });
                }
                
                // Check if it's an enum variant pattern: EnumName::Variant(pattern)
                if self.current_token == Token::Operator("::".to_string()) {
                    self.next_token();
                    
                    let variant_name = if let Token::Identifier(variant) = &self.current_token {
                        variant.clone()
                    } else {
                        return Err(CompileError::SyntaxError(
                            "Expected variant name after ::".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    };
                    self.next_token();
                    
                    // Check for payload pattern
                    let payload = if self.current_token == Token::Symbol('(') {
                        self.next_token();
                        let payload_pattern = self.parse_pattern()?;
                        
                        if self.current_token != Token::Symbol(')') {
                            return Err(CompileError::SyntaxError(
                                "Expected ')' after variant payload".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                        self.next_token();
                        
                        Some(Box::new(payload_pattern))
                    } else {
                        None
                    };
                    
                    return Ok(Pattern::EnumVariant {
                        enum_name: name,
                        variant: variant_name,
                        payload,
                    });
                }
                
                // Simple identifier pattern
                Ok(Pattern::Identifier(name))
            }
            Token::Symbol('(') => {
                // Parenthesized pattern or tuple pattern
                self.next_token();
                let mut patterns = vec![];
                
                while self.current_token != Token::Symbol(')') {
                    if self.current_token == Token::Eof {
                        return Err(CompileError::SyntaxError(
                            "Unexpected end of file in parenthesized pattern".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                    
                    patterns.push(self.parse_pattern()?);
                    
                    if self.current_token == Token::Symbol(',') {
                        self.next_token();
                    } else if self.current_token != Token::Symbol(')') {
                        return Err(CompileError::SyntaxError(
                            "Expected ',' or ')' in parenthesized pattern".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                }
                
                self.next_token(); // consume ')'
                
                if patterns.len() == 1 {
                    Ok(patterns.remove(0))
                } else {
                    // Tuple pattern - for now, just use the first pattern
                    // TODO: Implement proper tuple pattern support
                    Ok(patterns.remove(0))
                }
            }
            Token::Operator(op) if op == "|" => {
                // Or pattern: | pattern1 | pattern2
                self.next_token();
                let mut patterns = vec![];
                
                while let Token::Operator(op) = &self.current_token {
                    if op != "|" {
                        break;
                    }
                    self.next_token();
                    patterns.push(self.parse_pattern()?);
                }
                
                if patterns.is_empty() {
                    return Err(CompileError::SyntaxError(
                        "Expected at least one pattern after |".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                
                if patterns.len() == 1 {
                    Ok(patterns.remove(0))
                } else {
                    Ok(Pattern::Or(patterns))
                }
            }
            _ => Err(CompileError::SyntaxError(
                format!("Unexpected token in pattern: {:?}", self.current_token),
                Some(self.current_span.clone()),
            )),
        }
    }
    
    pub fn parse_binding_pattern(&mut self) -> Result<Pattern> {
        // Parse binding pattern: name -> pattern
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected identifier for binding".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        if self.current_token != Token::Operator("->".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '->' in binding pattern".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let pattern = self.parse_pattern()?;
        Ok(Pattern::Binding {
            name,
            pattern: Box::new(pattern),
        })
    }
}
