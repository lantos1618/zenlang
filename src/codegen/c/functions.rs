use super::*;

impl CEmitter {
    pub(super) fn emit_function(&mut self, func: &TypedFunction) {
        let ret = Self::c_type(&func.return_type);
        let name = c_func_ident(&func.name);
        let params = self.format_params(&func.params);
        self.line(&format!("{} {}({}) {{", ret, name, params));
        self.indent();

        for stmt in &func.body.statements {
            self.emit_statement(stmt);
        }

        if let Some(ref expr) = func.body.expr {
            if func.return_type != Type::Void && func.return_type != Type::Never {
                self.emit_return_expr(expr, &func.return_type, &func.defers);
            } else {
                self.emit_expr_statement(expr);
                self.emit_defers(&func.defers);
            }
        } else if !func.defers.is_empty() {
            self.emit_defers(&func.defers);
        }

        self.dedent();
        self.line("}");
    }

    fn emit_return_expr(
        &mut self,
        expr: &TypedExpression,
        return_type: &Type,
        defers: &[TypedExpression],
    ) {
        if defers.is_empty() && !matches!(expr.kind, TypedExprKind::Match { .. }) {
            let val = self.emit_expr_inline(expr);
            if !val.is_empty() {
                self.line(&format!("return {};", val));
            }
            return;
        }

        let tmp = self.fresh_tmp();
        let ty = Self::c_type(return_type);
        match &expr.kind {
            TypedExprKind::Match {
                scrutinee,
                arms,
                kind,
            } => {
                self.line(&format!("{} {};", ty, tmp));
                self.emit_match(scrutinee, arms, kind, Some(&tmp));
            }
            _ => {
                let val = self.emit_expr_inline(expr);
                self.line(&format!("{} {} = {};", ty, tmp, val));
            }
        }
        self.emit_defers(defers);
        self.line(&format!("return {};", tmp));
    }

    fn emit_defers(&mut self, defers: &[TypedExpression]) {
        for defer_expr in defers {
            self.emit_expr_statement(defer_expr);
        }
    }

    pub(super) fn format_params(&self, params: &[TypedParam]) -> String {
        if params.is_empty() {
            return "void".into();
        }
        params
            .iter()
            .map(|p| format!("{} {}", Self::c_type(&p.ty), c_ident(&p.name)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
