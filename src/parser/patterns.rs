use super::core::Parser;
use crate::ast::primitives;
use crate::ast::{BinaryOperator, Expression, Pattern};
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::well_known::well_known;

impl<'a> Parser<'a> {
    pub fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.current_token {
            Token::Underscore => {
                // Wildcard pattern
                self.next_token();
                Ok(Pattern::Wildcard)
            }
            Token::Integer(_) | Token::Float(_) | Token::StringLiteral(_) => {
                // Literal pattern or range pattern
                // Use pattern expression parser to avoid treating | as bitwise OR
                let expr = self.parse_pattern_expression()?;

                // Check for range pattern: 0..10 or 0..=10
                if let Token::Operator(op) = &self.current_token {
                    if op == ".." || op == "..=" {
                        let inclusive = op == "..=";
                        self.next_token(); // consume range operator
                        let end_expr = self.parse_pattern_expression()?;
                        return Ok(Pattern::Range {
                            start: Box::new(expr),
                            end: Box::new(end_expr),
                            inclusive,
                        });
                    }
                }

                Ok(Pattern::Literal(expr))
            }
            // Handle negative number patterns: -9, -3.14, etc.
            Token::Operator(op) if op == "-" => {
                self.next_token(); // consume '-'
                match &self.current_token {
                    Token::Integer(val) => {
                        let val = val.clone();
                        self.next_token();
                        // Create a negative literal by negating the integer
                        if let Ok(n) = val.parse::<i64>() {
                            Ok(Pattern::Literal(Expression::Integer64(-n)))
                        } else if let Ok(n) = val.parse::<i32>() {
                            Ok(Pattern::Literal(Expression::Integer32(-n)))
                        } else {
                            Err(CompileError::SyntaxError(
                                format!("Invalid negative integer: -{}", val),
                                Some(self.current_span.clone()),
                            ))
                        }
                    }
                    Token::Float(val) => {
                        let val = val.clone();
                        self.next_token();
                        if let Ok(n) = val.parse::<f64>() {
                            Ok(Pattern::Literal(Expression::Float64(-n)))
                        } else {
                            Err(CompileError::SyntaxError(
                                format!("Invalid negative float: -{}", val),
                                Some(self.current_span.clone()),
                            ))
                        }
                    }
                    _ => Err(CompileError::SyntaxError(
                        "Expected number after '-' in pattern".to_string(),
                        Some(self.current_span.clone()),
                    )),
                }
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

                // Use EnumLiteral for shorthand syntax (no enum name)
                Ok(Pattern::EnumLiteral {
                    variant: variant_name,
                    payload,
                })
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.next_token();

                // Check if it's a wildcard pattern
                if name == "_" {
                    return Ok(Pattern::Wildcard);
                }

                // Check if it's a boolean literal pattern - use centralized check
                if primitives::is_boolean_literal(&name) {
                    return Ok(Pattern::Literal(Expression::Boolean(name == "true")));
                }

                // Check if it's None without parentheses
                let wk = well_known();
                if wk.is_none(&name) {
                    return Ok(Pattern::EnumLiteral {
                        variant: name,
                        payload: None,
                    });
                }

                // Check if it's an enum variant without prefix (Some, None, Ok, Err)
                // followed by payload in parentheses
                if self.current_token == Token::Symbol('(') {
                    self.next_token();

                    // Check if it's empty parentheses (unit variant)
                    if self.current_token == Token::Symbol(')') {
                        self.next_token();
                        return Ok(Pattern::EnumLiteral {
                            variant: name,
                            payload: None,
                        });
                    }

                    // Parse the payload pattern
                    let payload_pattern = self.parse_pattern()?;

                    if self.current_token != Token::Symbol(')') {
                        return Err(CompileError::SyntaxError(
                            "Expected ')' after enum variant payload".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                    self.next_token();

                    return Ok(Pattern::EnumLiteral {
                        variant: name,
                        payload: Some(Box::new(payload_pattern)),
                    });
                }

                // IMPORTANT: Check for qualified enum pattern FIRST (EnumName.Variant)
                // This must happen before checking for standalone enum variant names
                if self.current_token == Token::Symbol('.')
                    || self.current_token == Token::Operator("::".to_string())
                {
                    let separator = self.current_token.clone();
                    self.next_token();

                    let variant_name = if let Token::Identifier(variant) = &self.current_token {
                        variant.clone()
                    } else {
                        return Err(CompileError::SyntaxError(
                            format!(
                                "Expected variant name after {}",
                                if separator == Token::Symbol('.') {
                                    "."
                                } else {
                                    "::"
                                }
                            ),
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

                // Check if we're parsing an enum variant without payload
                // Common enum variant names should be treated as enum literals
                let is_common_enum_variant =
                    wk.is_option_variant(&name) || wk.is_result_variant(&name);

                // Also check if the identifier starts with a capital letter (enum convention)
                let is_capitalized = name.chars().next().is_some_and(|c| c.is_uppercase());

                // If it's a known enum variant or capitalized, treat as enum pattern
                // BUT ONLY if not followed by '.' (which would make it a qualified pattern)
                if is_common_enum_variant || is_capitalized {
                    // It's an enum variant without payload
                    return Ok(Pattern::EnumLiteral {
                        variant: name,
                        payload: None,
                    });
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

                // Check for guard pattern with ->
                if self.current_token == Token::Operator("->".to_string()) {
                    self.next_token();

                    // This could be a type pattern binding (i32 -> n) or a guard (v -> v > 100)
                    // We need to look ahead to determine which
                    let condition_expr = self.parse_expression()?;

                    // If the expression is a boolean comparison, it's a guard
                    // Otherwise, it's a type pattern binding
                    if matches!(
                        &condition_expr,
                        Expression::BinaryOp {
                            op: BinaryOperator::GreaterThan,
                            ..
                        } | Expression::BinaryOp {
                            op: BinaryOperator::GreaterThanEquals,
                            ..
                        } | Expression::BinaryOp {
                            op: BinaryOperator::LessThan,
                            ..
                        } | Expression::BinaryOp {
                            op: BinaryOperator::LessThanEquals,
                            ..
                        } | Expression::BinaryOp {
                            op: BinaryOperator::Equals,
                            ..
                        } | Expression::BinaryOp {
                            op: BinaryOperator::NotEquals,
                            ..
                        }
                    ) {
                        // It's a guard pattern
                        return Ok(Pattern::Guard {
                            pattern: Box::new(Pattern::Identifier(name.clone())),
                            condition: Box::new(condition_expr),
                        });
                    } else {
                        // It's a type pattern binding
                        return Ok(Pattern::Type {
                            type_name: name,
                            binding: Some(match condition_expr {
                                Expression::Identifier(binding_name) => binding_name,
                                _ => {
                                    return Err(CompileError::SyntaxError(
                                        "Expected identifier after -> in type pattern".to_string(),
                                        Some(self.current_span.clone()),
                                    ));
                                }
                            }),
                        });
                    }
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
                    // Single pattern in parentheses - just unwrap it
                    Ok(patterns.remove(0))
                } else {
                    // Tuple pattern
                    Ok(Pattern::Tuple(patterns))
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
}
