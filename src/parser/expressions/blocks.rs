use crate::ast::Expression;
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

pub fn parse_block_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume '{'
    continue_parsing_block(parser)
}

/// Continue parsing a block after '{' has already been consumed
pub fn continue_parsing_block(parser: &mut Parser) -> Result<Expression> {
    let mut statements = vec![];

    while parser.current_token != Token::Symbol('}') && parser.current_token != Token::Eof {
        statements.push(parser.parse_statement()?);
    }

    if parser.current_token != Token::Symbol('}') {
        return Err(CompileError::SyntaxError(
            "Expected '}' to close block expression".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token(); // consume '}'

    Ok(Expression::Block(statements))
}

pub fn parse_interpolated_string(_parser: &mut Parser, input: String) -> Result<Expression> {
    use crate::ast::StringPart;
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x01' {
            // Found interpolation start marker

            // Save any literal part before this interpolation
            if !current.is_empty() {
                parts.push(StringPart::Literal(current.clone()));
                current.clear();
            }

            // Parse the interpolated expression
            let mut expr_str = String::new();

            for ch in chars.by_ref() {
                if ch == '\x02' {
                    // Found interpolation end marker
                    break;
                }
                expr_str.push(ch);
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
    if parts
        .iter()
        .any(|p| matches!(p, StringPart::Interpolation(_)))
    {
        Ok(Expression::StringInterpolation { parts })
    } else {
        // No interpolation found, return a simple string
        Ok(Expression::String(input))
    }
}
