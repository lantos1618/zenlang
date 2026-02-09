use crate::ast::{AstType, Expression};
use crate::error::Result;
use crate::lexer::Token;
use crate::parser::core::Parser;

/// Parse a function call: `name(args...)`
/// This version takes pre-parsed type_args
pub fn parse_call_expression_with_type_args(
    parser: &mut Parser,
    function_name: String,
    type_args: Vec<AstType>,
) -> Result<Expression> {
    let arguments = parse_argument_list(parser)?;

    // Split "module.func" into structured fields if present
    let (module, name) = if let Some((m, f)) = function_name.split_once('.') {
        (Some(m.to_string()), f.to_string())
    } else {
        (None, function_name)
    };

    let expr = Expression::FunctionCall {
        module,
        name,
        type_args,
        args: arguments,
        span: Some(parser.current_span.clone()),
    };

    parse_method_chain(parser, expr)
}

/// Parse a function call: `name(args...)`
/// Legacy version - type_args may be embedded in function_name as a string
pub fn parse_call_expression(parser: &mut Parser, function_name: String) -> Result<Expression> {
    // Extract type args from name if embedded (e.g., "foo<i32>" -> "foo", [I32])
    let (base_name, type_args) = extract_type_args_from_name(&function_name)?;
    parse_call_expression_with_type_args(parser, base_name, type_args)
}

/// Extract type args embedded in a name string like "HashMap<i32, String>"
/// Returns (base_name, type_args)
fn extract_type_args_from_name(name: &str) -> Result<(String, Vec<AstType>)> {
    if let Some(angle_pos) = name.find('<') {
        let base_name = crate::name_utils::strip_generics(name).to_string();
        let type_args_str = &name[angle_pos + 1..name.len() - 1];
        let type_args = crate::parser::parse_type_args_from_string(type_args_str)?;
        Ok((base_name, type_args))
    } else {
        Ok((name.to_string(), vec![]))
    }
}

/// Parse a method call: `object.method(args...)`
pub fn parse_call_expression_with_object(
    parser: &mut Parser,
    object: Expression,
    method_name: String,
) -> Result<Expression> {
    parser.next_token(); // consume '('

    // Check for @std and @builtin syntax - these are syntactic constructs
    let is_builtin_syntax = match &object {
        Expression::BuiltinReference => true,
        Expression::MemberAccess { object: base, .. } => {
            matches!(
                base.as_ref(),
                Expression::StdReference | Expression::BuiltinReference
            )
        }
        _ => false,
    };

    let arguments = parse_arguments_until_close(parser)?;
    parser.next_token(); // consume ')'

    // Extract type args if embedded in method name (e.g., "new<i32>")
    let (base_method, type_args) = extract_type_args_from_name(&method_name)?;

    let expr = if is_builtin_syntax {
        build_builtin_call(&object, &base_method, arguments, type_args)?
    } else {
        Expression::MethodCall {
            object: Box::new(object),
            method: base_method,
            type_args,
            args: arguments,
            span: Some(parser.current_span.clone()),
        }
    };

    parse_method_chain(parser, expr)
}

/// Parse argument list including the parentheses: `(arg1, arg2, ...)`
fn parse_argument_list(parser: &mut Parser) -> Result<Vec<Expression>> {
    parser.next_token(); // consume '('
    let args = parse_arguments_until_close(parser)?;
    parser.next_token(); // consume ')'
    Ok(args)
}

/// Parse arguments until ')' is reached (does not consume the ')')
fn parse_arguments_until_close(parser: &mut Parser) -> Result<Vec<Expression>> {
    let mut arguments = vec![];

    if parser.current_token == Token::Symbol(')') {
        return Ok(arguments);
    }

    loop {
        arguments.push(parse_argument(parser)?);

        if parser.current_token == Token::Symbol(')') {
            break;
        }
        if parser.current_token != Token::Symbol(',') {
            return Err(parser.syntax_error("Expected ',' or ')' in function call"));
        }
        parser.next_token(); // consume ','
    }

    Ok(arguments)
}

/// Parse a single argument - handles closures with `(params) { body }` syntax
fn parse_argument(parser: &mut Parser) -> Result<Expression> {
    // Check for closure syntax: (params) { body }
    if parser.current_token == Token::Symbol('(') {
        if let Some(closure) = try_parse_closure(parser)? {
            return Ok(closure);
        }
    }
    parser.parse_expression()
}

