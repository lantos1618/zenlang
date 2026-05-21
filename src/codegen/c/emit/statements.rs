use super::*;

impl CEmitter {
    pub(in crate::codegen::c) fn emit_block_body(&mut self, block: &TypedBlock) {
        for stmt in &block.statements {
            self.emit_statement(stmt);
        }
        if let Some(ref expr) = block.expr {
            let val = self.emit_expr_to_stmt(expr);
            if !val.is_empty() {
                self.line(&val);
            }
        }
    }

    pub(in crate::codegen::c) fn emit_statement(&mut self, stmt: &TypedStatement) {
        match &stmt.kind {
            TypedStatementKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            } => {
                let c_ty = self.c_type(ty);
                let val = self.emit_expr_inline(value);
                if *mutable {
                    self.line(&format!("{} {} = {};", c_ty, c_ident(name), val));
                } else {
                    self.line(&format!("const {} {} = {};", c_ty, c_ident(name), val));
                }
            }
            TypedStatementKind::Expression(expr) => {
                let s = self.emit_expr_to_stmt(expr);
                if !s.is_empty() {
                    self.line(&s);
                }
            }
        }
    }

    pub(in crate::codegen::c) fn emit_expr_to_stmt(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            TypedExprKind::Block(block) => {
                self.emit_block_body(block);
                String::new()
            }
            TypedExprKind::Match {
                scrutinee,
                arms,
                kind,
            } => {
                self.emit_match(scrutinee, arms, kind, None);
                String::new()
            }
            TypedExprKind::Break => "break;".into(),
            TypedExprKind::Continue => "continue;".into(),
            TypedExprKind::LoopControl { action, label } => format!("goto {label}_{action};"),
            TypedExprKind::Assign { target, value } => {
                let t = self.emit_expr_inline(target);
                let v = self.emit_expr_inline(value);
                format!("{} = {};", t, v)
            }
            _ => {
                let inline = self.emit_expr_inline(expr);
                if inline.is_empty() {
                    String::new()
                } else {
                    format!("{};", inline)
                }
            }
        }
    }
}
