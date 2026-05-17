use super::*;

impl Parser {
    // ── Declarations ──────────────────────────────────────────

    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, CompileError> {
        self.skip_newlines();

        // pub prefix
        let public = if matches!(self.peek(), Token::Pub) {
            self.advance();
            self.skip_newlines();
            true
        } else {
            false
        };

        // Import: `{ names } = module.path`
        if matches!(self.peek(), Token::LBrace) && self.is_import() {
            return self.parse_import();
        }

        // Must be identifier-led
        let (name, name_span) = self.expect_identifier()?;

        self.skip_newlines();

        match self.peek() {
            // Behavior: `Name: behavior { method: (Self) Return }`
            Token::Colon if self.colon_is_followed_by_identifier("behavior") => {
                self.parse_behavior_def(name, Vec::new(), public, name_span)
            }

            // Struct: `Name: { fields }`
            Token::Colon if self.is_struct_def() => self.parse_struct_def(name, public, name_span),

            // Enum: `Name: Variant1, Variant2` OR `Name:\n  Variant1,\n  Variant2`
            Token::Colon if self.is_enum_def() => self.parse_enum_def(name, public, name_span),

            // Generic type/function: `Name<T>...`
            Token::Lt => {
                let type_params = self.parse_type_params()?;
                self.skip_newlines();
                match self.peek() {
                    Token::Assign => {
                        // Generic function: `name<T> = (params) ret { body }`
                        self.parse_function_def(name, type_params, public, name_span)
                    }
                    Token::Dot => {
                        // Generic receiver method: `Type<T>.method = (self: Type<T>) ...`
                        self.advance(); // consume .
                        let (method_name, _method_span) = self.expect_identifier()?;
                        self.skip_newlines();

                        if let Ok(keyword) = method_name.parse::<TypeDeclarationKeyword>() {
                            if matches!(keyword, TypeDeclarationKeyword::Impl) {
                                return Err(CompileError::Syntax(
                                    "generic impl blocks are not implemented".to_string(),
                                    Some(self.peek_span()),
                                ));
                            }
                            return Err(CompileError::Syntax(
                                format!(
                                    "gated v1 feature '{keyword}': type association and behavior constraints are specified in docs/V1_SPEC.md but are not implemented"
                                ),
                                Some(self.peek_span()),
                            ));
                        }

                        let mut all_type_params = type_params;
                        if matches!(self.peek(), Token::Lt) {
                            all_type_params.extend(self.parse_type_params()?);
                        }

                        self.skip_newlines();
                        self.expect(&Token::Assign)?;
                        self.skip_newlines();

                        let (params, return_type, body) =
                            self.parse_function_signature_and_body()?;
                        let span = name_span.merge(body.span());
                        Ok(Declaration::Method {
                            type_name: name,
                            method_name,
                            type_params: all_type_params,
                            params,
                            return_type,
                            body,
                            public,
                            span,
                        })
                    }
                    Token::Colon if self.is_struct_def() => {
                        self.parse_struct_def_with_params(name, type_params, public, name_span)
                    }
                    Token::Colon if self.colon_is_followed_by_identifier("behavior") => {
                        self.parse_behavior_def(name, type_params, public, name_span)
                    }
                    Token::Colon => {
                        self.parse_enum_def_with_params(name, type_params, public, name_span)
                    }
                    _ => Err(CompileError::Syntax(
                        "expected '=' or ':' after generic type parameters".to_string(),
                        Some(self.peek_span()),
                    )),
                }
            }

            // Method: `Type.method = ...`
            Token::Dot => {
                self.advance(); // consume .
                let (method_name, _method_span) = self.expect_identifier()?;
                self.skip_newlines();

                if let Ok(keyword) = method_name.parse::<TypeDeclarationKeyword>() {
                    return match keyword {
                        TypeDeclarationKeyword::Impl => self.parse_impl_block(name, name_span),
                        TypeDeclarationKeyword::Implements => {
                            self.parse_behavior_impl_block(name, name_span)
                        }
                        TypeDeclarationKeyword::Requires => {
                            self.parse_behavior_requires(name, name_span)
                        }
                        TypeDeclarationKeyword::Extends => {
                            self.parse_behavior_extends(name, name_span)
                        }
                    };
                }

                // Type.method<T> = (params) ret { body }
                let type_params = if matches!(self.peek(), Token::Lt) {
                    self.parse_type_params()?
                } else {
                    Vec::new()
                };

                self.skip_newlines();
                self.expect(&Token::Assign)?;
                self.skip_newlines();

                let (params, return_type, body) = self.parse_function_signature_and_body()?;
                let span = name_span.merge(body.span());
                Ok(Declaration::Method {
                    type_name: name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    public,
                    span,
                })
            }

            // Function: `name = (params) ret { body }` or const `name = expr`
            Token::Assign => self.parse_function_def(name, Vec::new(), public, name_span),

            // Const assignment at top level: `name := expr`
            Token::ConstAssign => self.parse_const_decl(name, name_span),

            // Mutable decl assignment at top level: `name ::= expr`
            Token::DeclareAssign => self.parse_var_decl_toplevel(name, name_span),

            _ => Err(CompileError::Syntax(
                format!(
                    "unexpected token {:?} after identifier '{}'",
                    self.peek(),
                    name
                ),
                Some(self.peek_span()),
            )),
        }
    }

    // ── Function ──────────────────────────────────────────────

    fn parse_function_def(
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

    pub(super) fn parse_function_signature_and_body(
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

    pub(super) fn parse_param_list(&mut self) -> Result<Vec<Param>, CompileError> {
        let mut params = Vec::new();
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen) {
            return Ok(params);
        }
        loop {
            self.skip_newlines();
            let param_start = self.peek_span();

            // Optional `mut` qualifier
            let mutable = if matches!(self.peek(), Token::Identifier(ref s) if s == "mut") {
                self.advance();
                true
            } else {
                false
            };

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

    // ── Top-level const/var decl ──────────────────────────────

    fn parse_const_decl(
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

    fn parse_var_decl_toplevel(
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
