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

                        if matches!(method_name.as_str(), "implements" | "requires" | "extends") {
                            return Err(CompileError::Syntax(
                                format!(
                                    "gated v1 feature '{method_name}': type association and behavior constraints are specified in docs/V1_SPEC.md but are not implemented"
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

                if method_name == "implements" {
                    return self.parse_behavior_impl_block(name, name_span);
                }

                if method_name == "requires" {
                    return self.parse_behavior_requires(name, name_span);
                }

                if method_name == "extends" {
                    return self.parse_behavior_extends(name, name_span);
                }

                // Type.impl = { methods }
                if method_name == "impl" {
                    return self.parse_impl_block(name, name_span);
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

    // ── Import ────────────────────────────────────────────────

    fn parse_import(&mut self) -> Result<Declaration, CompileError> {
        let start = self.peek_span();

        // { name1, name2 }
        self.expect(&Token::LBrace)?;
        let mut names = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let (name, _) = self.expect_identifier()?;
            names.push(name);
            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();

        // module path: std, std.io, @std.io, @builtin
        let module_path = self.parse_module_path()?;
        let span = start.merge(self.prev_span());

        Ok(Declaration::Import {
            names,
            module_path,
            span,
        })
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>, CompileError> {
        let mut path = Vec::new();

        match self.peek().clone() {
            Token::AtStd => {
                self.advance();
                path.push("@std".to_string());
            }
            Token::AtBuiltin => {
                self.advance();
                path.push("@builtin".to_string());
            }
            Token::Identifier(name) => {
                self.advance();
                path.push(name);
            }
            _ => {
                return Err(CompileError::Syntax(
                    format!("expected module path, found {:?}", self.peek()),
                    Some(self.peek_span()),
                ));
            }
        }

        while matches!(self.peek(), Token::Dot) {
            self.advance();
            let (seg, _) = self.expect_identifier()?;
            path.push(seg);
        }

        Ok(path)
    }

    // ── Struct ────────────────────────────────────────────────

    fn parse_struct_def(
        &mut self,
        name: String,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.parse_struct_def_with_params(name, Vec::new(), public, name_span)
    }

    fn parse_struct_def_with_params(
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
            let mutable = if matches!(self.peek(), Token::Identifier(ref s) if s == "mut") {
                self.advance();
                true
            } else {
                false
            };

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

    fn parse_enum_def(
        &mut self,
        name: String,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.parse_enum_def_with_params(name, Vec::new(), public, name_span)
    }

    fn parse_enum_def_with_params(
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

    // ── Type Params ───────────────────────────────────────────

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, CompileError> {
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
