use super::*;

mod suffixes;

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
                            lhs = self.parse_struct_literal(name, Vec::new(), id_span)?;
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
                        if let Some(loop_control) =
                            self.parse_loop_control_function_call(&name, &args, span)
                        {
                            lhs = loop_control;
                            continue;
                        }
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
                            if matches!(self.peek(), Token::Dot) && first_char_is_upper(name) {
                                let enum_name = name.clone();
                                lhs =
                                    self.parse_generic_enum_variant(enum_name, type_args, id_span)?;
                                continue;
                            }
                            if matches!(self.peek(), Token::LBrace) && first_char_is_upper(name) {
                                let name = name.clone();
                                lhs = self.parse_struct_literal(name, type_args, id_span)?;
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
}
