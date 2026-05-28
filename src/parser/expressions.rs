use super::*;

mod infix;
mod suffixes;

use infix::InfixParse;

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, CompileError> {
        self.parse_expr_bp(0)
    }

    pub(super) fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expression, CompileError> {
        self.skip_newlines();
        let mut lhs = self.parse_prefix()?;

        loop {
            self.skip_newlines_if_continuation();

            match self.peek() {
                Token::Dot => {
                    if POSTFIX_BP < min_bp {
                        break;
                    }
                    lhs = self.parse_dot_suffix(lhs)?;
                    continue;
                }

                Token::LBracket => {
                    if POSTFIX_BP < min_bp {
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

                Token::LBrace => {
                    if let Some((name, id_span)) = expression_identifier(&lhs) {
                        if first_char_is_upper(name) {
                            if POSTFIX_BP < min_bp {
                                break;
                            }
                            let name = name.to_string();
                            lhs = self.parse_struct_literal(name, Vec::new(), id_span)?;
                            continue;
                        }
                    }
                }

                Token::LParen => {
                    if let Some((name, id_span)) = expression_identifier(&lhs) {
                        if POSTFIX_BP < min_bp {
                            break;
                        }
                        let name = name.to_string();
                        lhs = self.parse_function_call_tail(name, Vec::new(), id_span)?;
                        continue;
                    }
                }

                Token::Lt => {
                    if let Some((name, id_span)) = expression_identifier(&lhs) {
                        if POSTFIX_BP < min_bp {
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
                                        let name = name.to_string();
                                        lhs = self
                                            .parse_function_call_tail(name, type_args, id_span)?;
                                        continue;
                                    }
                                    if matches!(self.peek(), Token::Dot)
                                        && self.peek_span().start == type_args_end.end
                                        && first_char_is_upper(name)
                                    {
                                        let enum_name = name.to_string();
                                        lhs = self.parse_generic_enum_variant(
                                            enum_name, type_args, id_span,
                                        )?;
                                        continue;
                                    }
                                    if matches!(self.peek(), Token::LBrace)
                                        && first_char_is_upper(name)
                                    {
                                        let name = name.to_string();
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

fn expression_identifier(expr: &Expression) -> Option<(&str, Span)> {
    match expr {
        Expression::Identifier { name, span } => Some((name, *span)),
        _ => None,
    }
}
