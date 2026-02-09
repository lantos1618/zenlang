use crate::ast::primitives;
use crate::ast::{AstType, Expression, Statement};
use crate::error::{CompileError, Result};
use crate::intrinsics::well_known;
use crate::lexer::Token;
use crate::parser::core::Parser;

// Use shared chain parsers from calls module
use super::calls::{parse_method_chain, parse_postfix_chain};
// Use control flow parsers
use super::control_flow::{
    parse_break_expression, parse_comptime_expression, parse_continue_expression,
    parse_loop_expression, parse_return_expression,
};

pub fn parse_primary_expression(parser: &mut Parser) -> Result<Expression> {
    match &parser.current_token {
        // Control flow expressions - delegated to control_flow.rs
        Token::Identifier(id) if id == "loop" => parse_loop_expression(parser),
        Token::Identifier(id) if id == "break" => parse_break_expression(parser),
        Token::Identifier(id) if id == "continue" => parse_continue_expression(parser),
        Token::Identifier(id) if id == "return" => parse_return_expression(parser),
        Token::Identifier(id) if id == "comptime" => parse_comptime_expression(parser),

        // Literal expressions
        Token::Integer(value_str) => {
            let value_str = value_str.clone();
            super::literals::parse_integer_literal(parser, &value_str)
        }
        Token::Float(value_str) => {
            let value_str = value_str.clone();
            super::literals::parse_float_literal(parser, &value_str)
        }
        Token::StringLiteral(value) => {
            let value = value.clone();
            super::literals::parse_string_literal(parser, &value)
        }
        Token::Symbol('.') => super::literals::parse_shorthand_enum_variant(parser),
        Token::AtStd => super::literals::parse_special_identifier_with_ufc(parser, "@std"),
        Token::AtThis => super::literals::parse_special_identifier_with_ufc(parser, "@this"),
        Token::AtBuiltin => super::literals::parse_special_identifier_with_ufc(
            parser,
            crate::intrinsics::INTRINSIC_PREFIX,
        ),
        Token::Identifier(name) => {
            let name = name.clone();
            parse_identifier_expression(parser, name)
        }
        Token::Symbol('(') => parse_parenthesized_or_closure(parser),
        Token::Symbol('[') => super::collections::parse_array_literal(parser),
        Token::Symbol('{') => parse_brace_expression(parser),
        _ => Err(CompileError::SyntaxError(
            format!("Unexpected token: {:?}", parser.current_token),
            Some(parser.current_span.clone()),
        )),
    }
}

