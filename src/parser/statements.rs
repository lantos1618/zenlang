use super::*;

#[derive(Clone, Copy)]
struct UntypedVarDeclMode {
    mutable: bool,
    constant: bool,
}

impl UntypedVarDeclMode {
    fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::ConstAssign => Some(Self {
                mutable: false,
                constant: true,
            }),
            Token::DeclareAssign => Some(Self {
                mutable: true,
                constant: false,
            }),
            Token::Assign => Some(Self {
                mutable: false,
                constant: false,
            }),
            _ => None,
        }
    }
}

impl Parser {
    // ── Statements ────────────────────────────────────────────

    pub(super) fn parse_statement_or_expr(&mut self) -> Result<StmtOrExpr, CompileError> {
        self.skip_newlines();

        // Variable declaration: `name := expr` or `name ::= expr` or `name: Type = expr`
        if let Token::Identifier(ref name) = self.peek().clone() {
            let name = name.clone();

            let next = self.peek_ahead(1);
            if let Some(mode) = UntypedVarDeclMode::from_token(next) {
                let (_, name_span) = self.advance(); // consume name
                self.advance(); // consume assignment token
                return self.finish_var_decl(name, name_span, None, mode.mutable, mode.constant);
            }

            if matches!(next, Token::Colon) && self.is_typed_var_decl() {
                // Could be `name: Type = expr` (typed var decl)
                // or `name: Type` (type annotation — rare)
                // Peek further: name : Type = expr
                let (_, name_span) = self.advance(); // consume name
                self.advance(); // consume :
                let ty = self.parse_type()?;
                self.skip_newlines();
                self.expect(&Token::Assign)?;
                return self.finish_var_decl(name, name_span, Some(ty), false, false);
            }
        }

        // Parse as expression, then check for assignment
        let expr = self.parse_expression()?;

        // Check for assignment: `lhs = rhs`
        self.skip_newlines_if_continuation();
        if matches!(self.peek(), Token::Assign) {
            // This is an assignment to an existing variable/field
            self.advance(); // consume =
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

    /// Check if we have `name: Type = expr` pattern.
    fn is_typed_var_decl(&self) -> bool {
        // We're at `name`, peek_ahead(1) is `:`, check if there's eventually a `=`
        let mut i = self.pos + 2; // skip name, `:`, now at type
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
                Some(Token::Newline) if depth == 0 => return false,
                Some(Token::EOF) | None => return false,
                Some(Token::LBrace) if depth == 0 => return false,
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
