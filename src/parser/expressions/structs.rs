use crate::ast::Expression;
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

pub fn parse_struct_literal(
    parser: &mut Parser,
    name: String,
    _open_char: char,
    close_char: char,
) -> Result<Expression> {
    parser.next_token(); // consume opening char ('{' or '(')
    let mut fields = vec![];

    let close_token = Token::Symbol(close_char);
    while parser.current_token != close_token {
        // Parse field name
        let field_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(CompileError::SyntaxError(
                    "Expected field name in struct literal".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
        };
        parser.next_token();

        // Expect ':'
        if parser.current_token != Token::Symbol(':') {
            return Err(CompileError::SyntaxError(
                "Expected ':' after field name in struct literal".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token();

        // Parse field value
        let field_value = parser.parse_expression()?;
        fields.push((field_name, field_value));

        // Check for comma or end of struct
        if parser.current_token == Token::Symbol(',') {
            parser.next_token();
        } else if parser.current_token != close_token {
            // Allow trailing comma or no comma before closing delimiter
            // Only error if there's something unexpected
            if parser.current_token == Token::Eof {
                return Err(CompileError::SyntaxError(
                    format!(
                        "Unexpected end of file in struct literal, expected '{}'",
                        close_char
                    ),
                    Some(parser.current_span.clone()),
                ));
            }
            // If it's not a comma or closing delimiter, that's an error
            return Err(CompileError::SyntaxError(
                format!("Expected ',' or '{}' in struct literal", close_char),
                Some(parser.current_span.clone()),
            ));
        }
    }

    parser.next_token(); // consume closing char ('}' or ')')
    Ok(Expression::StructLiteral { name, fields })
}
