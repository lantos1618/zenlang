use crate::ast::{BinaryOperator, Expression};
use crate::error::Result;
use crate::lexer::Token;
use crate::parser::core::Parser;

/// Parse expressions for use in patterns - doesn't allow `|` as bitwise OR
/// since `|` is used as pattern alternative separator in pattern context
pub fn parse_pattern_expression(parser: &mut Parser) -> Result<Expression> {
    parse_binary_expression_impl(parser, 0, false)
}

pub fn parse_binary_expression(parser: &mut Parser, precedence: u8) -> Result<Expression> {
    parse_binary_expression_impl(parser, precedence, true)
}

fn parse_binary_expression_impl(
    parser: &mut Parser,
    precedence: u8,
    allow_pipe_as_bitor: bool,
) -> Result<Expression> {
    let mut left = parse_unary_expression(parser)?;

    loop {
        // Check for ternary operator '?' first, before checking for other operators
        // This ensures that expressions like "x > y ?" are handled correctly
        // The ternary operator has precedence lower than comparison operators
        if parser.current_token == Token::Question {
            // Handle pattern matching with low precedence (but higher than assignment)
            // This ensures x < y ? ... parses as (x < y) ? ...
            if precedence < 1 {
                // Pattern match has very low precedence
                parser.next_token(); // consume '?'
                left = super::patterns::parse_pattern_match(parser, left)?;
            } else {
                // If we're in a recursive call with higher precedence, break
                // so the outer call (with lower precedence) can handle the '?'
                break;
            }
        } else if let Token::Operator(op) = &parser.current_token {
            let op_clone = op.clone();
            let next_prec = get_precedence(&op_clone);
            if next_prec > precedence {
                parser.next_token(); // advance past the operator

                // Handle range expressions specially
                if op_clone == ".." || op_clone == "..=" {
                    let right =
                        parse_binary_expression_impl(parser, next_prec, allow_pipe_as_bitor)?;
                    left = Expression::Range {
                        start: Box::new(left),
                        end: Box::new(right),
                        inclusive: op_clone == "..=",
                    };
                } else {
                    // Parse right-hand side, but stop early if we encounter '?'
                    // so the outer call can handle the ternary operator
                    let right =
                        parse_binary_expression_impl(parser, next_prec, allow_pipe_as_bitor)?;
                    left = Expression::BinaryOp {
                        left: Box::new(left),
                        op: token_to_binary_operator(&op_clone)?,
                        right: Box::new(right),
                    };
                }
            } else {
                break;
            }
        // Handle Token::Pipe as bitwise OR in expression context
        // (Pattern context uses parse_pattern_expression which disables this)
        } else if allow_pipe_as_bitor && parser.current_token == Token::Pipe {
            let next_prec = get_precedence("|");
            if next_prec > precedence {
                parser.next_token(); // consume '|'
                let right = parse_binary_expression_impl(parser, next_prec, allow_pipe_as_bitor)?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::BitwiseOr,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        // Handle Token::Symbol('&') as bitwise AND in expression context
        } else if parser.current_token == Token::Symbol('&') {
            let next_prec = get_precedence("&");
            if next_prec > precedence {
                parser.next_token(); // consume '&'
                let right = parse_binary_expression_impl(parser, next_prec, allow_pipe_as_bitor)?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::BitwiseAnd,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(left)
}

fn parse_unary_expression(parser: &mut Parser) -> Result<Expression> {
    match &parser.current_token {
        Token::Operator(op) if op == "-" => {
            parser.next_token();
            let expr = parse_unary_expression(parser)?;
            // For now, represent unary minus as BinaryOp with 0 - expr
            Ok(Expression::BinaryOp {
                left: Box::new(Expression::Integer32(0)),
                op: BinaryOperator::Subtract,
                right: Box::new(expr),
            })
        }
        Token::Operator(op) if op == "!" => {
            parser.next_token();
            let expr = parse_unary_expression(parser)?;
            // For now, represent logical not as a function call
            Ok(Expression::FunctionCall {
                name: "not".to_string(),
                type_args: vec![],
                args: vec![expr],
            })
        }
        // Address-of operator: &expr
        Token::Symbol('&') => {
            parser.next_token();
            let expr = parse_unary_expression(parser)?;
            Ok(Expression::AddressOf(Box::new(expr)))
        }
        _ => parse_postfix_expression(parser),
    }
}

fn parse_postfix_expression(parser: &mut Parser) -> Result<Expression> {
    super::primary::parse_primary_expression(parser)
}

pub fn get_precedence(op: &str) -> u8 {
    match op {
        ".." | "..=" => 1,            // Range has lowest precedence
        "||" => 2,                    // Logical OR
        "&&" => 3,                    // Logical AND
        "|" => 4,                     // Bitwise OR
        "^" => 5,                     // Bitwise XOR
        "&" => 6,                     // Bitwise AND
        "==" | "!=" => 7,             // Equality
        "<" | "<=" | ">" | ">=" => 8, // Comparison
        "<<" | ">>" => 9,             // Bit shift
        "+" | "-" => 10,              // Addition/Subtraction
        "*" | "/" | "%" => 11,        // Multiplication/Division/Modulo
        _ => 0,
    }
}

fn token_to_binary_operator(op: &str) -> Result<BinaryOperator> {
    match op {
        "+" => Ok(BinaryOperator::Add),
        "-" => Ok(BinaryOperator::Subtract),
        "*" => Ok(BinaryOperator::Multiply),
        "/" => Ok(BinaryOperator::Divide),
        "%" => Ok(BinaryOperator::Modulo),
        "==" => Ok(BinaryOperator::Equals),
        "!=" => Ok(BinaryOperator::NotEquals),
        "<" => Ok(BinaryOperator::LessThan),
        "<=" => Ok(BinaryOperator::LessThanEquals),
        ">" => Ok(BinaryOperator::GreaterThan),
        ">=" => Ok(BinaryOperator::GreaterThanEquals),
        "&&" => Ok(BinaryOperator::And),
        "||" => Ok(BinaryOperator::Or),
        // Bitwise operators
        "&" => Ok(BinaryOperator::BitwiseAnd),
        "|" => Ok(BinaryOperator::BitwiseOr),
        "^" => Ok(BinaryOperator::BitwiseXor),
        "<<" => Ok(BinaryOperator::ShiftLeft),
        ">>" => Ok(BinaryOperator::ShiftRight),
        _ => Err(crate::error::CompileError::SyntaxError(
            format!("Unknown binary operator: {}", op),
            None,
        )),
    }
}