/// Try to parse a closure `(params) { body }`, returns None if not a closure
fn try_parse_closure(parser: &mut Parser) -> Result<Option<Expression>> {
    let saved_state = parser.lexer.save_state();
    let saved_current = parser.current_token.clone();
    let saved_peek = parser.peek_token.clone();

    parser.next_token(); // consume '('
    let mut params = vec![];

    // Try parsing parameter list
    while parser.current_token != Token::Symbol(')') && parser.current_token != Token::Eof {
        if let Token::Identifier(param_name) = &parser.current_token {
            let name = param_name.clone();
            parser.next_token();

            // Optional type annotation
            let param_type = if parser.current_token == Token::Symbol(':') {
                parser.next_token();
                Some(parser.parse_type()?)
            } else {
                None
            };

            params.push((name, param_type));

            if parser.current_token == Token::Symbol(',') {
                parser.next_token();
            }
        } else {
            // Not a valid parameter - restore and return None
            parser.lexer.restore_state(saved_state);
            parser.current_token = saved_current;
            parser.peek_token = saved_peek;
            return Ok(None);
        }
    }

    if parser.current_token != Token::Symbol(')') {
        parser.lexer.restore_state(saved_state);
        parser.current_token = saved_current;
        parser.peek_token = saved_peek;
        return Ok(None);
    }

    parser.next_token(); // consume ')'

    // Must be followed by '{' for closure body
    if parser.current_token != Token::Symbol('{') {
        parser.lexer.restore_state(saved_state);
        parser.current_token = saved_current;
        parser.peek_token = saved_peek;
        return Ok(None);
    }

    let body = super::blocks::parse_block_expression(parser)?;
    Ok(Some(Expression::Closure {
        params,
        return_type: None,
        body: Box::new(body),
    }))
}

/// Build a function call for @std or @builtin syntax
/// Note: This function expects specific expression types. Passing other types indicates
/// a parser bug (the caller should have validated the expression type before calling).
fn build_builtin_call(
    object: &Expression,
    method_name: &str,
    args: Vec<Expression>,
    type_args: Vec<AstType>,
) -> Result<Expression> {
    match object {
        Expression::MemberAccess {
            object: base,
            member,
        } => match base.as_ref() {
            Expression::StdReference => Ok(Expression::FunctionCall {
                module: Some(member.clone()),
                name: method_name.to_string(),
                type_args,
                args,
                span: None,
            }),
            Expression::BuiltinReference => Ok(Expression::FunctionCall {
                // @builtin.module.method → module = "@builtin.module", name = "method"
                module: Some(format!(
                    "{}.{}",
                    crate::intrinsics::INTRINSIC_PREFIX,
                    member
                )),
                name: method_name.to_string(),
                type_args,
                args,
                span: None,
            }),
            other => Err(crate::error::CompileError::InternalError(
                format!(
                    "Parser bug: build_builtin_call received unexpected base expression: {:?}",
                    other
                ),
                None,
            )),
        },
        Expression::BuiltinReference => Ok(Expression::FunctionCall {
            module: Some(crate::intrinsics::INTRINSIC_PREFIX.to_string()),
            name: method_name.to_string(),
            type_args,
            args,
            span: None,
        }),
        other => Err(crate::error::CompileError::InternalError(
            format!(
                "Parser bug: build_builtin_call received unexpected object expression: {:?}",
                other
            ),
            None,
        )),
    }
}

/// Parse dot-only method chaining after an expression: `.member`, `.method()`, `.val`, etc.
/// Public so it can be reused by other expression parsers (literals, etc.)
pub fn parse_method_chain(parser: &mut Parser, mut expr: Expression) -> Result<Expression> {
    loop {
        if parser.current_token != Token::Symbol('.') {
            break;
        }
        parser.next_token(); // consume '.'

        // Check for .loop(...) collection loop
        if let Token::Identifier(id) = &parser.current_token {
            if id == "loop" {
                expr = super::literals::parse_collection_loop(parser, expr)?;
                continue;
            }
        }

        let member = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.syntax_error("Expected identifier after '.'")),
        };
        parser.next_token();

        expr = parse_member_access(parser, expr, member)?;
    }

    Ok(expr)
}

