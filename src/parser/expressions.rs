use super::*;

mod infix;
mod suffixes;

use infix::InfixParse;

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

                        if self.peek_span().start == id_span.end {
                            let type_args_start = self.pos;
                            let checkpoint = self.checkpoint();
                            match self.parse_type_arg_list() {
                                Ok(type_args) => {
                                    let type_args_end = self.prev_span();
                                    if matches!(self.peek(), Token::LParen)
                                        && self.peek_span().start == type_args_end.end
                                    {
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
                                    if matches!(self.peek(), Token::Dot)
                                        && self.peek_span().start == type_args_end.end
                                        && first_char_is_upper(name)
                                    {
                                        let enum_name = name.clone();
                                        lhs = self.parse_generic_enum_variant(
                                            enum_name, type_args, id_span,
                                        )?;
                                        continue;
                                    }
                                    if matches!(self.peek(), Token::LBrace)
                                        && first_char_is_upper(name)
                                    {
                                        let name = name.clone();
                                        lhs =
                                            self.parse_struct_literal(name, type_args, id_span)?;
                                        continue;
                                    }
                                }
                                Err(err) => {
                                    if self.generic_close_has_attached_suffix_from(type_args_start)
                                    {
                                        return Err(err);
                                    }
                                }
                            }
                            self.restore(checkpoint);
                        }
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

            match self.parse_infix_or_range_expr(lhs, min_bp)? {
                InfixParse::Parsed(next) => {
                    lhs = next;
                    continue;
                }
                InfixParse::Stop(current) => {
                    lhs = current;
                    break;
                }
                InfixParse::Continue(current) => lhs = current,
            }

            break;
        }

        Ok(lhs)
    }
}
