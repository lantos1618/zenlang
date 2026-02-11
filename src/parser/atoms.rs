use super::*;

impl Parser {
    /// Parse prefix / atom expressions.
    pub(super) fn parse_prefix(&mut self) -> Result<Expression, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            // Unary operators
            Token::Minus => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Not => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Tilde => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span: s,
                })
            }

            // Literals
            Token::IntLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::IntLiteral(v) = tok {
                    Ok(Expression::IntLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::FloatLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::FloatLiteral(v) = tok {
                    Ok(Expression::FloatLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::StringLiteral(v) = tok {
                    Ok(Expression::StringLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }

            // String interpolation
            Token::StringChunk(_) | Token::InterpolationStart => self.parse_string_interpolation(),

            // Identifier (or keyword-like: true, false, return, break, continue, loop, cast)
            Token::Identifier(ref name) => match name.as_str() {
                "true" => {
                    let (_, span) = self.advance();
                    Ok(Expression::BoolLiteral { value: true, span })
                }
                "false" => {
                    let (_, span) = self.advance();
                    Ok(Expression::BoolLiteral { value: false, span })
                }
                "return" => {
                    let (_, span) = self.advance();
                    self.skip_newlines();
                    let value = if self.is_expression_start() {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };
                    let end = value.as_ref().map(|v| v.span()).unwrap_or(span);
                    Ok(Expression::Return {
                        value,
                        span: span.merge(end),
                    })
                }
                "break" => {
                    let (_, span) = self.advance();
                    Ok(Expression::Break { span })
                }
                "continue" => {
                    let (_, span) = self.advance();
                    Ok(Expression::Continue { span })
                }
                "loop" => {
                    let (_, span) = self.advance();
                    self.parse_loop(span)
                }
                "cast" => {
                    let (_, span) = self.advance();
                    self.parse_cast(span)
                }
                _ => {
                    let name = name.clone();
                    let (_, span) = self.advance();
                    Ok(Expression::Identifier { name, span })
                }
            },

            // @this.defer(expr) or other @ tokens
            Token::AtThis => {
                let (_, span) = self.advance();
                // @this.defer(expr)
                if matches!(self.peek(), Token::Dot) {
                    self.advance(); // consume .
                    let (method, _) = self.expect_identifier()?;
                    if method == "defer" {
                        self.expect(&Token::LParen)?;
                        let expr = self.parse_expression()?;
                        let end = self.expect(&Token::RParen)?;
                        return Ok(Expression::Defer {
                            expr: Box::new(expr),
                            span: span.merge(end),
                        });
                    }
                    // Other @this.method calls
                    return Err(CompileError::Syntax(
                        format!("unknown @this method: {}", method),
                        Some(span),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "@this".to_string(),
                    span,
                })
            }

            // Module-qualified: @builtin.func(args), @std.mod.func(args)
            Token::AtBuiltin => {
                let (_, span) = self.advance();
                self.expect(&Token::Dot)?;
                let (func_name, _) = self.expect_identifier()?;

                // Check for type args
                let type_args = if matches!(self.peek(), Token::Lt) {
                    self.parse_type_arg_list()?
                } else {
                    Vec::new()
                };

                self.expect(&Token::LParen)?;
                let args = self.parse_arg_list()?;
                let end = self.expect(&Token::RParen)?;

                Ok(Expression::FunctionCall {
                    name: func_name,
                    module: Some("@builtin".to_string()),
                    type_args,
                    args,
                    span: span.merge(end),
                })
            }

            Token::AtStd => {
                let (_, span) = self.advance();
                self.expect(&Token::Dot)?;
                let mut module_parts = Vec::new();
                let (first, _) = self.expect_identifier()?;
                module_parts.push(first);

                // Collect module.sub.func path
                while matches!(self.peek(), Token::Dot) {
                    let saved = self.pos;
                    self.advance(); // consume .
                    if let Token::Identifier(_) = self.peek() {
                        let (part, _) = self.expect_identifier()?;
                        module_parts.push(part);
                    } else {
                        self.pos = saved;
                        break;
                    }
                }

                // Last part is the function name
                let func_name = module_parts.pop().unwrap();
                let module = if module_parts.is_empty() {
                    "@std".to_string()
                } else {
                    format!("@std.{}", module_parts.join("."))
                };

                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    let end = self.expect(&Token::RParen)?;
                    Ok(Expression::FunctionCall {
                        name: func_name,
                        module: Some(module),
                        type_args: Vec::new(),
                        args,
                        span: span.merge(end),
                    })
                } else {
                    // Might be a member access
                    Ok(Expression::MemberAccess {
                        object: Box::new(Expression::Identifier { name: module, span }),
                        field: func_name,
                        span: span.merge(self.prev_span()),
                    })
                }
            }

            // Parenthesized expression or closure
            Token::LParen => {
                // Try to detect if this is a closure: `(params) ret? { body }`
                // vs a grouped expression: `(expr)`
                if self.is_closure() {
                    self.parse_closure()
                } else {
                    self.advance(); // consume (
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen)?;
                    Ok(expr)
                }
            }

            // Block expression
            Token::LBrace => self.parse_block_expression(),

            // Array literal
            Token::LBracket => {
                let (_, span) = self.advance();
                let mut elements = Vec::new();
                loop {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBracket) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                let end = self.expect(&Token::RBracket)?;
                Ok(Expression::ArrayLiteral {
                    elements,
                    span: span.merge(end),
                })
            }

            _ => {
                let (tok, span) = self.advance();
                Err(CompileError::Syntax(
                    format!("unexpected token {:?} in expression", tok),
                    Some(span),
                ))
            }
        }
    }

    // ── Match / conditional / while ───────────────────────────

    pub(super) fn parse_match_or_while(
        &mut self,
        scrutinee: Expression,
    ) -> Result<Expression, CompileError> {
        self.advance(); // consume ?
        self.skip_newlines();

        // Shorthand conditional: `expr ? { body }` → if expr { body }
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

        // Match with arms: `expr ? | pattern { body } | pattern { body }`
        let mut arms = Vec::new();
        while matches!(self.peek(), Token::Pipe) {
            self.advance(); // consume |
            self.skip_newlines();

            let arm_start = self.peek_span();
            let pattern = self.parse_pattern()?;

            // Optional guard
            let guard = None; // TODO: guard expressions

            self.skip_newlines();
            let body = self.parse_block_expression()?;
            let arm_span = arm_start.merge(body.span());

            arms.push(MatchArm {
                pattern,
                guard,
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

    // ── Loop ──────────────────────────────────────────────────

    fn parse_loop(&mut self, start_span: Span) -> Result<Expression, CompileError> {
        self.skip_newlines();
        // loop(() { body }) — closure-style
        if matches!(self.peek(), Token::LParen) {
            self.advance(); // (
            self.skip_newlines();
            // The inner () is the empty param list of the closure
            if matches!(self.peek(), Token::LParen) {
                self.advance(); // inner (
                self.expect(&Token::RParen)?; // inner )
                self.skip_newlines();
                let body = self.parse_block_expression()?;
                let end = self.expect(&Token::RParen)?;
                return Ok(Expression::Loop {
                    body: Box::new(body),
                    span: start_span.merge(end),
                });
            }
            // loop(expr) — this shouldn't happen but handle gracefully
            let expr = self.parse_expression()?;
            let end = self.expect(&Token::RParen)?;
            return Ok(Expression::Loop {
                body: Box::new(expr),
                span: start_span.merge(end),
            });
        }

        // loop { body }
        let body = self.parse_block_expression()?;
        let span = start_span.merge(body.span());
        Ok(Expression::Loop {
            body: Box::new(body),
            span,
        })
    }

    // ── Cast ──────────────────────────────────────────────────

    fn parse_cast(&mut self, start_span: Span) -> Result<Expression, CompileError> {
        // cast(expr, Type)
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

    // ── String Interpolation ──────────────────────────────────

    fn parse_string_interpolation(&mut self) -> Result<Expression, CompileError> {
        let start = self.peek_span();
        let mut parts = Vec::new();

        loop {
            match self.peek() {
                Token::StringChunk(_) => {
                    let (tok, _) = self.advance();
                    if let Token::StringChunk(s) = tok {
                        parts.push(StringPart::Literal(s));
                    }
                }
                Token::InterpolationStart => {
                    self.advance(); // consume ${
                    let expr = self.parse_expression()?;
                    parts.push(StringPart::Expr(expr));
                    // expect InterpolationEnd
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

    // ── Block expression ──────────────────────────────────────

    pub(super) fn parse_block_expression(&mut self) -> Result<Expression, CompileError> {
        let start = self.expect(&Token::LBrace)?;
        let mut statements = Vec::new();
        let mut final_expr: Option<Box<Expression>> = None;

        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            if self.at_eof() {
                return Err(CompileError::Syntax(
                    "unterminated block".into(),
                    Some(start),
                ));
            }

            // Try to parse a statement
            let stmt_or_expr = self.parse_statement_or_expr()?;

            match stmt_or_expr {
                StmtOrExpr::Stmt(stmt) => {
                    statements.push(stmt);
                }
                StmtOrExpr::Expr(expr) => {
                    // If this is followed by } (possibly with newlines), it's the final expr
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBrace) {
                        final_expr = Some(Box::new(expr));
                    } else {
                        // It's an expression statement
                        let span = expr.span();
                        statements.push(Statement::Expression { expr, span });
                    }
                }
            }
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Expression::Block {
            statements,
            expr: final_expr,
            span: start.merge(end),
        })
    }

    // ── Closure detection and parsing ─────────────────────────

    fn is_closure(&self) -> bool {
        // Look for pattern: `(` ... `)` type? `{`
        // vs plain grouping: `(` expr `)`
        let mut i = self.pos + 1; // skip (
        let mut depth = 1u32;
        loop {
            match self.tokens.get(i).map(|(t, _)| t) {
                Some(Token::LParen) => {
                    depth += 1;
                    i += 1;
                }
                Some(Token::RParen) => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        // Skip newlines
                        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                            i += 1;
                        }
                        // If followed by `{` or a type then `{`, it's a closure
                        return matches!(
                            self.tokens.get(i).map(|(t, _)| t),
                            Some(Token::LBrace) | Some(Token::Identifier(_))
                        );
                    }
                    i += 1;
                }
                Some(Token::Colon) if depth == 1 => {
                    // Has a type annotation — likely a closure param
                    return true;
                }
                Some(Token::Comma) if depth == 1 => {
                    // Multiple params — likely a closure
                    return true;
                }
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    fn parse_closure(&mut self) -> Result<Expression, CompileError> {
        let start = self.peek_span();
        self.advance(); // consume (
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        self.skip_newlines();

        let return_type = if !matches!(self.peek(), Token::LBrace) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_newlines();
        let body = self.parse_block_expression()?;
        let span = start.merge(body.span());

        Ok(Expression::Closure {
            params,
            return_type,
            body: Box::new(body),
            span,
        })
    }

    // ── Argument list ─────────────────────────────────────────

    pub(super) fn parse_arg_list(&mut self) -> Result<Vec<Expression>, CompileError> {
        let mut args = Vec::new();
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen) {
            return Ok(args);
        }
        loop {
            self.skip_newlines();
            args.push(self.parse_expression()?);
            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    pub(super) fn parse_type_arg_list(&mut self) -> Result<Vec<AstType>, CompileError> {
        self.expect(&Token::Lt)?;
        let mut args = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Gt) {
                break;
            }
            args.push(self.parse_type()?);
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::Gt)?;
        Ok(args)
    }
}
