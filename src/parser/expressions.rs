use super::*;

impl Parser {
    // ── Expressions (Pratt parser) ────────────────────────────

    pub(super) fn parse_expression(&mut self) -> Result<Expression, CompileError> {
        self.parse_expr_bp(0)
    }

    /// Pratt parser: parse expression with minimum binding power.
    pub(super) fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expression, CompileError> {
        self.skip_newlines();
        let mut lhs = self.parse_prefix()?;

        loop {
            self.skip_newlines_if_continuation();

            // Check for postfix operators / calls / access
            match self.peek() {
                // Method call / field access: .name or .name(args)
                Token::Dot => {
                    let (l_bp, _) = postfix_bp();
                    if l_bp < min_bp {
                        break;
                    }
                    lhs = self.parse_dot_suffix(lhs)?;
                    continue;
                }

                // Index access: expr[index]
                Token::LBracket => {
                    let (l_bp, _) = postfix_bp();
                    if l_bp < min_bp {
                        break;
                    }
                    self.advance();
                    let index = self.parse_expression()?;
                    let end = self.expect(&Token::RBracket)?;
                    let span = lhs.span().merge(end);
                    lhs = Expression::IndexAccess {
                        object: Box::new(lhs),
                        index: Box::new(index),
                        span,
                    };
                    continue;
                }

                // Struct literal: Name { field: value, ... }
                Token::LBrace => {
                    if let Expression::Identifier {
                        ref name,
                        span: id_span,
                    } = lhs
                    {
                        if first_char_is_upper(name) {
                            let (l_bp, _) = postfix_bp();
                            if l_bp < min_bp {
                                break;
                            }
                            let name = name.clone();
                            lhs = self.parse_struct_literal(name, id_span)?;
                            continue;
                        }
                    }
                }

                // Function call: expr(args) -- only when lhs is identifier
                Token::LParen => {
                    if let Expression::Identifier {
                        ref name,
                        span: id_span,
                    } = lhs
                    {
                        let (l_bp, _) = postfix_bp();
                        if l_bp < min_bp {
                            break;
                        }
                        let name = name.clone();
                        self.advance(); // consume (
                        let args = self.parse_arg_list()?;
                        let end = self.expect(&Token::RParen)?;
                        let span = id_span.merge(end);
                        lhs = Expression::FunctionCall {
                            name,
                            module: None,
                            type_args: Vec::new(),
                            args,
                            span,
                        };
                        continue;
                    }
                }

                // Generic function call: name<T, U>(args)
                Token::Lt => {
                    if let Expression::Identifier {
                        ref name,
                        span: id_span,
                    } = lhs
                    {
                        let (l_bp, _) = postfix_bp();
                        if l_bp < min_bp {
                            break;
                        }

                        let saved = self.pos;
                        if let Ok(type_args) = self.parse_type_arg_list() {
                            if matches!(self.peek(), Token::LParen) {
                                let name = name.clone();
                                self.advance(); // consume (
                                let args = self.parse_arg_list()?;
                                let end = self.expect(&Token::RParen)?;
                                let span = id_span.merge(end);
                                lhs = Expression::FunctionCall {
                                    name,
                                    module: None,
                                    type_args,
                                    args,
                                    span,
                                };
                                continue;
                            }
                        }
                        self.pos = saved;
                    }
                }

                // Match/conditional: expr ?
                Token::Question => {
                    let (l_bp, _) = (2, 1);
                    if l_bp < min_bp {
                        break;
                    }
                    lhs = self.parse_match_or_while(lhs)?;
                    continue;
                }

                // `as` cast: expr as Type
                Token::Identifier(ref s) if s == "as" => {
                    let (l_bp, _) = (12, 13);
                    if l_bp < min_bp {
                        break;
                    }
                    self.advance(); // consume `as`
                    let target_type = self.parse_type()?;
                    let span = lhs.span().merge(self.prev_span());
                    lhs = Expression::Cast {
                        expr: Box::new(lhs),
                        target_type,
                        span,
                    };
                    continue;
                }

                _ => {}
            }

            // Infix binary operators
            if let Some((l_bp, r_bp)) = infix_bp(self.peek()) {
                if l_bp < min_bp {
                    break;
                }

                let op = match self.peek() {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Sub,
                    Token::Star => BinaryOp::Mul,
                    Token::Slash => BinaryOp::Div,
                    Token::Percent => BinaryOp::Mod,
                    Token::Eq => BinaryOp::Eq,
                    Token::NotEq => BinaryOp::NotEq,
                    Token::Lt => BinaryOp::Lt,
                    Token::Gt => BinaryOp::Gt,
                    Token::LtEq => BinaryOp::LtEq,
                    Token::GtEq => BinaryOp::GtEq,
                    Token::And => BinaryOp::And,
                    Token::Or => BinaryOp::Or,
                    Token::BitAnd => BinaryOp::BitAnd,
                    Token::Pipe => BinaryOp::BitOr,
                    Token::BitXor => BinaryOp::BitXor,
                    Token::ShiftLeft => BinaryOp::ShiftLeft,
                    Token::ShiftRight => BinaryOp::ShiftRight,
                    _ => break,
                };

                self.advance(); // consume operator
                let rhs = self.parse_expr_bp(r_bp)?;
                let span = lhs.span().merge(rhs.span());
                lhs = Expression::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                    span,
                };
                continue;
            }

            // Range operators: .. and ..=
            match self.peek() {
                Token::DotDot => {
                    let (l_bp, r_bp) = (3, 4);
                    if l_bp < min_bp {
                        break;
                    }
                    self.advance();
                    let rhs = self.parse_expr_bp(r_bp)?;
                    let span = lhs.span().merge(rhs.span());
                    lhs = Expression::Range {
                        start: Box::new(lhs),
                        end: Box::new(rhs),
                        inclusive: false,
                        span,
                    };
                    continue;
                }
                Token::DotDotEq => {
                    let (l_bp, r_bp) = (3, 4);
                    if l_bp < min_bp {
                        break;
                    }
                    self.advance();
                    let rhs = self.parse_expr_bp(r_bp)?;
                    let span = lhs.span().merge(rhs.span());
                    lhs = Expression::Range {
                        start: Box::new(lhs),
                        end: Box::new(rhs),
                        inclusive: true,
                        span,
                    };
                    continue;
                }
                _ => {}
            }

            break;
        }

