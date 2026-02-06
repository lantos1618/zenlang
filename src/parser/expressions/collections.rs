use crate::ast::Expression;
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;

pub fn looks_like_generic_type_args(parser: &Parser) -> bool {
    // Try to determine if this is generic type args Vec<T> or vec_new<i32> vs comparison x < y
    // Heuristics:
    // 1. If next token is a type-like identifier (uppercase or known types like i32), likely generic
    // 2. If next token is '(' it's a function type like () ReturnType
    // 3. If next token is '[' it's an array type like [i32]
    // 4. If next token is '*' or '&' it's a pointer/reference type
    if let Token::Operator(op) = &parser.current_token {
        if op == "<" {
            match &parser.peek_token {
                Token::Identifier(name) => {
                    // Check for type-like names
                    // Token::Identifier is guaranteed non-empty by the lexer
                    // Types typically start with uppercase or are primitive types
                    return name.chars().next().is_some_and(|c| c.is_uppercase())
                        || name == "i8"
                        || name == "i16"
                        || name == "i32"
                        || name == "i64"
                        || name == "u8"
                        || name == "u16"
                        || name == "u32"
                        || name == "u64"
                        || name == "f32"
                        || name == "f64"
                        || name == "bool"
                        || name == "string";
                }
                // Function type: <() ReturnType> or <(param: Type) ReturnType>
                Token::Symbol('(') => return true,
                // Array type: <[T]> or <[T; N]>
                Token::Symbol('[') => return true,
                // Pointer type: <*T> or <&T>
                Token::Symbol('*') | Token::Symbol('&') => return true,
                Token::Operator(op) if op == "*" => return true,
                // Anonymous struct type: <{field: Type}>
                Token::Symbol('{') => return true,
                _ => {}
            }
        }
    }
    false
}

pub fn parse_array_literal(parser: &mut Parser) -> Result<Expression> {
    // Consume '['
    parser.next_token();

    let mut elements = Vec::new();

    // Handle empty array
    if parser.current_token == Token::Symbol(']') {
        parser.next_token();
        return Ok(Expression::ArrayLiteral(elements));
    }

    // Parse first element
    elements.push(parser.parse_expression()?);

    // Parse remaining elements
    while parser.current_token == Token::Symbol(',') {
        parser.next_token(); // consume ','

        // Allow trailing comma
        if parser.current_token == Token::Symbol(']') {
            break;
        }

        elements.push(parser.parse_expression()?);
    }

    // Expect closing ']'
    if parser.current_token != Token::Symbol(']') {
        return Err(CompileError::SyntaxError(
            format!(
                "Expected ']' in array literal, got {:?}",
                parser.current_token
            ),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    Ok(Expression::ArrayLiteral(elements))
}

pub fn parse_vec_constructor(parser: &mut Parser) -> Result<Expression> {
    // Consume '<'
    parser.next_token();

    // Parse element type
    let element_type = parser.parse_type()?;

    // Expect comma
    if parser.current_token != Token::Symbol(',') {
        return Err(CompileError::SyntaxError(
            "Expected ',' after element type in Vec<T, size>".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    // Parse size (must be integer literal)
    let size = match &parser.current_token {
        Token::Integer(size_str) => size_str.parse::<usize>().map_err(|_| {
            CompileError::SyntaxError(
                format!("Invalid Vec size: {}", size_str),
                Some(parser.current_span.clone()),
            )
        })?,
        _ => {
            return Err(CompileError::SyntaxError(
                "Expected integer literal for Vec size".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
    };
    parser.next_token();

    // Expect '>' (handles >> for nested generics)
    parser.expect_operator(">")?;

    // Expect '()'
    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' after Vec<T, size>".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    // Parse optional initial values
    let initial_values = if parser.current_token == Token::Symbol(')') {
        None
    } else {
        let mut values = vec![];
        loop {
            values.push(parser.parse_expression()?);

            if parser.current_token == Token::Symbol(')') {
                break;
            } else if parser.current_token == Token::Symbol(',') {
                parser.next_token();
            } else {
                return Err(CompileError::SyntaxError(
                    "Expected ',' or ')' in Vec constructor arguments".to_string(),
                    Some(parser.current_span.clone()),
                ));
            }
        }
        Some(values)
    };

    if parser.current_token != Token::Symbol(')') {
        return Err(CompileError::SyntaxError(
            "Expected ')' after Vec constructor arguments".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    Ok(Expression::VecConstructor {
        element_type,
        size,
        initial_values,
    })
}

pub fn parse_dynvec_constructor(parser: &mut Parser) -> Result<Expression> {
    // Consume '<'
    parser.next_token();

    // Parse element types (can be multiple for mixed variant vectors)
    let mut element_types = vec![];
    loop {
        element_types.push(parser.parse_type()?);

        // Handle >> for nested generics
        if parser.try_consume_operator(">") {
            break;
        } else if parser.current_token == Token::Symbol(',') {
            parser.next_token();
        } else {
            return Err(CompileError::SyntaxError(
                "Expected ',' or '>' in DynVec type arguments".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
    }

    // Expect '('
    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' after DynVec<T>".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    // Check if allocator is provided (optional for now, defaults to malloc)
    let (allocator, initial_capacity) = if parser.current_token == Token::Symbol(')') {
        // Empty constructor - use default allocator (0 as placeholder, malloc in codegen)
        (Expression::Integer64(0), None)
    } else {
        // Parse allocator expression
        let alloc = parser.parse_expression()?;

        // Parse optional initial capacity
        let capacity = if parser.current_token == Token::Symbol(',') {
            parser.next_token();
            Some(Box::new(parser.parse_expression()?))
        } else {
            None
        };

        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                "Expected ')' after DynVec constructor arguments".to_string(),
                Some(parser.current_span.clone()),
            ));
        }

        (alloc, capacity)
    };

    parser.next_token();

    Ok(Expression::DynVecConstructor {
        element_types,
        allocator: Box::new(allocator),
        initial_capacity,
    })
}

pub fn parse_array_constructor(parser: &mut Parser) -> Result<Expression> {
    // Consume '<'
    parser.next_token();

    // Parse element type - this could be nested like Option<i32>
    let element_type = parser.parse_type()?;

    // Expect '>' (handles >> for nested generics)
    parser.expect_operator(">")?;

    // Expect '('
    if parser.current_token != Token::Symbol('(') {
        return Err(CompileError::SyntaxError(
            "Expected '(' after Array<T>".to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    // Array constructor can take optional arguments (initial capacity, default value, etc.)
    // For now, we'll support empty constructor Array<T>()
    if parser.current_token != Token::Symbol(')') {
        return Err(CompileError::SyntaxError(
            "Expected ')' after Array constructor (only empty constructor supported for now)"
                .to_string(),
            Some(parser.current_span.clone()),
        ));
    }
    parser.next_token();

    // Create an ArrayConstructor expression
    Ok(Expression::ArrayConstructor { element_type })
}
