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

        // Check if this is a function (starts with `(`) or a top-level const expression
        if !matches!(self.peek(), Token::LParen) {
            // Top-level expression/const: `name = expr`
            let value = self.parse_expression()?;
            let span = name_span.merge(value.span());
            return Ok(Declaration::TopLevelExpr {
                expr: Expression::BinaryOp {
                    op: BinaryOp::Eq,
                    left: Box::new(Expression::Identifier {
                        name,
                        span: name_span,
                    }),
                    right: Box::new(value),
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
            span,
        })
    }

    pub(in crate::parser) fn parse_function_signature_and_body(
        &mut self,
    ) -> Result<(Vec<Param>, Option<AstType>, Expression), CompileError> {
        // (params)
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        self.skip_newlines();

        // optional return type
        let return_type = if !matches!(self.peek(), Token::LBrace) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_newlines();

        // body block
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

            // Optional `mut` qualifier
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
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    pub(in crate::parser) fn parse_const_decl(
        &mut self,
        name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.advance(); // consume :=
        self.skip_newlines();
        let value = self.parse_expression()?;
        let span = name_span.merge(value.span());
        Ok(Declaration::TopLevelExpr {
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
        })
    }

    pub(in crate::parser) fn parse_var_decl_toplevel(
        &mut self,
        name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.advance(); // consume ::=
        self.skip_newlines();
        let value = self.parse_expression()?;
        let span = name_span.merge(value.span());
        Ok(Declaration::TopLevelExpr {
            expr: Expression::Block {
                statements: vec![Statement::VarDecl {
                    name,
                    ty: None,
                    value,
                    mutable: true,
                    constant: false,
                    span,
                }],
                expr: None,
                span,
            },
            span,
        })
    }
}