        Ok(lhs)
    }

    // ── Dot suffix (field access, method call, module call) ───

    fn parse_dot_suffix(&mut self, lhs: Expression) -> Result<Expression, CompileError> {
        self.advance(); // consume .

        // Handle enum variant: Name.Variant
        let (name, name_span) = self.expect_identifier()?;

        if let Expression::Identifier {
            name: ref enum_name,
            span: id_span,
        } = lhs
        {
            if first_char_is_upper(enum_name) && first_char_is_upper(&name) {
                let payload = if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen)?;
                    Some(Box::new(expr))
                } else {
                    None
                };
                let span = id_span.merge(self.prev_span());
                return Ok(Expression::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: name,
                    payload,
                    span,
                });
            }
        }

        // Check for method call: expr.name(args)
        if matches!(self.peek(), Token::LParen) {
            self.advance();
            let args = self.parse_arg_list()?;
            let end = self.expect(&Token::RParen)?;
            let span = lhs.span().merge(end);

            // If lhs is an identifier, this could be module.func(args) or ufc
            if let Expression::Identifier {
                name: ref _mod_name,
                span: _,
            } = lhs
            {
                return Ok(Expression::MethodCall {
                    receiver: Box::new(lhs),
                    method: name,
                    type_args: Vec::new(),
                    args,
                    span,
                });
            }

            return Ok(Expression::MethodCall {
                receiver: Box::new(lhs),
                method: name,
                type_args: Vec::new(),
                args,
                span,
            });
        }

        // Check if this is a struct literal: ident { field: val, ... }
        if matches!(self.peek(), Token::LBrace) && first_char_is_upper(&name) {
            return self.parse_struct_literal(name, lhs.span().merge(name_span));
        }

        // Plain field access
        let span = lhs.span().merge(name_span);
        Ok(Expression::MemberAccess {
            object: Box::new(lhs),
            field: name,
            span,
        })
    }

    // ── Struct literal ────────────────────────────────────────

    fn parse_struct_literal(
        &mut self,
        name: String,
        start_span: Span,
    ) -> Result<Expression, CompileError> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let (field_name, _) = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let value = self.parse_expression()?;
            fields.push((field_name, value));
            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        let end = self.expect(&Token::RBrace)?;
        Ok(Expression::StructLiteral {
            name,
            type_args: Vec::new(),
            fields,
            span: start_span.merge(end),
        })
    }
}
