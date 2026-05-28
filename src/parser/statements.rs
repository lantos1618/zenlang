use super::*;

impl Parser {
    pub(super) fn parse_statement_or_expr(&mut self) -> Result<StmtOrExpr, CompileError> {
        self.skip_newlines();

        if let Token::Identifier(ref name) = self.peek().clone() {
            let name = name.clone();

            let next = self.peek_ahead(1);
            if let Some((mutable, constant)) = match next {
                Token::ConstAssign => Some((false, true)),
                Token::DeclareAssign => Some((true, false)),
                Token::Assign => Some((false, false)),
                _ => None,
            } {
                let (_, name_span) = self.advance();
                self.advance();
                return self.finish_var_decl(name, name_span, None, mutable, constant);
            }

            if matches!(next, Token::Colon) && self.is_typed_var_decl() {
                let (_, name_span) = self.advance();
                self.advance();
                let ty = self.parse_type()?;
                self.skip_newlines();
                self.expect(&Token::Assign)?;
                return self.finish_var_decl(name, name_span, Some(ty), false, false);
            }
        }

        let expr = self.parse_expression()?;

        self.skip_newlines_if_continuation();
        if matches!(self.peek(), Token::Assign) {
            self.advance();
            self.skip_newlines();
            let value = self.parse_expression()?;
            let span = expr.span().merge(value.span());
            return Ok(StmtOrExpr::Stmt(Statement::Assignment {
                target: expr,
                value,
                span,
            }));
        }

        Ok(StmtOrExpr::Expr(expr))
    }

    fn is_typed_var_decl(&self) -> bool {
        let mut i = self.pos + 2;
        let mut depth = 0u32;
        loop {
            match self.tokens.get(i).map(|(t, _)| t) {
                Some(Token::Lt) => {
                    depth += 1;
                    i += 1;
                }
                Some(Token::Gt) => {
                    depth = depth.saturating_sub(1);
                    i += 1;
                }
                Some(Token::Assign) if depth == 0 => return true,
                Some(Token::Newline | Token::LBrace) if depth == 0 => return false,
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    fn finish_var_decl(
        &mut self,
        name: String,
        name_span: Span,
        ty: Option<AstType>,
        mutable: bool,
        constant: bool,
    ) -> Result<StmtOrExpr, CompileError> {
        self.skip_newlines();
        let value = self.parse_expression()?;
        let span = name_span.merge(value.span());
        Ok(StmtOrExpr::Stmt(Statement::VarDecl {
            name,
            ty,
            value,
            mutable,
            constant,
            span,
        }))
    }
}