/// Parse an expression starting with an identifier: variables, constructors,
/// struct literals, enum variants, function calls, and method chains.
fn parse_identifier_expression(parser: &mut Parser, name: String) -> Result<Expression> {
    parser.next_token();

    // Check for boolean and unit literals first (these don't chain)
    if primitives::is_boolean_literal(&name) {
        return Ok(Expression::Boolean(name == "true"));
    } else if name == "void" {
        return Ok(Expression::Unit);
    } else if primitives::is_null_literal(&name) {
        return Ok(Expression::None);
    }

    // Check for Vec<T, size>() constructor vs Vec<T> { ... } struct literal vs Vec<T>.method()
    if name == "Vec" && parser.current_token == Token::Operator("<".to_string()) {
        let saved = parser.save_state();

        // Skip past the generic type: Vec<T> or Vec<T, U, ...>
        parser.next_token(); // consume '<'
        let mut depth: i32 = 1;
        let mut iterations = 0;
        while depth > 0 && parser.current_token != Token::Eof && iterations < 1000 {
            iterations += 1;
            if parser.current_token == Token::Operator("<".to_string()) {
                depth += 1;
            } else if parser.current_token == Token::Operator(">".to_string()) {
                depth -= 1;
            } else if parser.current_token == Token::Operator(">>".to_string()) {
                depth -= 2;
            }
            parser.next_token();
        }

        let is_struct_literal = parser.current_token == Token::Symbol('{');
        let is_method_call = parser.current_token == Token::Symbol('.');
        let is_constructor = parser.current_token == Token::Symbol('(');

        parser.restore_state(saved);

        if !is_struct_literal && !is_method_call && is_constructor {
            return super::collections::parse_vec_constructor(parser);
        }
    }

    // Check for DynVec<T>() or DynVec<T1, T2, ...>() constructor
    if name == "DynVec" && parser.current_token == Token::Operator("<".to_string()) {
        return super::collections::parse_dynvec_constructor(parser);
    }

    // Check for Array<T>() constructor
    if name == "Array" && parser.current_token == Token::Operator("<".to_string()) {
        return super::collections::parse_array_constructor(parser);
    }

    // Check for Option type constructors: Some(value) and None
    let wk = well_known();
    if wk.is_some(&name) && parser.current_token == Token::Symbol('(') {
        parser.next_token(); // consume '('
        let value = parser.parse_expression()?;
        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                "Expected ')' after Some value".to_string(),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token(); // consume ')'
        return Ok(Expression::Some(Box::new(value)));
    } else if wk.is_none(&name) {
        return Ok(Expression::None);
    }

    // Check for enum variant syntax: EnumName::VariantName
    if parser.current_token == Token::Operator("::".to_string()) {
        parser.next_token(); // consume '::'

        let variant = match &parser.current_token {
            Token::Identifier(v) => v.clone(),
            _ => {
                return Err(CompileError::SyntaxError(
                    "Expected variant name after '::'".to_string(),
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

        return Ok(Expression::EnumVariant {
            enum_name: name,
            variant,
            payload,
        });
    }

    // Check for generic type parameters
    let (name_with_generics, consumed_generics) =
        if parser.current_token == Token::Operator("<".to_string()) {
            if name == "Array" {
                (name.clone(), false)
            } else if super::collections::looks_like_generic_type_args(parser) {
                let type_args_str = super::literals::parse_generic_type_args_to_string(parser)?;
                (format!("{}<{}>", name, type_args_str), true)
            } else {
                (name.clone(), false)
            }
        } else {
            (name.clone(), false)
        };

    // Check for struct literal syntax: Name { field: value, ... }
    if parser.current_token == Token::Symbol('{') {
        return super::structs::parse_struct_literal(parser, name_with_generics, '{', '}');
    }

    // Check for struct literal with parentheses: Name ( field: value, ... )
    if parser.current_token == Token::Symbol('(') {
        let saved = parser.save_state();

        parser.next_token(); // consume '('

        let looks_like_struct_literal = if let Token::Identifier(_) = &parser.current_token {
            parser.next_token();
            parser.current_token == Token::Symbol(':')
        } else {
            false
        };

        parser.restore_state(saved);

        if looks_like_struct_literal {
            return super::structs::parse_struct_literal(parser, name_with_generics, '(', ')');
        }
    }

    // Check for function call with generics: vec_new<i32>()
    if consumed_generics && parser.current_token == Token::Symbol('(') {
        return super::calls::parse_call_expression(parser, name_with_generics);
    }

    // Initialize expression based on special identifiers or regular names
    let expr = if name == "@std" {
        Expression::StdReference
    } else if name == "@this" {
        Expression::ThisReference
    } else if well_known().is_none(&name) {
        let wk = well_known();
        Expression::EnumVariant {
            enum_name: wk
                .get_variant_parent_name(&name)
                .unwrap_or(wk.option_name())
                .to_string(),
            variant: name.clone(),
            payload: None,
        }
    } else if consumed_generics {
        Expression::Identifier(name_with_generics)
    } else {
        Expression::Identifier(name)
    };

    // Handle member access, array indexing, and function calls via shared chain parser
    parse_postfix_chain(parser, expr)
}

/// Parse parenthesized expressions `(expr)` and closure expressions `(params) { body }`.
fn parse_parenthesized_or_closure(parser: &mut Parser) -> Result<Expression> {
    parser.next_token(); // consume '('

    // Check for empty closure: () { ... } or () => expr
    if parser.current_token == Token::Symbol(')') {
        parser.next_token(); // consume ')'

        if parser.current_token == Token::Operator("=>".to_string()) {
            parser.next_token(); // consume '=>'
            let body_expr = parser.parse_expression()?;
            let body = Expression::Block(vec![Statement::Expression {
                expr: body_expr,
                span: Some(parser.current_span.clone()),
            }]);
            return Ok(Expression::Closure {
                params: vec![],
                return_type: None,
                body: Box::new(body),
            });
        } else if parser.current_token == Token::Symbol('{') {
            let body = super::blocks::parse_block_expression(parser)?;
            return Ok(Expression::Closure {
                params: vec![],
                return_type: None,
                body: Box::new(body),
            });
        } else if matches!(&parser.current_token, Token::Identifier(_)) {
            let saved = parser.save_state();

            if let Ok(return_type) = parser.parse_type() {
                if parser.current_token == Token::Symbol('{') {
                    let body = super::blocks::parse_block_expression(parser)?;
                    return Ok(Expression::Closure {
                        params: vec![],
                        return_type: Some(return_type),
                        body: Box::new(body),
                    });
                }
            }
            parser.restore_state(saved);
            return Ok(Expression::Unit);
        } else {
            return Ok(Expression::Unit);
        }
    }

    // IMPORTANT: Use lookahead to determine if this is a closure or expression
    // BEFORE consuming tokens. This prevents ambiguity errors.
    let is_definitely_expression = if let Token::Identifier(name) = &parser.current_token {
        parser.peek_token == Token::Symbol('.')
            || matches!(
                &parser.peek_token,
                Token::Operator(op) if matches!(op.as_str(),
                    "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" |
                    "&&" | "||" | "&" | "|" | "^" | "<<" | ">>" | ".." | "..="
                )
            )
            || matches!(&parser.peek_token, Token::Symbol(')'))
            || name.chars().next().is_some_and(|c| c.is_lowercase())
    } else {
        !matches!(&parser.current_token, Token::Symbol(')'))
    };

    if is_definitely_expression {
        let expr = parser.parse_expression()?;
        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                format!(
                    "Expected ')' after parenthesized expression, got {:?}",
                    parser.current_token
                ),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token(); // consume ')'
        return parse_method_chain(parser, expr);
    }

    // Check if this looks like a closure parameter list
    if let Token::Identifier(param_name) = &parser.current_token {
        let first_param = param_name.clone();
        parser.next_token();

        let param_type = if parser.current_token == Token::Symbol(':') {
            parser.next_token(); // consume ':'
            Some(parser.parse_type()?)
        } else {
            None
        };

        let mut params = vec![(first_param, param_type)];
        let mut is_closure = false;
        let mut closure_return_type: Option<AstType> = None;

        while parser.current_token == Token::Symbol(',') {
            parser.next_token(); // consume ','

            if let Token::Identifier(param_name) = &parser.current_token {
                let param = param_name.clone();
                parser.next_token();

                let param_type = if parser.current_token == Token::Symbol(':') {
                    parser.next_token(); // consume ':'
                    Some(parser.parse_type()?)
                } else {
                    None
                };

                params.push((param, param_type));
            } else {
                break;
            }
        }

        if parser.current_token == Token::Symbol(')') {
            parser.next_token(); // consume ')'

            if parser.current_token == Token::Operator("=>".to_string()) {
                parser.next_token(); // consume '=>'
                let body_expr = parser.parse_expression()?;
                let body = Expression::Block(vec![Statement::Expression {
                    expr: body_expr,
                    span: Some(parser.current_span.clone()),
                }]);
                return Ok(Expression::Closure {
                    params,
                    return_type: None,
                    body: Box::new(body),
                });
            } else if parser.current_token == Token::Symbol('{') {
                is_closure = true;
                closure_return_type = None;
            } else if matches!(&parser.current_token, Token::Identifier(_)) {
                let return_type = parser.parse_type()?;
                if parser.current_token == Token::Symbol('{') {
                    is_closure = true;
                    closure_return_type = Some(return_type);
                }
            }
        }

        if is_closure {
            let body = super::blocks::parse_block_expression(parser)?;
            Ok(Expression::Closure {
                params,
                return_type: closure_return_type,
                body: Box::new(body),
            })
        } else {
            Err(CompileError::SyntaxError(
                "Ambiguous syntax: Use explicit type annotations for closure parameters or parenthesize complex expressions".to_string(),
                Some(parser.current_span.clone()),
            ))
        }
    } else {
        let expr = parser.parse_expression()?;
        if parser.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                format!(
                    "Expected ')' after parenthesized expression, got {:?}",
                    parser.current_token
                ),
                Some(parser.current_span.clone()),
            ));
        }
        parser.next_token(); // consume ')'
        parse_method_chain(parser, expr)
    }
}

/// Parse brace expressions: block expressions `{ stmts... }` or anonymous struct literals `{ field: value, ... }`.
fn parse_brace_expression(parser: &mut Parser) -> Result<Expression> {
    let saved = parser.save_state();

    parser.next_token(); // consume '{'

    let is_struct_literal = match &parser.current_token {
        Token::Symbol('}') => false,
        Token::Identifier(field_name) => {
            if parser.peek_token == Token::Symbol(':') {
                let _field = field_name.clone();
                parser.next_token(); // consume identifier
                parser.next_token(); // consume ':'

                match &parser.current_token {
                    Token::StringLiteral(_) => true,
                    Token::Integer(_) | Token::Float(_) => true,
                    Token::Identifier(id) if id == "true" || id == "false" => true,
                    Token::Identifier(id) if well_known().is_none(id) => true,
                    Token::Identifier(id) => {
                        id.chars().next().is_some_and(|c| c.is_lowercase())
                            && !primitives::is_primitive_name(id)
                    }
                    Token::Symbol('[') => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    };

    parser.restore_state(saved);

    if is_struct_literal {
        super::structs::parse_struct_literal(parser, String::new(), '{', '}')
    } else {
        super::blocks::parse_block_expression(parser)
    }
}
