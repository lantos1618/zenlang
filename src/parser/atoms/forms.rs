use super::*;

impl Parser {
    pub(super) fn parse_shorthand_enum_variant_expr(&mut self) -> Result<Expression, CompileError> {
        let (_, dot_span) = self.advance();
        let (variant, _) = self.expect_identifier()?;
        let payload = self.parse_optional_enum_variant_payload()?;
        let span = dot_span.merge(self.prev_span());

        Ok(Expression::EnumVariant {
            enum_name: String::new(),
            type_args: Vec::new(),
            variant,
            payload,
            span,
        })
    }

    pub(in crate::parser) fn parse_match_or_while(
        &mut self,
        scrutinee: Expression,
    ) -> Result<Expression, CompileError> {
        self.advance();
        self.skip_newlines();

        if matches!(self.peek(), Token::LBrace) {
            let body = self.parse_block_expression()?;
            let span = scrutinee.span().merge(body.span());
            return Ok(Expression::If {
                condition: Box::new(scrutinee),
                then_body: Box::new(body),
                else_body: None,
                span,
            });
        }

        let mut arms = Vec::new();
        while matches!(self.peek(), Token::Pipe) {
            self.advance();
            self.skip_newlines();

            let arm_start = self.peek_span();
            let pattern = self.parse_pattern()?;

            self.skip_newlines();
            let body = self.parse_block_expression()?;
            let arm_span = arm_start.merge(body.span());

            arms.push(MatchArm {
                pattern,
                guard: None,
                body,
                span: arm_span,
            });
            self.skip_newlines();
        }

        let span = scrutinee
            .span()
            .merge(arms.last().map(|a| a.span).unwrap_or(scrutinee.span()));

        Ok(Expression::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span,
        })
    }

    pub(super) fn parse_loop(&mut self, start_span: Span) -> Result<Expression, CompileError> {
        self.skip_newlines();
        if matches!(self.peek(), Token::LParen) {
            self.advance();
            self.skip_newlines();
            if matches!(self.peek(), Token::LParen) {
                self.advance();
                self.skip_newlines();
                let control = if matches!(self.peek(), Token::RParen) {
                    None
                } else {
                    let (name, _) = self.expect_identifier()?;
                    Some((name, self.fresh_loop_control_label()))
                };
                self.skip_newlines();
                self.expect(&Token::RParen)?;
                self.skip_newlines();
                if let Some((name, label)) = &control {
                    self.loop_controls.push((name.clone(), label.clone()));
                }
                let body_result = self.parse_block_expression();
                if control.is_some() {
                    self.loop_controls.pop();
                }
                let body = body_result?;
                let end = self.expect(&Token::RParen)?;
                return Ok(Expression::Loop {
                    body: Box::new(body),
                    control_label: control.map(|(_, label)| label),
                    span: start_span.merge(end),
                });
            }

            let expr = self.parse_expression()?;
            let end = self.expect(&Token::RParen)?;
            return Ok(Expression::Loop {
                body: Box::new(expr),
                control_label: None,
                span: start_span.merge(end),
            });
        }

        let body = self.parse_block_expression()?;
        let span = start_span.merge(body.span());
        Ok(Expression::Loop {
            body: Box::new(body),
            control_label: None,
            span,
        })
    }

    pub(super) fn parse_cast(&mut self, start_span: Span) -> Result<Expression, CompileError> {
        self.expect(&Token::LParen)?;
        let expr = self.parse_expression()?;
        self.expect(&Token::Comma)?;
        let target_type = self.parse_type()?;
        let end = self.expect(&Token::RParen)?;
        Ok(Expression::Cast {
            expr: Box::new(expr),
            target_type,
            span: start_span.merge(end),
        })
    }

    pub(super) fn parse_string_interpolation(&mut self) -> Result<Expression, CompileError> {
        let start = self.peek_span();
        let mut parts = Vec::new();

        loop {
            match self.peek().clone() {
                Token::StringChunk(s) => {
                    self.advance();
                    parts.push(StringPart::Literal(s));
                }
                Token::InterpolationStart => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    parts.push(StringPart::Expr(expr));
                    match self.peek() {
                        Token::InterpolationEnd => {
                            self.advance();
                        }
                        _ => {
                            return Err(CompileError::Syntax(
                                "expected closing } for string interpolation".into(),
                                Some(self.peek_span()),
                            ));
                        }
                    }
                }
                _ => break,
            }
        }

        let span = start.merge(self.prev_span());
        Ok(Expression::StringInterpolation { parts, span })
    }
}
