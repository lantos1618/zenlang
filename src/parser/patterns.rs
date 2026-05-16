use super::*;

impl Parser {
    // ── Patterns ──────────────────────────────────────────────

    pub(super) fn parse_pattern(&mut self) -> Result<Pattern, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            Token::Identifier(ref name) if name == "true" => {
                let (_, span) = self.advance();
                Ok(Pattern::BoolTrue { span })
            }
            Token::Identifier(ref name) if name == "false" => {
                let (_, span) = self.advance();
                Ok(Pattern::BoolFalse { span })
            }
            Token::Identifier(ref name) if name == "_" => {
                let (_, span) = self.advance();
                Ok(Pattern::Wildcard { span })
            }
            Token::Dot => self.parse_shorthand_enum_pattern(),
            Token::Identifier(ref name) if first_char_is_upper(name) => {
                // Enum variant pattern: VariantName or VariantName(binding)
                let name = name.clone();
                let (_, span) = self.advance();

                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(&Token::RParen)?;
                    // This is a variant pattern — enum name will be inferred by typechecker
                    Ok(Pattern::Enum {
                        enum_name: String::new(), // inferred
                        variant: name,
                        payload: Some(Box::new(inner)),
                        span: span.merge(self.prev_span()),
                    })
                } else if matches!(self.peek(), Token::LBrace) && self.is_struct_pattern() {
                    // Struct destructuring pattern: Name { field1, field2 }
                    self.advance();
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newlines();
                        if matches!(self.peek(), Token::RBrace) {
                            break;
                        }
                        let (field, _) = self.expect_identifier()?;
                        fields.push((field, None));
                        self.skip_newlines();
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(&Token::RBrace)?;
                    Ok(Pattern::Struct {
                        name,
                        fields,
                        span: span.merge(self.prev_span()),
                    })
                } else {
                    // Simple enum variant (no payload)
                    Ok(Pattern::Enum {
                        enum_name: String::new(),
                        variant: name,
                        payload: None,
                        span,
                    })
                }
            }
            Token::Identifier(_) => {
                let (tok, span) = self.advance();
                if let Token::Identifier(name) = tok {
                    Ok(Pattern::Identifier { name, span })
                } else {
                    unreachable!()
                }
            }
            Token::IntLiteral(_) | Token::FloatLiteral(_) | Token::StringLiteral(_) => {
                let expr = self.parse_expression()?;
                let span = expr.span();
                Ok(Pattern::Literal { value: expr, span })
            }
            _ => Err(CompileError::Syntax(
                format!("expected pattern, found {:?}", self.peek()),
                Some(self.peek_span()),
            )),
        }
    }

    fn parse_shorthand_enum_pattern(&mut self) -> Result<Pattern, CompileError> {
        let (_, dot_span) = self.advance();
        let (variant, variant_span) = self.expect_identifier()?;
        let mut span = dot_span.merge(variant_span);

        let payload = if matches!(self.peek(), Token::LParen) {
            self.advance();
            let inner = self.parse_pattern()?;
            let end = self.expect(&Token::RParen)?;
            span = span.merge(end);
            Some(Box::new(inner))
        } else {
            None
        };

        Ok(Pattern::Enum {
            enum_name: String::new(),
            variant,
            payload,
            span,
        })
    }
}
