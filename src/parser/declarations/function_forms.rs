use super::*;

impl Parser {
    pub(in crate::parser) fn parse_function_def(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.expect(&Token::Assign)?;
        self.skip_newlines();

        // A function literal starts with `(`. Anything else is a module-level
        // constant binding: `PI = 3.14159` — lowered to a single-statement
        // `VarDecl` block, the same shape as the `:=` const form.
        if !matches!(self.peek(), Token::LParen) {
            let value = self.parse_expression()?;
            let span = name_span.merge(value.span());
            return Ok(Declaration::TopLevelExpr {
                expr: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name,
                        ty: None,
                        value,
                        mutable: false,
                        constant: true,
                        span,
                    }],
                    expr: None,
                    span,
                },
                span,
            });
        }

        let (params, return_type, body) = self.parse_function_signature_and_body()?;
        let span = name_span.merge(body.span());

        Ok(Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            public,
            external: false,
            span,
        })
    }

    /// Parse an `extern` C function declaration: `extern NAME = (params) Ret`
    /// — a bodyless callable bound to a `link:`-ed library symbol. Stored as a
    /// `Function` with `external: true` and an empty-block placeholder body.
    pub(in crate::parser) fn parse_extern_function(
        &mut self,
        public: bool,
        start_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (name, _) = self.expect_identifier()?;
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        // The return type (if any) sits on the same line; a line break or EOF
        // means a `void` extern.
        let return_type = match self.peek() {
            Token::Newline | Token::EOF => None,
            _ => Some(self.parse_type()?),
        };
        let span = start_span.merge(self.prev_span());
        Ok(Declaration::Function {
            name,
            type_params: Vec::new(),
            params,
            return_type,
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span,
            },
            public,
            external: true,
            span,
        })
    }

    pub(in crate::parser) fn parse_function_signature_and_body(
        &mut self,
    ) -> Result<(Vec<Param>, Option<AstType>, Expression), CompileError> {
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        self.skip_newlines();

        let return_type = self.parse_optional_return_type_before_block()?;
        self.skip_newlines();

        let body = self.parse_block_expression()?;
        Ok((params, return_type, body))
    }

    pub(in crate::parser) fn parse_param_list(&mut self) -> Result<Vec<Param>, CompileError> {
        let mut params = Vec::new();
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen) {
            return Ok(params);
        }
        loop {
            self.skip_newlines();
            let param_start = self.peek_span();

            let mutable = self.consume_mutability_keyword();

            let (name, _) = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            let span = param_start.merge(self.prev_span());
            params.push(Param {
                name,
                ty,
                mutable,
                span,
            });
            self.skip_newlines();
            if !self.consume_comma() {
                break;
            }
        }
        Ok(params)
    }

    pub(in crate::parser) fn parse_method_declaration(
        &mut self,
        type_name: String,
        method_name: String,
        mut type_params: Vec<TypeParam>,
        public: bool,
        start_span: Span,
    ) -> Result<Declaration, CompileError> {
        if matches!(self.peek(), Token::Lt) {
            type_params.extend(self.parse_type_params()?);
        }
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();
        let (params, return_type, body) = self.parse_function_signature_and_body()?;
        let span = start_span.merge(body.span());
        Ok(Declaration::Method {
            type_name,
            method_name,
            type_params,
            params,
            return_type,
            body,
            public,
            span,
        })
    }

    pub(in crate::parser) fn parse_top_level_var_decl(
        &mut self,
        name: String,
        name_span: Span,
        mutable: bool,
        constant: bool,
    ) -> Result<Declaration, CompileError> {
        self.advance();
        self.skip_newlines();
        let value = self.parse_expression()?;
        let span = name_span.merge(value.span());
        Ok(Declaration::TopLevelExpr {
            expr: Expression::Block {
                statements: vec![Statement::VarDecl {
                    name,
                    ty: None,
                    value,
                    mutable,
                    constant,
                    span,
                }],
                expr: None,
                span,
            },
            span,
        })
    }
}
