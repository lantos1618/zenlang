use super::*;

mod forms;

impl Parser {
    /// Parse prefix / atom expressions.
    pub(super) fn parse_prefix(&mut self) -> Result<Expression, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            // Unary operators
            Token::Minus => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Not => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Tilde => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span: s,
                })
            }

            // Literals
            Token::IntLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::IntLiteral(v) = tok {
                    Ok(Expression::IntLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::FloatLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::FloatLiteral(v) = tok {
                    Ok(Expression::FloatLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::StringLiteral(v) = tok {
                    Ok(Expression::StringLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }

            // String interpolation
            Token::StringChunk(_) | Token::InterpolationStart => self.parse_string_interpolation(),

            // Identifier (or keyword-like: true, false, break, continue, loop, cast)
            Token::Identifier(ref name) => match name.as_str() {
                "true" => {
                    let (_, span) = self.advance();
                    Ok(Expression::BoolLiteral { value: true, span })
                }
                "false" => {
                    let (_, span) = self.advance();
                    Ok(Expression::BoolLiteral { value: false, span })
                }
                "return" => {
                    let (_, span) = self.advance();
                    Err(CompileError::Syntax(
                        "return keyword has been removed; use the final expression in the block"
                            .into(),
                        Some(span),
                    ))
                }
                "break" => {
                    let (_, span) = self.advance();
                    Ok(Expression::Break { span })
                }
                "continue" => {
                    let (_, span) = self.advance();
                    Ok(Expression::Continue { span })
                }
                "loop" => {
                    let (_, span) = self.advance();
                    self.parse_loop(span)
                }
                "cast" => {
                    let (_, span) = self.advance();
                    self.parse_cast(span)
                }
                _ => {
                    let name = name.clone();
                    let (_, span) = self.advance();
                    Ok(Expression::Identifier { name, span })
                }
            },

            // @this.defer(expr) or other @ tokens
            Token::AtThis => {
                let (_, span) = self.advance();
                // @this.defer(expr)
                if matches!(self.peek(), Token::Dot) {
                    self.advance(); // consume .
                    let (method, _) = self.expect_identifier()?;
                    if method == "defer" {
                        self.expect(&Token::LParen)?;
                        let expr = self.parse_expression()?;
                        let end = self.expect(&Token::RParen)?;
                        return Ok(Expression::Defer {
                            expr: Box::new(expr),
                            span: span.merge(end),
                        });
                    }
                    // Other @this.method calls
                    return Err(CompileError::Syntax(
                        format!("unknown @this method: {}", method),
                        Some(span),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "@this".to_string(),
                    span,
                })
            }

            Token::Dot => self.parse_shorthand_enum_variant_expr(),

            // Module-qualified: @builtin.func(args), @std.mod.func(args)
            Token::AtBuiltin => {
                let (_, span) = self.advance();
                self.expect(&Token::Dot)?;
                let (func_name, _) = self.expect_identifier()?;

                // Check for type args
                let type_args = if matches!(self.peek(), Token::Lt) {
                    self.parse_type_arg_list()?
                } else {
                    Vec::new()
                };

                self.expect(&Token::LParen)?;
                let args = self.parse_arg_list()?;
                let end = self.expect(&Token::RParen)?;

                Ok(Expression::FunctionCall {
                    name: func_name,
                    module: Some("@builtin".to_string()),
                    type_args,
                    args,
                    span: span.merge(end),
                })
            }

            Token::AtStd => {
                let (_, span) = self.advance();
                self.expect(&Token::Dot)?;
                let mut module_parts = Vec::new();
                let (first, _) = self.expect_identifier()?;
                module_parts.push(first);

                // Collect module.sub.func path
                while matches!(self.peek(), Token::Dot) {
                    let saved = self.pos;
                    self.advance(); // consume .
                    if let Token::Identifier(_) = self.peek() {
                        let (part, _) = self.expect_identifier()?;
                        module_parts.push(part);
                    } else {
                        self.pos = saved;
                        break;
                    }
                }

                // Last part is the function name
                let func_name = module_parts.pop().unwrap();
                let module = if module_parts.is_empty() {
                    "@std".to_string()
                } else {
                    format!("@std.{}", module_parts.join("."))
                };

                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    let end = self.expect(&Token::RParen)?;
                    Ok(Expression::FunctionCall {
                        name: func_name,
                        module: Some(module),
                        type_args: Vec::new(),
                        args,
                        span: span.merge(end),
                    })
                } else {
                    // Might be a member access
                    Ok(Expression::MemberAccess {
                        object: Box::new(Expression::Identifier { name: module, span }),
                        field: func_name,
                        span: span.merge(self.prev_span()),
                    })
                }
            }

            // Parenthesized expression or closure
            Token::LParen => {
                // Try to detect if this is a closure: `(params) ret? { body }`
                // vs a grouped expression: `(expr)`
                if self.is_closure() {
                    self.parse_closure()
                } else {
                    self.advance(); // consume (
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen)?;
                    Ok(expr)
                }
            }

            // Block expression
            Token::LBrace => self.parse_block_expression(),

            // Array literal
            Token::LBracket => {
                let (_, span) = self.advance();
                let mut elements = Vec::new();
                loop {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBracket) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                let end = self.expect(&Token::RBracket)?;
                Ok(Expression::ArrayLiteral {
                    elements,
                    span: span.merge(end),
                })
            }

            _ => {
                let (tok, span) = self.advance();
                Err(CompileError::Syntax(
                    format!("unexpected token {:?} in expression", tok),
                    Some(span),
                ))
            }
        }
    }
}
