//! Literal expression parsing: integers, floats, strings

#![allow(dead_code)]

use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

// Re-export parse_method_chain for use in this module
use super::calls::parse_method_chain;

// ============================================================================
// Literal Parsers
// ============================================================================

/// Parse integer literal with UFC support
/// Handles decimal, hex (0x), binary (0b), and octal (0o) formats
pub fn parse_integer_literal(parser: &mut Parser, value_str: &str) -> Result<Expression> {
    // Remove underscores used as digit separators
    let clean_str: String = value_str.chars().filter(|&c| c != '_').collect();

    // Helper closure to parse with a given radix and format name
    let parse_radix = |digits: &str, radix: u32, format_name: &str| -> Result<i64> {
        i64::from_str_radix(digits, radix).map_err(|_| {
            let error_msg = if format_name.is_empty() {
                format!("Invalid integer: {}", value_str)
            } else {
                format!("Invalid {} integer: {}", format_name, value_str)
            };
            CompileError::SyntaxError(error_msg, Some(parser.current_span.clone()))
        })
    };

    let value = if clean_str.starts_with("0x") || clean_str.starts_with("0X") {
        // Hexadecimal
        parse_radix(&clean_str[2..], 16, "hexadecimal")?
    } else if clean_str.starts_with("0b") || clean_str.starts_with("0B") {
        // Binary
        parse_radix(&clean_str[2..], 2, "binary")?
    } else if clean_str.starts_with("0o") || clean_str.starts_with("0O") {
        // Octal
        parse_radix(&clean_str[2..], 8, "octal")?
    } else {
        // Decimal
        parse_radix(&clean_str, 10, "")?
    };
    parser.next_token();

    let expr = if value <= i32::MAX as i64 && value >= i32::MIN as i64 {
        Expression::Integer32(value as i32)
    } else {
        Expression::Integer64(value)
    };

    parse_method_chain(parser, expr)
}

/// Parse float literal with UFC support
pub fn parse_float_literal(parser: &mut Parser, value_str: &str) -> Result<Expression> {
    let value = value_str.parse::<f64>().map_err(|_| {
        CompileError::SyntaxError(
            format!("Invalid float: {}", value_str),
            Some(parser.current_span.clone()),
        )
    })?;
    parser.next_token();

    parse_method_chain(parser, Expression::Float64(value))
}

/// Parse string literal with interpolation and UFC support
pub fn parse_string_literal(parser: &mut Parser, value: &str) -> Result<Expression> {
    let value = value.to_string();
    parser.next_token();

    let expr = if value.contains('\x01') {
        super::blocks::parse_interpolated_string(parser, value)?
    } else {
        Expression::String(value)
    };

    parse_method_chain(parser, expr)
}

/// Parse shorthand enum variant syntax: .VariantName or .VariantName(payload)
pub fn parse_shorthand_enum_variant(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume '.'

    let variant = match &parser.current_token {
        Token::Identifier(v) => v.clone(),
        _ => {
            return Err(CompileError::SyntaxError(
                "Expected variant name after '.'".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
    };
    parser.next_token();

    // Check for variant payload
    let payload = if parser.current_token == Token::Symbol('(') {
        parser.next_token(); // consume '('
        let expr = parser.parse_expression()?;
        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                "Expected ')' after enum variant payload".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token(); // consume ')'
        Some(Box::new(expr))
    } else {
        None
    };

    Ok(Expression::EnumLiteral { variant, payload })
}

pub fn parse_special_identifier_with_ufc(parser: &mut Parser, name: &str) -> Result<Expression> {
    parser.next_token();
    // Use the appropriate reference type based on the identifier
    let mut expr = match name {
        "@std" => Expression::StdReference,
        "@builtin" => Expression::BuiltinReference,
        _ => Expression::Identifier(name.to_string()),
    };

    while let Token::Symbol('.') = &parser.current_token {
        parser.next_token();

        let member = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(CompileError::SyntaxError(
                    "Expected identifier after '.'".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
        };
        parser.next_token();

        // Check for generic type arguments like sizeof<i32>
        let method_name = if parser.current_token == Token::Operator("<".to_string()) {
            // Parse generic type arguments and append to method name
            let type_args = parse_generic_type_args_to_string(parser)?;
            format!("{}<{}>", member, type_args)
        } else {
            member
        };

        if parser.current_token == Token::Symbol('(') {
            return super::calls::parse_call_expression_with_object(parser, expr, method_name);
        } else {
            expr = Expression::MemberAccess {
                object: Box::new(expr),
                member: method_name,
            };
        }
    }

    Ok(expr)
}

/// Parse generic type arguments like `<i32, String>` and return as Vec<AstType>
pub fn parse_generic_type_args(parser: &mut Parser) -> Result<Vec<AstType>> {
    parser.next_token(); // consume '<'
    let mut type_args = Vec::new();

    loop {
        type_args.push(parser.parse_type()?);

        if parser.current_token == Token::Symbol(',') {
            parser.next_token();
        } else if parser.try_consume_operator(">") {
            // Handles >> for nested generics
            break;
        } else {
            return Err(CompileError::SyntaxError(
                "Expected ',' or '>' in generic type arguments".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
    }

    Ok(type_args)
}

/// Parse generic type arguments and return as string (legacy - prefer parse_generic_type_args)
/// Kept for backwards compatibility with callers that need string representation
pub fn parse_generic_type_args_to_string(parser: &mut Parser) -> Result<String> {
    let type_args = parse_generic_type_args(parser)?;
    let type_args_str = type_args
        .iter()
        .map(|t| format!("{}", t))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(type_args_str)
}

pub fn parse_collection_loop(parser: &mut Parser, collection: Expression) -> Result<Expression> {
    parser.next_token();
    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' after '.loop'".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' for loop closure parameter".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    let param_name = if let Token::Identifier(p) = &parser.current_token {
        p.clone()
    } else {
        return Err(CompileError::SyntaxError(
            "Expected parameter name in loop closure".to_string(),
            Some(parser.current_span.clone()),
        ));
    };
    parser.next_token();

    let param_type = if parser.current_token == Token::Symbol(':') {
        parser.next_token();
        Some(parser.parse_type()?)
    } else {
        None
    };

    let param = (param_name, param_type);

    let index_param = if parser.current_token == Token::Symbol(',') {
        parser.next_token();
        if let Token::Identifier(idx) = &parser.current_token {
            let idx_name = idx.clone();
            parser.next_token();

            let idx_type = if parser.current_token == Token::Symbol(':') {
                parser.next_token();
                Some(parser.parse_type()?)
            } else {
                None
            };

            Some((idx_name, idx_type))
        } else {
            return Err(CompileError::SyntaxError(
                "Expected index parameter name after ','".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
    } else {
        None
    };

    if parser.current_token != Token::Symbol(')') {
        return Err(CompileError::SyntaxError(
            "Expected ')' after loop parameters".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    if parser.current_token != Token::Symbol('{') {
        return Err(CompileError::SyntaxError(
            "Expected '{' for loop body".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    let body = super::blocks::parse_block_expression(parser)?;

    if parser.current_token != Token::Symbol(')') {
        return Err(CompileError::SyntaxError(
            "Expected ')' to close loop call".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    Ok(Expression::CollectionLoop {
        collection: Box::new(collection),
        param,
        index_param,
        body: Box::new(body),
    })
}
