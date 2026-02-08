use crate::ast::primitives;
use crate::ast::{Expression, MatchArm, Pattern};
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

/// Parse an expression for a match arm body, but stop at tokens that could start a new pattern
/// This prevents "-13" being interpreted as subtraction in cases like:
///   9 => "value"
///   -13 => "other"
fn parse_match_arm_body(parser: &mut Parser) -> Result<Expression> {
    // Check if we need to peek ahead to avoid consuming operators that start patterns
    // For now, use a simple heuristic: parse only primary expressions and method chains
    // If we need more complex expressions, users can use parentheses: (a + b)
    super::primary::parse_primary_expression(parser)
}

/// Parse a block body `{ statements... final_expr }` for a match arm.
/// Detects the final expression by peeking for `}`.
fn parse_match_arm_block_body(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume '{'

    let mut statements = Vec::new();
    let mut final_expr = None;

    while parser.current_token != Token::Symbol('}') && parser.current_token != Token::Eof {
        // Check if this could be the final expression
        if parser.peek_token == Token::Symbol('}') {
            // This might be the final expression (no semicolon)
            let expr = parser.parse_expression()?;
            if parser.current_token == Token::Symbol(';') {
                // It's a statement, not the final expression
                parser.next_token();
                statements.push(crate::ast::Statement::Expression {
                    expr,
                    span: Some(parser.current_span.clone()),
                });
            } else {
                // It's the final expression
                final_expr = Some(expr);
            }
        } else {
            // Parse as statement to handle variable declarations and assignments
            let stmt = parser.parse_statement()?;
            statements.push(stmt);
        }
    }

    if parser.current_token != Token::Symbol('}') {
        return Err(CompileError::SyntaxError(
            "Expected '}' to close block in match arm".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token(); // consume '}'

    if !statements.is_empty() || final_expr.is_some() {
        if let Some(expr) = final_expr {
            statements.push(crate::ast::Statement::Expression {
                expr,
                span: Some(parser.current_span.clone()),
            });
        }
        Ok(Expression::Block(statements))
    } else {
        Ok(Expression::Block(vec![]))
    }
}

pub fn parse_pattern_match(parser: &mut Parser, scrutinee: Expression) -> Result<Expression> {
    // Parse: scrutinee ? | pattern => expr | pattern => expr ...
    // OR bool short form: scrutinee ? { block }
    let scrutinee = Box::new(scrutinee);

    // Check for block-style pattern match: expr ? { pattern => expr, ... }
    // OR bool pattern short form: expr ? { block }
    // OR ternary conditional: expr ? { true_block } : { false_block }
    if parser.current_token == Token::Symbol('{') {
        // Look ahead to disambiguate between ternary and pattern match
        // Save parser state to restore if needed
        let saved_state = parser.save_state();

        parser.next_token(); // consume '{'

        // Try to parse the block content to see if we hit ':' at block end
        // This indicates a ternary conditional
        let mut brace_depth = 0;
        let mut found_colon_after_block = false;

        // Scan ahead to see if this looks like: ? { ... } :
        // Limit iterations to prevent infinite loops on malformed input
        let mut scan_iterations = 0;
        const MAX_SCAN_ITERATIONS: usize = 10_000;
        loop {
            if scan_iterations >= MAX_SCAN_ITERATIONS {
                break;
            }
            scan_iterations += 1;
            match &parser.current_token {
                Token::Symbol('{') => brace_depth += 1,
                Token::Symbol('}') => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    } else {
                        // End of the block - check if next token is ':'
                        parser.next_token(); // consume '}'
                        if parser.current_token == Token::Symbol(':') {
                            found_colon_after_block = true;
                        }
                        break;
                    }
                }
                Token::Eof => break,
                _ => {}
            }
            parser.next_token();
        }

        // Restore parser state
        parser.restore_state(saved_state);
        parser.next_token(); // consume '{' again

        if found_colon_after_block {
            // This is a ternary conditional: ? { true_block } : { false_block }
            let true_block = super::blocks::continue_parsing_block(parser)?;

            if parser.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError(
                    "Expected ':' after true block in ternary conditional".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
            parser.next_token(); // consume ':'

            if parser.current_token != Token::Symbol('{') {
                return Err(CompileError::SyntaxError(
                    "Expected '{' for false block in ternary conditional".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
            parser.next_token(); // consume '{'

            let false_block = super::blocks::continue_parsing_block(parser)?;

            // Convert to pattern match with true/false patterns
            let arms = vec![
                MatchArm {
                    pattern: Pattern::Literal(Expression::Boolean(true)),
                    guard: None,
                    body: true_block,
                },
                MatchArm {
                    pattern: Pattern::Literal(Expression::Boolean(false)),
                    guard: None,
                    body: false_block,
                },
            ];

            return Ok(Expression::QuestionMatch { scrutinee, arms });
        }

        // Not a ternary - check if it's a pattern match or bool short form
        // First check if it could be a pattern (starts with identifier, number, _, -, etc.)
        let is_arrow_pattern_match = match &parser.current_token {
            Token::Integer(_)
            | Token::Float(_)
            | Token::StringLiteral(_)
            | Token::Underscore
            | Token::Symbol('.') => true,
            Token::Operator(op) if op == "-" => true,
            Token::Identifier(name) if primitives::is_boolean_literal(name) => true,
            _ => false,
        };

        if parser.current_token == Token::Pipe {
            let mut arms = vec![];
            while parser.current_token == Token::Pipe {
                parser.next_token();

                let mut patterns = vec![parser.parse_pattern()?];
                while parser.current_token == Token::Pipe
                    && parser.peek_token != Token::Pipe
                    && parser.peek_token != Token::Eof
                {
                    parser.next_token();
                    patterns.push(parser.parse_pattern()?);
                }
                let pattern = if patterns.len() == 1 {
                    patterns.remove(0)
                } else {
                    Pattern::Or(patterns)
                };

                let guard = if parser.current_token == Token::Operator("->".to_string()) {
                    parser.next_token();
                    Some(parser.parse_expression()?)
                } else {
                    None
                };

                let body = if parser.current_token == Token::Symbol('{') {
                    parse_match_arm_block_body(parser)?
                } else if parser.current_token == Token::Operator("=>".to_string()) {
                    parser.next_token(); // consume '=>'
                    parser.parse_expression()?
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected '{' or '=>' after pattern in match arm".to_string(),
                        Some(parser.current_span.clone()),
                    ));
                };
                arms.push(MatchArm {
                    pattern,
                    guard,
                    body,
                });
            }

            if parser.current_token != Token::Symbol('}') {
                return Err(CompileError::SyntaxError(
                    "Expected '}' after pattern match block".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
            parser.next_token(); // consume '}'

            return Ok(Expression::QuestionMatch { scrutinee, arms });
        } else if is_arrow_pattern_match {
            // Block-based pattern match: { pattern => expr, pattern => expr, ... }
            let mut arms = vec![];

            while parser.current_token != Token::Symbol('}') && parser.current_token != Token::Eof {
                let pattern = parser.parse_pattern()?;

                if parser.current_token != Token::Operator("=>".to_string()) {
                    return Err(CompileError::SyntaxError(
                        "Expected '=>' after pattern in block match".to_string(),
                        Some(parser.current_span.clone()),
                    ));
                }
                parser.next_token(); // consume '=>'

                let body = parse_match_arm_body(parser)?;

                arms.push(MatchArm {
                    pattern,
                    guard: None,
                    body,
                });

                if parser.current_token == Token::Symbol(',') {
                    parser.next_token();
                }
            }

            if parser.current_token != Token::Symbol('}') {
                return Err(CompileError::SyntaxError(
                    "Expected '}' after pattern match block".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
            parser.next_token(); // consume '}'

            return Ok(Expression::QuestionMatch { scrutinee, arms });
        } else {
            // Bool pattern short form - parse the rest of the block as statements
            let body = super::blocks::continue_parsing_block(parser)?;

            // Convert to standard pattern match with true pattern
            let arms = vec![
                MatchArm {
                    pattern: Pattern::Literal(Expression::Boolean(true)),
                    guard: None,
                    body,
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::Block(vec![]), // Empty block for else case
                },
            ];

            return Ok(Expression::QuestionMatch { scrutinee, arms });
        }
    }

    // Standard pattern matching with | pattern => expr
    if parser.current_token != Token::Pipe {
        return Err(CompileError::SyntaxError(
            "Expected '|' to start pattern matching arms or '{' for bool pattern".to_string(),
            Some(parser.current_span.clone()),
        ));
    }

    let mut arms = vec![];

    // Parse arms: | pattern => expr | pattern => expr ...
    while parser.current_token == Token::Pipe {
        parser.next_token(); // consume '|'

        // Parse pattern - could be single or multiple (or patterns)
        let mut patterns = vec![parser.parse_pattern()?];

        // Check for additional patterns separated by |
        while parser.current_token == Token::Pipe &&
                  parser.peek_token != Token::Pipe && // Not start of next arm
                  parser.peek_token != Token::Eof
        {
            // This is an or pattern - consume the | and parse the next pattern
            parser.next_token();
            patterns.push(parser.parse_pattern()?);
        }

        // Create the final pattern
        let pattern = if patterns.len() == 1 {
            patterns.remove(0)
        } else {
            Pattern::Or(patterns)
        };

        // Check for destructuring/guard with ->
        let guard = if parser.current_token == Token::Operator("->".to_string()) {
            parser.next_token();
            // Currently treats -> as guard; destructuring would need separate syntax
            Some(parser.parse_expression()?)
        } else {
            None
        };

        // Parse body - can be either { block } or => expr (for compatibility)
        let body = if parser.current_token == Token::Symbol('{') {
            parse_match_arm_block_body(parser)?
        } else if parser.current_token == Token::Operator("=>".to_string()) {
            // Legacy => syntax for compatibility
            parser.next_token(); // consume '=>'
                                 // Handle return statement in pattern arm specially
            if let Token::Identifier(id) = &parser.current_token {
                if id == "return" {
                    parser.next_token(); // consume 'return'
                                         // Wrap return in a special expression type or handle it differently
                                         // For now, we'll just use the return expression directly
                                         // In a full implementation, we'd need a Block expression type with statements
                    parser.parse_expression()?
                } else {
                    parser.parse_expression()?
                }
            } else {
                parser.parse_expression()?
            }
        } else {
            return Err(CompileError::SyntaxError(
                "Expected '{' or '=>' after pattern in match arm".to_string(),
                Some(parser.current_span.clone()),
            ));
        };

        arms.push(MatchArm {
            pattern,
            guard,
            body,
        });

        // Check if there are more arms
        if parser.current_token != Token::Pipe {
            break;
        }
    }
    Ok(Expression::QuestionMatch { scrutinee, arms })
}
