use super::*;

impl Parser {
    pub(super) fn parse_dot_suffix(&mut self, lhs: Expression) -> Result<Expression, CompileError> {
        self.advance();

        let (name, name_span) = self.expect_identifier()?;

        if let Expression::Identifier {
            name: ref enum_name,
            span: id_span,
        } = lhs
        {
            if first_char_is_upper(enum_name) && first_char_is_upper(&name) {
                let payload = self.parse_optional_enum_variant_payload()?;
                let span = id_span.merge(self.prev_span());
                return Ok(Expression::EnumVariant {
                    enum_name: enum_name.clone(),
                    type_args: Vec::new(),
                    variant: name,
                    payload,
                    span,
                });
            }
        }

        let type_args =
            if matches!(self.peek(), Token::Lt) && self.peek_span().start == name_span.end {
                let type_args_start = self.pos;
                let checkpoint = self.checkpoint();
                match self.parse_type_arg_list() {
                    Ok(type_args)
                        if matches!(self.peek(), Token::LParen)
                            && self.peek_span().start == self.prev_span().end =>
                    {
                        type_args
                    }
                    Err(err) if self.generic_close_has_attached_suffix_from(type_args_start) => {
                        return Err(err);
                    }
                    _ => {
                        self.restore(checkpoint);
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

        if matches!(self.peek(), Token::LParen) {
            self.advance();
            let args = self.parse_arg_list()?;
            let end = self.expect(&Token::RParen)?;
            let span = lhs.span().merge(end);

            if let Some(loop_control) =
                self.parse_loop_control_method_call(&lhs, &name, &args, span)
            {
                return Ok(loop_control);
            }

            return Ok(Expression::MethodCall {
                receiver: Box::new(lhs),
                method: name,
                type_args,
                args,
                span,
            });
        }

        if matches!(self.peek(), Token::LBrace) && first_char_is_upper(&name) {
            return self.parse_struct_literal(name, Vec::new(), lhs.span().merge(name_span));
        }

        let span = lhs.span().merge(name_span);
        Ok(Expression::MemberAccess {
            object: Box::new(lhs),
            field: name,
            span,
        })
    }

    pub(super) fn parse_loop_control_function_call(
        &self,
        name: &str,
        args: &[Expression],
        span: Span,
    ) -> Option<Expression> {
        (args.len() == 1).then_some(())?;
        self.loop_control_invocation(name, &args[0], span)
    }

    pub(super) fn parse_function_call_tail(
        &mut self,
        name: String,
        type_args: Vec<AstType>,
        start_span: Span,
    ) -> Result<Expression, CompileError> {
        self.advance();
        let args = self.parse_arg_list()?;
        let end = self.expect(&Token::RParen)?;
        let span = start_span.merge(end);
        if type_args.is_empty() {
            if let Some(loop_control) = self.parse_loop_control_function_call(&name, &args, span) {
                return Ok(loop_control);
            }
        }
        Ok(Expression::FunctionCall {
            name,
            module: None,
            type_args,
            args,
            span,
        })
    }

    fn parse_loop_control_method_call(
        &self,
        receiver: &Expression,
        name: &str,
        args: &[Expression],
        span: Span,
    ) -> Option<Expression> {
        args.is_empty().then_some(())?;
        self.loop_control_invocation(name, receiver, span)
    }

    fn loop_control_invocation(
        &self,
        action_name: &str,
        control: &Expression,
        span: Span,
    ) -> Option<Expression> {
        let action = action_name.parse::<LoopControlAction>().ok()?;
        let Expression::Identifier {
            name: control_name, ..
        } = control
        else {
            return None;
        };
        self.loop_control_label(control_name)
            .map(|target_label| Expression::LoopControl {
                action,
                target_label,
                span,
            })
    }

    pub(super) fn parse_struct_literal(
        &mut self,
        name: String,
        type_args: Vec<AstType>,
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
            self.consume_comma();
        }
        let end = self.expect(&Token::RBrace)?;
        Ok(Expression::StructLiteral {
            name,
            type_args,
            fields,
            span: start_span.merge(end),
        })
    }

    pub(super) fn parse_generic_enum_variant(
        &mut self,
        enum_name: String,
        type_args: Vec<AstType>,
        start_span: Span,
    ) -> Result<Expression, CompileError> {
        self.expect(&Token::Dot)?;
        let (variant, _) = self.expect_identifier()?;
        let payload = self.parse_optional_enum_variant_payload()?;
        let span = start_span.merge(self.prev_span());
        Ok(Expression::EnumVariant {
            enum_name,
            type_args,
            variant,
            payload,
            span,
        })
    }

    pub(in crate::parser) fn parse_optional_enum_variant_payload(
        &mut self,
    ) -> Result<Option<Box<Expression>>, CompileError> {
        if !matches!(self.peek(), Token::LParen) {
            return Ok(None);
        }

        self.advance();
        let expr = self.parse_expression()?;
        self.expect(&Token::RParen)?;
        Ok(Some(Box::new(expr)))
    }
}