/// Parse full postfix chain after an identifier expression: `.member`, `[index]`, `(args)`, `{ fields }`.
/// Only used from parse_identifier_expression where `(`, `[`, `{` are valid continuations.
pub fn parse_postfix_chain(parser: &mut Parser, mut expr: Expression) -> Result<Expression> {
    loop {
        match &parser.current_token {
            Token::Symbol('.') => {
                parser.next_token(); // consume '.'

                // Check for .loop(...) collection loop
                if let Token::Identifier(id) = &parser.current_token {
                    if id == "loop" {
                        expr = super::literals::parse_collection_loop(parser, expr)?;
                        continue;
                    }
                }

                let member = match &parser.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(parser.syntax_error("Expected identifier after '.'")),
                };
                parser.next_token();

                expr = parse_member_access(parser, expr, member)?;
            }
            Token::Symbol('[') => {
                // Array indexing
                parser.next_token(); // consume '['
                let index = parser.parse_expression()?;
                parser.expect_symbol(']')?;
                expr = Expression::ArrayIndex {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            }
            Token::Symbol('(') => {
                // Function call or enum variant constructor with payload
                if let Expression::MemberAccess { object, member } = expr {
                    return parse_call_expression_with_object(parser, *object, member);
                } else if let Expression::Identifier(name) = expr {
                    return parse_call_expression(parser, name);
                } else if let Expression::EnumVariant {
                    enum_name,
                    variant,
                    payload: None,
                } = expr
                {
                    // Check if this looks like a struct literal: Module.Struct(field: value, ...)
                    // vs an enum variant with payload: Module.Variant(expression)
                    let saved = parser.save_state();

                    parser.next_token(); // consume '('

                    let looks_like_struct_literal =
                        if let Token::Identifier(_) = &parser.current_token {
                            parser.next_token();
                            parser.current_token == Token::Symbol(':')
                        } else {
                            false
                        };

                    parser.restore_state(saved);

                    if looks_like_struct_literal {
                        let qualified_name = format!("{}.{}", enum_name, variant);
                        return super::structs::parse_struct_literal(
                            parser,
                            qualified_name,
                            '(',
                            ')',
                        );
                    } else {
                        parser.next_token(); // consume '('
                        let payload_expr = parser.parse_expression()?;
                        parser.expect_symbol(')')?;
                        expr = Expression::EnumVariant {
                            enum_name,
                            variant,
                            payload: Some(Box::new(payload_expr)),
                        };
                    }
                } else {
                    // Expression doesn't support function call syntax here.
                    // Break and let the outer parser handle '(' as a new expression.
                    break;
                }
            }
            Token::Symbol('{') => {
                // Struct literal after a qualified name: Module.Struct { ... }
                if let Expression::EnumVariant {
                    enum_name,
                    variant,
                    payload: None,
                } = &expr
                {
                    let qualified_name = format!("{}.{}", enum_name, variant);
                    return super::structs::parse_struct_literal(parser, qualified_name, '{', '}');
                }
                // Not an EnumVariant — break and let outer code handle it
                break;
            }
            _ => break,
        }
    }

    Ok(expr)
}

/// Parse a single member access or method call after `.member` has been consumed
fn parse_member_access(
    parser: &mut Parser,
    expr: Expression,
    member: String,
) -> Result<Expression> {
    let is_call = parser.current_token == Token::Symbol('(');

    // Pointer operations (only when NOT followed by parentheses)
    if !is_call {
        match member.as_str() {
            "val" => return Ok(Expression::PointerDereference(Box::new(expr))),
            "addr" => return Ok(Expression::PointerAddress(Box::new(expr))),
            _ => {}
        }
    }

    // Built-in call operations (require parentheses)
    if is_call {
        match member.as_str() {
            "ref" => {
                parser.next_token(); // consume '('
                parser.expect_symbol(')')?;
                return Ok(Expression::CreateReference(Box::new(expr)));
            }
            "mut_ref" => {
                parser.next_token(); // consume '('
                parser.expect_symbol(')')?;
                return Ok(Expression::CreateMutableReference(Box::new(expr)));
            }
            "raise" => {
                parser.next_token(); // consume '('
                parser.expect_symbol(')')?;
                return Ok(Expression::Raise(Box::new(expr)));
            }
            "step" => {
                parser.next_token(); // consume '('
                let step_value = parser.parse_expression()?;
                parser.expect_symbol(')')?;
                return Ok(Expression::MethodCall {
                    object: Box::new(expr),
                    method: "step".to_string(),
                    type_args: vec![],
                    args: vec![step_value],
                    span: Some(parser.current_span.clone()),
                });
            }
            _ => {}
        }
    }

    // Check for generic type arguments on the member: member<T>
    let member_with_generics = if parser.current_token == Token::Operator("<".to_string())
        && super::collections::looks_like_generic_type_args(parser)
    {
        let type_args_str = super::literals::parse_generic_type_args_to_string(parser)?;
        format!("{}<{}>", member, type_args_str)
    } else {
        member.clone()
    };

    // Enum variant detection: EnumName.Variant where Variant starts with uppercase.
    // Must be checked BEFORE method call dispatch so Result.Ok(value) becomes
    // EnumVariant (with payload handled by parse_postfix_chain's '(' handler).
    if member.chars().next().is_some_and(|c| c.is_uppercase()) {
        if let Expression::Identifier(enum_name) = &expr {
            return Ok(Expression::EnumVariant {
                enum_name: enum_name.clone(),
                variant: member_with_generics,
                payload: None,
            });
        }
    }

    // Method call or member access
    if parser.current_token == Token::Symbol('(') {
        parse_call_expression_with_object(parser, expr, member_with_generics)
    } else {
        Ok(Expression::MemberAccess {
            object: Box::new(expr),
            member: member_with_generics,
        })
    }
}
