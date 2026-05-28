use super::*;

impl Parser {
    pub(super) fn parse_pattern(&mut self) -> Result<Pattern, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            Token::Identifier(ref name) => {
                if matches!(name.as_str(), "true" | "false" | "_") {
                    let (_, span) = self.advance();
                    match name.as_str() {
                        "true" => Ok(Pattern::BoolTrue { span }),
                        "false" => Ok(Pattern::BoolFalse { span }),
                        _ => Ok(Pattern::Wildcard { span }),
                    }
                } else if first_char_is_upper(name) {
                    let name = name.clone();
                    let (_, span) = self.advance();

                    if matches!(self.peek(), Token::LBrace) && self.is_struct_pattern() {
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
                            self.consume_comma();
                        }
                        self.expect(&Token::RBrace)?;
                        Ok(Pattern::Struct {
                            name,
                            fields,
                            span: span.merge(self.prev_span()),
                        })
                    } else {
                        let payload = self.parse_optional_enum_pattern_payload()?;
                        Ok(Pattern::Enum {
                            enum_name: String::new(),
                            variant: name,
                            payload,
                            span: span.merge(self.prev_span()),
                        })
                    }
                } else {
                    let name = name.clone();
                    let (_, span) = self.advance();
                    Ok(Pattern::Identifier { name, span })
                }
            }
            Token::Dot => self.parse_shorthand_enum_pattern(),
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
        let (variant, _) = self.expect_identifier()?;
        let payload = self.parse_optional_enum_pattern_payload()?;
        let span = dot_span.merge(self.prev_span());

        Ok(Pattern::Enum {
            enum_name: String::new(),
            variant,
            payload,
            span,
        })
    }

    fn parse_optional_enum_pattern_payload(
        &mut self,
    ) -> Result<Option<Box<Pattern>>, CompileError> {
        if !matches!(self.peek(), Token::LParen) {
            return Ok(None);
        }

        self.advance();
        let inner = self.parse_pattern()?;
        self.expect(&Token::RParen)?;
        Ok(Some(Box::new(inner)))
    }
}
