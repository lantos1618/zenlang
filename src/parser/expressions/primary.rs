use crate::ast::primitives;
use crate::ast::{AstType, Expression, Statement};
use crate::error::{CompileError, Result};
use crate::lexer::Token;
use crate::parser::core::Parser;
use crate::well_known::well_known;

// Use shared method chain parser from calls module
use super::calls::parse_method_chain;
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
        Token::AtBuiltin => super::literals::parse_special_identifier_with_ufc(parser, "@builtin"),
        Token::Identifier(name) => {
            let name = name.clone();
            parser.next_token();

            // Check for boolean and unit literals first (these don't chain)
            // Use centralized literal checks
            if primitives::is_boolean_literal(&name) {
                return Ok(Expression::Boolean(name == "true"));
            } else if name == "void" {
                // void is a unit value - like () in other languages
                return Ok(Expression::Unit);
            } else if primitives::is_null_literal(&name) {
                // null is an alias for None (Option::None)
                return Ok(Expression::None);
            }

            // Check for Vec<T, size>() constructor vs Vec<T> { ... } struct literal vs Vec<T>.method()
            if name == "Vec" && parser.current_token == Token::Operator("<".to_string()) {
                // Look ahead to see what follows Vec<...>
                let saved_state = parser.lexer.save_state();
                let saved_current_token = parser.current_token.clone();
                let saved_peek_token = parser.peek_token.clone();
                let saved_current_span = parser.current_span.clone();
                let saved_peek_span = parser.peek_span.clone();

                // Skip past the generic type: Vec<T> or Vec<T, U, ...>
                parser.next_token(); // consume '<'
                let mut depth = 1;
                while depth > 0 && parser.current_token != Token::Eof {
                    if parser.current_token == Token::Operator("<".to_string()) {
                        depth += 1;
                    } else if parser.current_token == Token::Operator(">".to_string()) {
                        depth -= 1;
                    } else if parser.current_token == Token::Operator(">>".to_string()) {
                        // Handle >> as two > tokens for nested generics
                        depth -= 2;
                    }
                    parser.next_token();
                }

                // Check what follows the generic type
                let is_struct_literal = parser.current_token == Token::Symbol('{');
                let is_method_call = parser.current_token == Token::Symbol('.');
                let is_constructor = parser.current_token == Token::Symbol('(');

                // Restore parser state
                parser.lexer.restore_state(saved_state);
                parser.current_token = saved_current_token;
                parser.peek_token = saved_peek_token;
                parser.current_span = saved_current_span;
                parser.peek_span = saved_peek_span;

                if !is_struct_literal && !is_method_call && is_constructor {
                    // Vec<T, size>() constructor - only if followed by ()
                    return super::collections::parse_vec_constructor(parser);
                }
                // Otherwise fall through to normal expression parsing (struct literal or method call)
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
            // Could be:
            // 1. Generic struct literal: Vec<T> { ... }
            // 2. Generic function call: vec_new<i32>()
            // 3. Comparison: x < y
            // 4. Collection types: HashMap<K,V>.new()
            let (name_with_generics, consumed_generics) = if parser.current_token
                == Token::Operator("<".to_string())
            {
                if name == "Array" {
                    (name.clone(), false)
                // Check if this is a generic type (collection or user-defined)
                // Using stdlib lookup first, then fallback to heuristic for generic args
                } else if crate::stdlib_types::stdlib_types().is_collection_type(&name)
                    || super::collections::looks_like_generic_type_args(parser)
                {
                    let type_args_str = super::literals::parse_generic_type_args_to_string(parser)?;
                    (format!("{}<{}>", name, type_args_str), true)
                } else {
                    (name.clone(), false)
                }
            } else {
                (name.clone(), false)
            };

            // Check for struct literal syntax: Name { field: value, ... } or Name ( field: value, ... )
            if parser.current_token == Token::Symbol('{') {
                return super::structs::parse_struct_literal(parser, name_with_generics, '{', '}');
            }

            // Check for struct literal with parentheses: Name ( field: value, ... )
            if parser.current_token == Token::Symbol('(') {
                // Peek ahead to distinguish struct literal from function call
                // Struct literal: Person ( age: 20, name: "John" )
                // Function call: Person ( arg1, arg2 )
                // We look for the pattern: identifier followed by ':'
                let saved_state = parser.lexer.save_state();
                let saved_token = parser.current_token.clone();
                let saved_peek = parser.peek_token.clone();

                parser.next_token(); // consume '('

                // The lexer skips whitespace automatically, so we just check the next token
                // Check if first token is identifier followed by ':'
                let looks_like_struct_literal = if let Token::Identifier(_) = &parser.current_token
                {
                    parser.next_token();
                    parser.current_token == Token::Symbol(':')
                } else {
                    false
                };

                // Restore state
                parser.lexer.restore_state(saved_state);
                parser.current_token = saved_token;
                parser.peek_token = saved_peek;

                if looks_like_struct_literal {
                    return super::structs::parse_struct_literal(
                        parser,
                        name_with_generics,
                        '(',
                        ')',
                    );
                }
                // Otherwise, fall through to function call handling
            }

            // Check for function call with generics: vec_new<i32>()
            if consumed_generics && parser.current_token == Token::Symbol('(') {
                return super::calls::parse_call_expression(parser, name_with_generics);
            }

            // Initialize expression based on special identifiers or regular names
            let mut expr = if name == "@std" {
                Expression::StdReference
            } else if name == "@this" {
                Expression::ThisReference
            } else if well_known().is_none(&name) {
                // Handle None without parentheses
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
                // If we consumed generics, use the name with generics
                // This handles cases like HashMap<K,V>.new()
                Expression::Identifier(name_with_generics)
            } else {
                Expression::Identifier(name)
            };

            // Handle member access, array indexing, and function calls
            loop {
                match &parser.current_token {
                    Token::Symbol('.') => {
                        parser.next_token(); // consume '.'

                        if let Token::Identifier(id) = &parser.current_token {
                            if id == "loop" {
                                expr = super::literals::parse_collection_loop(parser, expr)?;
                                continue;
                            }
                        }

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

                        let member_with_generics = if parser.current_token
                            == Token::Operator("<".to_string())
                            && super::collections::looks_like_generic_type_args(parser)
                        {
                            let type_args_str =
                                super::literals::parse_generic_type_args_to_string(parser)?;
                            format!("{}<{}>", member, type_args_str)
                        } else {
                            member.clone()
                        };

                        // Check for pointer-specific operations first
                        // Only treat .val and .addr as pointer operations if NOT followed by ()
                        // This allows user-defined .val() and .addr() methods to work
                        if member == "val" && parser.current_token != Token::Symbol('(') {
                            // Pointer dereference: ptr.val (not a method call)
                            expr = Expression::PointerDereference(Box::new(expr));
                        } else if member == "addr" && parser.current_token != Token::Symbol('(') {
                            // Pointer address: expr.addr (not a method call)
                            expr = Expression::PointerAddress(Box::new(expr));
                        } else if member == "ref" && parser.current_token == Token::Symbol('(') {
                            // Reference creation: expr.ref()
                            parser.next_token(); // consume '('
                            if parser.current_token != Token::Symbol(')') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ')' after '.ref('".to_string(),
                                    Some(parser.current_span.clone()),
                                ));
                            }
                            parser.next_token(); // consume ')'
                            expr = Expression::CreateReference(Box::new(expr));
                        } else if member == "mut_ref" && parser.current_token == Token::Symbol('(')
                        {
                            // Mutable reference creation: expr.mut_ref()
                            parser.next_token(); // consume '('
                            if parser.current_token != Token::Symbol(')') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ')' after '.mut_ref('".to_string(),
                                    Some(parser.current_span.clone()),
                                ));
                            }
                            parser.next_token(); // consume ')'
                            expr = Expression::CreateMutableReference(Box::new(expr));
                        } else if member == "raise" && parser.current_token == Token::Symbol('(') {
                            parser.next_token(); // consume '('
                            if parser.current_token != Token::Symbol(')') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ')' after 'raise('".to_string(),
                                    Some(parser.current_span.clone()),
                                ));
                            }
                            parser.next_token(); // consume ')'
                            expr = Expression::Raise(Box::new(expr));
                        } else if member == "step" && parser.current_token == Token::Symbol('(') {
                            // Handle range.step(n) syntax for stepped ranges
                            parser.next_token(); // consume '('
                            let step_value = parser.parse_expression()?;
                            if parser.current_token != Token::Symbol(')') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ')' after step value".to_string(),
                                    Some(parser.current_span.clone()),
                                ));
                            }
                            parser.next_token(); // consume ')'
                                                 // For now, treat as a method call - could be optimized later
                            expr = Expression::MethodCall {
                                object: Box::new(expr),
                                method: "step".to_string(),
                                type_args: vec![],
                                args: vec![step_value],
                            };
                        } else {
                            // Check if this could be an enum variant constructor (EnumName.Variant)
                            // First letter capitalized usually indicates a type/variant name
                            if member.chars().next().is_some_and(|c| c.is_uppercase()) {
                                // Check if the base is an identifier (potential enum name)
                                if let Expression::Identifier(enum_name) = &expr {
                                    // This looks like an enum variant constructor
                                    expr = Expression::EnumVariant {
                                        enum_name: enum_name.clone(),
                                        variant: member_with_generics,
                                        payload: None,
                                    };
                                } else {
                                    // Regular member access (use generics if present)
                                    expr = Expression::MemberAccess {
                                        object: Box::new(expr),
                                        member: member_with_generics,
                                    };
                                }
                            } else {
                                // Regular member access (use generics if present for method calls)
                                expr = Expression::MemberAccess {
                                    object: Box::new(expr),
                                    member: member_with_generics,
                                };
                            }
                        }
                    }
                    Token::Symbol('[') => {
                        // Array indexing
                        parser.next_token(); // consume '['
                        let index = parser.parse_expression()?;
                        if parser.current_token != Token::Symbol(']') {
                            return Err(CompileError::SyntaxError(
                                "Expected ']' after array index".to_string(),
                                Some(parser.current_span.clone()),
                            ));
                        }
                        parser.next_token(); // consume ']'
                        expr = Expression::ArrayIndex {
                            array: Box::new(expr),
                            index: Box::new(index),
                        };
                    }
                    Token::Symbol('(') => {
                        // Function call or enum variant constructor with payload
                        if let Expression::MemberAccess { object, member } = expr {
                            return super::calls::parse_call_expression_with_object(
                                parser, *object, member,
                            );
                        } else if let Expression::Identifier(name) = expr {
                            return super::calls::parse_call_expression(parser, name);
                        } else if let Expression::EnumVariant {
                            enum_name,
                            variant,
                            payload: None,
                        } = expr
                        {
                            // Check if this looks like a struct literal: Module.Struct(field: value, ...)
                            // vs an enum variant with payload: Module.Variant(expression)
                            // We peek ahead to see if after '(' we have 'identifier:'
                            let saved_state = parser.lexer.save_state();
                            let saved_token = parser.current_token.clone();
                            let saved_peek = parser.peek_token.clone();
                            let saved_span = parser.current_span.clone();
                            let saved_peek_span = parser.peek_span.clone();

                            parser.next_token(); // consume '('

                            // Check if first token is identifier followed by ':'
                            let looks_like_struct_literal =
                                if let Token::Identifier(_) = &parser.current_token {
                                    parser.next_token();
                                    parser.current_token == Token::Symbol(':')
                                } else {
                                    false
                                };

                            // Restore state
                            parser.lexer.restore_state(saved_state);
                            parser.current_token = saved_token;
                            parser.peek_token = saved_peek;
                            parser.current_span = saved_span;
                            parser.peek_span = saved_peek_span;

                            if looks_like_struct_literal {
                                // Parse as struct literal with qualified name
                                let qualified_name = format!("{}.{}", enum_name, variant);
                                return super::structs::parse_struct_literal(
                                    parser,
                                    qualified_name,
                                    '(',
                                    ')',
                                );
                            } else {
                                // This is an enum variant constructor with payload
                                parser.next_token(); // consume '('
                                let payload_expr = parser.parse_expression()?;
                                if parser.current_token != Token::Symbol(')') {
                                    return Err(CompileError::SyntaxError(
                                        "Expected ')' after enum variant payload".to_string(),
                                        Some(parser.current_span.clone()),
                                    ));
                                }
                                parser.next_token(); // consume ')'
                                expr = Expression::EnumVariant {
                                    enum_name,
                                    variant,
                                    payload: Some(Box::new(payload_expr)),
                                };
                            }
                        } else {
                            return Err(CompileError::SyntaxError(
                                "Unexpected expression type for function call".to_string(),
                                Some(parser.current_span.clone()),
                            ));
                        }
                    }
                    Token::Symbol('{') => {
                        // Check if this is a struct literal after a qualified name: Module.Struct { ... }
                        // EnumVariant here means we parsed Module.Struct as an enum variant,
                        // but it could actually be a struct initialization
                        if let Expression::EnumVariant {
                            enum_name,
                            variant,
                            payload: None,
                        } = &expr
                        {
                            let qualified_name = format!("{}.{}", enum_name, variant);
                            return super::structs::parse_struct_literal(
                                parser,
                                qualified_name,
                                '{',
                                '}',
                            );
                        }
                        // If it's not an EnumVariant, break and let outer code handle it
                        break;
                    }
                    _ => break,
                }
            }

            Ok(expr)
        }
        Token::Symbol('(') => {
            // Try to determine if this is a closure or a parenthesized expression
            // We'll use a simple heuristic: if we see (ident) { or (ident, ...) {
            // then it's likely a closure

            parser.next_token(); // consume '('

            // Check for empty closure: () { ... } or () => expr
            if parser.current_token == Token::Symbol(')') {
                parser.next_token(); // consume ')'

                // Check for arrow function syntax: () => expr
                if parser.current_token == Token::Operator("=>".to_string()) {
                    parser.next_token(); // consume '=>'
                    let body_expr = parser.parse_expression()?;
                    // Convert expression to block that returns the value
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
                    // It's a closure with no parameters: () { ... }
                    let body = super::blocks::parse_block_expression(parser)?;
                    return Ok(Expression::Closure {
                        params: vec![],
                        return_type: None,
                        body: Box::new(body),
                    });
                }
                // Check for return type: () ReturnType { ... }
                else if matches!(&parser.current_token, Token::Identifier(_)) {
                    // Check if this is a closure with return type or just the unit value
                    // Save state in case we need to backtrack
                    let saved_state = parser.lexer.save_state();
                    let saved_current_token = parser.current_token.clone();
                    let saved_peek_token = parser.peek_token.clone();
                    let saved_span = parser.current_span.clone();

                    // Try to parse as return type
                    if let Ok(return_type) = parser.parse_type() {
                        if parser.current_token == Token::Symbol('{') {
                            // It's a closure with return type: () ReturnType { ... }
                            let body = super::blocks::parse_block_expression(parser)?;
                            return Ok(Expression::Closure {
                                params: vec![],
                                return_type: Some(return_type),
                                body: Box::new(body),
                            });
                        }
                    }
                    // Not a closure, restore and return unit value
                    parser.lexer.restore_state(saved_state);
                    parser.current_token = saved_current_token;
                    parser.peek_token = saved_peek_token;
                    parser.current_span = saved_span;
                    return Ok(Expression::Unit);
                } else {
                    // Empty parens () as the unit value
                    return Ok(Expression::Unit);
                }
            }

            // Try to parse as closure with parameters
            // A closure looks like: (param) { ... } or (param1, param2) { ... }
            // or with types: (param: Type) { ... }
            let mut is_closure = false;
            let mut closure_return_type: Option<AstType> = None;
            let mut params = vec![];

            // IMPORTANT: Use lookahead to determine if this is a closure or expression
            // BEFORE consuming tokens. This prevents ambiguity errors.
            // If we see (identifier followed by an operator OR a '.', it's definitely an expression.
            // - Binary operator: (a + b)
            // - Member access: (foo.bar ...)
            let is_definitely_expression = if let Token::Identifier(name) = &parser.current_token {
                // Check what comes after the identifier using peek
                // It's an expression if followed by a binary operator or member access (.)
                parser.peek_token == Token::Symbol('.')
                    || matches!(
                        &parser.peek_token,
                        Token::Operator(op) if matches!(op.as_str(),
                            "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" |
                            "&&" | "||" | "&" | "|" | "^" | "<<" | ">>" | ".." | "..="
                        )
                    )
                    // Also check for closing paren immediately - single identifier in parens is an expression
                    || matches!(&parser.peek_token, Token::Symbol(')'))
                    // And check if the name itself suggests it's a value (lowercase first char)
                    || name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
            } else {
                // Non-identifier tokens like numbers, strings, etc. are definitely expressions
                !matches!(&parser.current_token, Token::Symbol(')'))
            };

            // If it's definitely an expression, parse it as such
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

                // Check for optional type annotation
                let param_type = if parser.current_token == Token::Symbol(':') {
                    parser.next_token(); // consume ':'
                    Some(parser.parse_type()?)
                } else {
                    None
                };

                params.push((first_param, param_type));

                // Check for more parameters
                while parser.current_token == Token::Symbol(',') {
                    parser.next_token(); // consume ','

                    if let Token::Identifier(param_name) = &parser.current_token {
                        let param = param_name.clone();
                        parser.next_token();

                        // Check for optional type annotation
                        let param_type = if parser.current_token == Token::Symbol(':') {
                            parser.next_token(); // consume ':'
                            Some(parser.parse_type()?)
                        } else {
                            None
                        };

                        params.push((param, param_type));
                    } else {
                        // Not a valid parameter list, reset and parse as expression
                        break;
                    }
                }

                // Check if we have a valid closure pattern
                if parser.current_token == Token::Symbol(')') {
                    parser.next_token(); // consume ')'

                    // Check for arrow function syntax: () => expr
                    if parser.current_token == Token::Operator("=>".to_string()) {
                        parser.next_token(); // consume '=>'
                                             // Parse the expression body
                        let body_expr = parser.parse_expression()?;
                        // Convert expression to block that returns the value
                        let body = Expression::Block(vec![Statement::Expression {
                            expr: body_expr,
                            span: Some(parser.current_span.clone()),
                        }]);
                        return Ok(Expression::Closure {
                            params,
                            return_type: None,
                            body: Box::new(body),
                        });
                    }
                    // Check for the body
                    else if parser.current_token == Token::Symbol('{') {
                        is_closure = true;
                        closure_return_type = None;
                    }
                    // Check for return type: (params) ReturnType { ... }
                    else if matches!(&parser.current_token, Token::Identifier(_)) {
                        // Return type specified, check for body after
                        let return_type = parser.parse_type()?;
                        if parser.current_token == Token::Symbol('{') {
                            is_closure = true;
                            closure_return_type = Some(return_type);
                        }
                    }
                }

                if is_closure {
                    // Parse the closure body
                    let body = super::blocks::parse_block_expression(parser)?;
                    Ok(Expression::Closure {
                        params,
                        return_type: closure_return_type,
                        body: Box::new(body),
                    })
                } else {
                    // Not a closure, we have an issue with parsing
                    // Since we consumed tokens trying to parse as closure, we can't backtrack
                    Err(CompileError::SyntaxError(
                        "Ambiguous syntax: Use explicit type annotations for closure parameters or parenthesize complex expressions".to_string(),
                        Some(parser.current_span.clone()),
                    ))
                }
            } else {
                // Not starting with an identifier, so it's definitely not a closure
                // Parse as a parenthesized expression
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
        Token::Symbol('[') => {
            // Array literal: [expr, expr, ...]
            super::collections::parse_array_literal(parser)
        }
        Token::Symbol('{') => {
            // Disambiguate between block expression and anonymous struct literal
            // Anonymous struct literals look like: { identifier: <value>, ... }
            // Block expressions contain statements

            // Save parser state for look-ahead
            let saved_state = parser.lexer.save_state();
            let saved_current_token = parser.current_token.clone();
            let saved_peek_token = parser.peek_token.clone();
            let saved_current_span = parser.current_span.clone();
            let saved_peek_span = parser.peek_span.clone();

            parser.next_token(); // consume '{'

            // Check if this looks like a struct literal
            let is_struct_literal = match &parser.current_token {
                Token::Symbol('}') => {
                    // Empty braces {} - treat as empty block for backwards compatibility
                    false
                }
                Token::Identifier(field_name) => {
                    // Check if followed by ':'
                    if parser.peek_token == Token::Symbol(':') {
                        // Look at what comes after the ':'
                        let _field = field_name.clone();
                        parser.next_token(); // consume identifier
                        parser.next_token(); // consume ':'

                        // If the next token is clearly a value (not a type), it's a struct literal
                        match &parser.current_token {
                            // Literals are definitely values
                            Token::StringLiteral(_) => true,
                            Token::Integer(_) | Token::Float(_) => true,

                            // Boolean literals
                            Token::Identifier(id) if id == "true" || id == "false" => true,

                            // None is a value (Option::None)
                            Token::Identifier(id) if well_known().is_none(id) => true,

                            // Lowercase identifiers that aren't primitive types are likely variables (values)
                            Token::Identifier(id) => {
                                let first_char = id.chars().next().unwrap_or('a');
                                // Use centralized primitive definitions
                                first_char.is_lowercase() && !primitives::is_primitive_name(id)
                            }

                            // Array literal suggests value context
                            Token::Symbol('[') => true,

                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // Restore parser state
            parser.lexer.restore_state(saved_state);
            parser.current_token = saved_current_token;
            parser.peek_token = saved_peek_token;
            parser.current_span = saved_current_span;
            parser.peek_span = saved_peek_span;

            if is_struct_literal {
                // Parse as anonymous struct literal (empty name)
                super::structs::parse_struct_literal(parser, String::new(), '{', '}')
            } else {
                // Block expression: { statements... }
                super::blocks::parse_block_expression(parser)
            }
        }
        _ => Err(CompileError::SyntaxError(
            format!("Unexpected token: {:?}", parser.current_token),
            Some(parser.current_span.clone()),
        )),
    }
}
