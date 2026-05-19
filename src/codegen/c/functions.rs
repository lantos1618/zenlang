use super::*;

impl CEmitter {
    pub(super) fn emit_function_forward_decl(&mut self, func: &TypedFunction) {
        let ret = self.c_type(&func.return_type);
        let name = c_func_ident(&func.name);
        let params = self.format_params(&func.params);
        self.line(&format!("{} {}({});", ret, name, params));
    }

    pub(super) fn emit_function(&mut self, func: &TypedFunction) {
        let ret = self.c_type(&func.return_type);
        let name = c_func_ident(&func.name);
        let params = self.format_params(&func.params);
        self.line(&format!("{} {}({}) {{", ret, name, params));
        self.indent();
        let previous_defers = std::mem::replace(&mut self.current_defers, func.defers.clone());

        for stmt in &func.body.statements {
            self.emit_statement(stmt);
        }

        if let Some(ref expr) = func.body.expr {
            if func.return_type != Type::Void && func.return_type != Type::Never {
                if !func.defers.is_empty() {
                    let tmp = self.fresh_tmp();
                    let ty = self.c_type(&func.return_type);
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
                    for defer_expr in &func.defers {
                        let s = self.emit_expr_to_stmt(defer_expr);
                        if !s.is_empty() {
                            self.line(&s);
                        }
                    }
                    self.line(&format!("return {};", tmp));
                } else {
                    match &expr.kind {
                        TypedExprKind::Match {
                            scrutinee,
                            arms,
                            kind,
                        } => {
                            let tmp = self.fresh_tmp();
                            let ty = self.c_type(&func.return_type);
                            self.line(&format!("{} {};", ty, tmp));
                            self.emit_match(scrutinee, arms, kind, Some(&tmp));
                            self.line(&format!("return {};", tmp));
                        }
                        _ => {
                            let val = self.emit_expr_inline(expr);
                            if !val.is_empty() {
                                self.line(&format!("return {};", val));
                            }
                        }
                    }
                }
            } else {
                let s = self.emit_expr_to_stmt(expr);
                if !s.is_empty() {
                    self.line(&s);
                }
                for defer_expr in &func.defers {
                    let s = self.emit_expr_to_stmt(defer_expr);
                    if !s.is_empty() {
                        self.line(&s);
                    }
                }
            }
        } else if !func.defers.is_empty() {
            for defer_expr in &func.defers {
                let s = self.emit_expr_to_stmt(defer_expr);
                if !s.is_empty() {
                    self.line(&s);
                }
            }
        }

        self.current_defers = previous_defers;
        self.dedent();
        self.line("}");
    }

    fn format_params(&self, params: &[TypedParam]) -> String {
        if params.is_empty() {
            return "void".into();
        }
        params
            .iter()
            .map(|p| format!("{} {}", self.c_type(&p.ty), c_ident(&p.name)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(super) fn emit_global(&mut self, global: &TypedGlobal) {
        let ty = self.c_type(&global.ty);
        let name = c_ident(&global.name);
        let val = self.emit_expr_inline(&global.value);
        if global.mutable {
            self.line(&format!("{} {} = {};", ty, name, val));
        } else {
            self.line(&format!("const {} {} = {};", ty, name, val));
        }
    }
}
