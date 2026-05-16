use super::*;

impl Parser {
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

            let stmt_or_expr = self.parse_statement_or_expr()?;

            match stmt_or_expr {
                StmtOrExpr::Stmt(stmt) => {
                    statements.push(stmt);
                }
                StmtOrExpr::Expr(expr) => {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBrace) {
                        final_expr = Some(Box::new(expr));
                    } else {
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

    pub(super) fn is_closure(&self) -> bool {
        let mut i = self.pos + 1;
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
                        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                            i += 1;
                        }
                        return matches!(
                            self.tokens.get(i).map(|(t, _)| t),
                            Some(Token::LBrace) | Some(Token::Identifier(_))
                        );
                    }
                    i += 1;
                }
                Some(Token::Colon) if depth == 1 => return true,
                Some(Token::Comma) if depth == 1 => return true,
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    pub(super) fn parse_closure(&mut self) -> Result<Expression, CompileError> {
        let start = self.peek_span();
        self.advance();
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
            if matches!(self.peek(), Token::Gt | Token::ShiftRight) {
                break;
            }
            args.push(self.parse_type()?);
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect_gt()?;
        Ok(args)
    }
}
