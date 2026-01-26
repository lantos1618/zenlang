//! Control flow expression parsing: loop, break, continue, return, comptime
//! Extracted from primary.rs

use crate::ast::Expression;
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

/// Parse loop() function syntax
pub fn parse_loop_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume 'loop'
    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' after 'loop'".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token(); // consume '('

    // Parse the closure argument
    let loop_body = if parser.current_token == Token::Symbol('(') {
        // Parse closure parameter list
        parser.next_token(); // consume '('
        let mut params = vec![];

        // Handle empty parameter list
        if parser.current_token != Token::Symbol(')') {
            while parser.current_token != Token::Symbol(')') && parser.current_token != Token::Eof {
                if let Token::Identifier(param_name) = &parser.current_token {
                    let pname = param_name.clone();
                    parser.next_token();

                    // Check for optional type annotation
                    let param_type = if parser.current_token == Token::Symbol(':') {
                        parser.next_token(); // consume ':'
                        Some(parser.parse_type()?)
                    } else {
                        None
                    };

                    params.push((pname, param_type));

                    if parser.current_token == Token::Symbol(',') {
                        parser.next_token();
                    } else if parser.current_token != Token::Symbol(')') {
                        return Err(CompileError::SyntaxError(
                            "Expected ',' or ')' in closure parameters".to_string(),
                            Some(parser.current_span.clone()),
                        ));
                    }
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected parameter name in closure".to_string(),
                        Some(parser.current_span.clone()),
                    ));
                }
            }
        }

        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                "Expected ')' after closure parameters".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token(); // consume ')'

        // Parse closure body
        if parser.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' for closure body".to_string(),
                Some(parser.current_span.clone()),
            ));
        }

        let body = super::blocks::parse_block_expression(parser)?;
        Expression::Closure {
            params,
            return_type: None,
            body: Box::new(body),
        }
    } else {
        return Err(CompileError::SyntaxError(
            "Expected closure as argument to loop()".to_string(),
            Some(parser.current_span.clone()),
        ));
    };

    if parser.current_token != Token::Symbol(')') {
        return Err(CompileError::SyntaxError(
            "Expected ')' after loop closure".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token(); // consume ')'

    // Return a Loop expression
    Ok(Expression::Loop {
        body: Box::new(loop_body),
    })
}

/// Parse break expression
pub fn parse_break_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume 'break'
                         // Check for optional label
    let mut label = None;
    let mut value = None;

    // Check if next token could be a label (identifier not followed by an operator)
    if let Token::Identifier(label_name) = &parser.current_token {
        // Look ahead to see if this is a label or a value expression
        if matches!(&parser.peek_token, Token::Symbol(_) | Token::Eof) {
            label = Some(label_name.clone());
            parser.next_token();
        }
    }

    // Check if there's a value expression after break
    if !matches!(
        &parser.current_token,
        Token::Symbol('}') | Token::Eof | Token::Symbol(';')
    ) && label.is_none()
    {
        // This must be a value expression
        value = Some(Box::new(parser.parse_expression()?));
    }

    Ok(Expression::Break { label, value })
}

/// Parse continue expression
pub fn parse_continue_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume 'continue'
    let label = if let Token::Identifier(label_name) = &parser.current_token {
        let label_name = label_name.clone();
        parser.next_token();
        Some(label_name)
    } else {
        None
    };
    Ok(Expression::Continue { label })
}

/// Parse return expression
pub fn parse_return_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume 'return'
    let value = if !matches!(
        &parser.current_token,
        Token::Symbol('}') | Token::Eof | Token::Symbol(';')
    ) {
        Box::new(parser.parse_expression()?)
    } else {
        // Return void/unit if no expression
        Box::new(Expression::Block(vec![]))
    };
    Ok(Expression::Return(value))
}

/// Parse comptime expression
pub fn parse_comptime_expression(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume 'comptime'
    let expr = parser.parse_expression()?;
    Ok(Expression::Comptime(Box::new(expr)))
}
