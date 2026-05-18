use super::*;

impl Parser {
    // ── Struct ────────────────────────────────────────────────

    pub(super) fn parse_struct_def(
        &mut self,
        name: String,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.parse_struct_def_with_params(name, Vec::new(), public, name_span)
    }

    pub(super) fn parse_struct_def_with_params(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.expect(&Token::Colon)?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }

            let field_start = self.peek_span();

            // optional `mut` prefix
            let mutable = self.consume_mutability_keyword();

            let (field_name, _) = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;

            // optional default
            let default = if matches!(self.peek(), Token::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            let field_span = field_start.merge(self.prev_span());
            fields.push(StructField {
                name: field_name,
                ty,
                default,
                mutable,
                span: field_span,
            });

            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Declaration::Struct {
            name,
            type_params,
            fields,
            public,
            span: name_span.merge(end),
        })
    }

    // ── Enum ──────────────────────────────────────────────────

    pub(super) fn parse_enum_def(
        &mut self,
        name: String,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.parse_enum_def_with_params(name, Vec::new(), public, name_span)
    }

    pub(super) fn parse_enum_def_with_params(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.expect(&Token::Colon)?;
        self.skip_newlines();

        let mut variants = Vec::new();
        loop {
            self.skip_newlines();
            // Check for end of enum (next declaration or EOF)
            match self.peek() {
                Token::EOF => break,
                // If we see an identifier that is NOT followed by a comma, newline, (, or EOF
                // at same indentation level, it's a new declaration
                Token::Identifier(_) => {}
                _ => break,
            }

            let (var_name, var_span) = self.expect_identifier()?;

            // Optional payload: `Variant(Type)`
            let payload = if matches!(self.peek(), Token::LParen) {
                self.advance();
                let ty = self.parse_type()?;
                self.expect(&Token::RParen)?;
                Some(ty)
            } else {
                None
            };

            variants.push(EnumVariant {
                name: var_name,
                payload,
                span: var_span.merge(self.prev_span()),
            });

            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let end_span = if let Some(v) = variants.last() {
            v.span
        } else {
            name_span
        };

        Ok(Declaration::Enum {
            name,
            type_params,
            variants,
            public,
            span: name_span.merge(end_span),
        })
    }

    // ── Type Params ───────────────────────────────────────────

    pub(super) fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, CompileError> {
        self.expect(&Token::Lt)?;
        let mut params = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Gt) {
                break;
            }
            let (name, span) = self.expect_identifier()?;
            let constraint = if matches!(self.peek(), Token::Colon) {
                self.advance();
                let (c, _) = self.expect_identifier()?;
                Some(c)
            } else {
                None
            };
            let constraint_type_args = if constraint.is_some() && matches!(self.peek(), Token::Lt) {
                self.parse_type_arg_list()?
            } else {
                Vec::new()
            };
            params.push(TypeParam {
                name,
                constraint,
                constraint_type_args,
                span,
            });
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::Gt)?;
        Ok(params)
    }
}
