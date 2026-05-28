use super::*;

impl TypeChecker {
    pub(super) fn check_identifier_expr(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        if matches!(name, "true" | "false") {
            return typed_ok(TypedExprKind::BoolLiteral(name == "true"), Type::Bool, span);
        }

        let ty = if let Some(info) = self.lookup_var_info(name) {
            info.ty.clone()
        } else {
            if !self.structs.contains_key(name)
                && !self.enums.contains_key(name)
                && !self.functions.contains_key(name)
                && !self.imports.contains(name)
            {
                self.push_error(E3040, format!("undefined variable `{}`", name), span);
            }
            Type::Unknown
        };

        typed_ok(TypedExprKind::Variable(name.to_owned()), ty, span)
    }

    pub(super) fn check_block_expr(
        &mut self,
        statements: &[crate::ast::Statement],
        expr: &Option<Box<Expression>>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let (typed_stmts, typed_tail, ty) = self.with_scope(|checker| {
            let typed_stmts = statements
                .iter()
                .map(|stmt| checker.check_statement(stmt))
                .collect::<Result<Vec<_>, _>>()?;
            let typed_tail = match expr {
                Some(e) => Some(Box::new(checker.check_expr(e)?)),
                None => None,
            };
            let ty = typed_tail.as_ref().map_or(Type::Void, |e| e.ty.clone());
            Ok((typed_stmts, typed_tail, ty))
        })?;

        typed_ok(
            TypedExprKind::Block(TypedBlock {
                statements: typed_stmts,
                expr: typed_tail,
                ty: ty.clone(),
                span,
            }),
            ty,
            span,
        )
    }

    pub(super) fn check_cast_expr(
        &mut self,
        expr: &Expression,
        target_type: &AstType,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let checked_expr = self.check_expr(expr)?;
        let from_type = checked_expr.ty.clone();
        let to_type = self.resolve_type(target_type);

        typed_ok(
            TypedExprKind::Cast {
                expr: Box::new(checked_expr),
                from_type,
                to_type: to_type.clone(),
            },
            to_type,
            span,
        )
    }

    pub(super) fn check_string_interpolation_expr(
        &mut self,
        parts: &[StringPart],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_parts = parts
            .iter()
            .map(|part| match part {
                StringPart::Literal(s) => Ok(TypedStringPart::Literal(s.clone())),
                StringPart::Expr(e) => Ok(TypedStringPart::Expr(self.check_expr(e)?)),
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;

        typed_ok(
            TypedExprKind::StringInterpolation { parts: typed_parts },
            Type::Str,
            span,
        )
    }

    pub(super) fn check_defer_expr(
        &mut self,
        expr: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let checked_expr = self.check_expr(expr)?;
        self.pending_defers.push(checked_expr);

        typed_ok(
            TypedExprKind::Block(TypedBlock {
                statements: Vec::new(),
                expr: None,
                ty: Type::Void,
                span,
            }),
            Type::Void,
            span,
        )
    }

    pub(super) fn check_closure_expr(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let had_errors = !self.diagnostics.is_empty();
        let (param_types, typed_body) = self.with_scope(|checker| {
            let mut param_types = Vec::with_capacity(params.len());
            for param in params {
                let ty = checker.resolve_type(&param.ty);
                checker.define_var_with_mutability(&param.name, ty.clone(), param.mutable);
                param_types.push(ty);
            }
            Ok((param_types, checker.check_expr(body)?))
        })?;
        let ret = return_type
            .as_ref()
            .map_or_else(|| typed_body.ty.clone(), |ty| self.resolve_type(ty));

        if !had_errors {
            self.push_error(
                E3056,
                "closure expressions are gated until closure lowering and ABI are implemented",
                span,
            );
        }
        let closure_type = Type::Function {
            params: param_types,
            ret: Box::new(ret),
        };
        typed_ok(
            TypedExprKind::Block(TypedBlock {
                statements: Vec::new(),
                expr: None,
                ty: closure_type.clone(),
                span,
            }),
            closure_type,
            span,
        )
    }
}
