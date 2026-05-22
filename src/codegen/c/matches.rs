use super::*;

mod enum_match;

impl CEmitter {
    pub(super) fn emit_match(
        &mut self,
        scrutinee: &TypedExpression,
        arms: &[TypedMatchArm],
        kind: &MatchKind,
        result_var: Option<&str>,
    ) {
        match kind {
            MatchKind::ConditionalElse | MatchKind::Conditional => {
                self.emit_conditional(scrutinee, arms, result_var);
            }
            MatchKind::WhileLoop => {
                let cond = self.emit_expr_inline(scrutinee);
                self.line(&format!("while ({}) {{", cond));
                self.indent();
                if let Some(arm) = arms.first() {
                    self.emit_block_body(&arm.body);
                }
                self.dedent();
                self.line("}");
            }
            MatchKind::ControlledLoop { label } => {
                let cond = self.emit_expr_inline(scrutinee);
                self.line(&format!("while ({}) {{", cond));
                self.indent();
                if let Some(arm) = arms.first() {
                    self.emit_block_body(&arm.body);
                }
                self.line(&format!("{label}_next:"));
                self.line("continue;");
                self.dedent();
                self.line("}");
                self.line(&format!("{label}_done:;"));
            }
            MatchKind::EnumMatch => {
                self.emit_enum_match(scrutinee, arms, result_var);
            }
            MatchKind::ValueMatch => {
                self.emit_value_match(scrutinee, arms, result_var);
            }
        }
    }

    fn emit_conditional(
        &mut self,
        scrutinee: &TypedExpression,
        arms: &[TypedMatchArm],
        result_var: Option<&str>,
    ) {
        let cond = self.emit_expr_inline(scrutinee);

        let mut first = true;
        for arm in arms {
            match &arm.pattern {
                TypedPattern::Bool(true) => {
                    if first {
                        self.line(&format!("if ({}) {{", cond));
                        first = false;
                    } else {
                        self.line(&format!("else if ({}) {{", cond));
                    }
                }
                TypedPattern::Bool(false) => {
                    self.line("else {");
                }
                TypedPattern::Wildcard => {
                    if first {
                        self.line("{");
                        first = false;
                    } else {
                        self.line("else {");
                    }
                }
                _ => {
                    self.line(&format!("if ({}) {{", cond));
                    first = false;
                }
            }
            self.indent();
            self.emit_block_body_with_result(&arm.body, result_var);
            self.dedent();
            self.line("}");
        }
    }

    fn emit_value_match(
        &mut self,
        scrutinee: &TypedExpression,
        arms: &[TypedMatchArm],
        result_var: Option<&str>,
    ) {
        let scrut = self.emit_expr_inline(scrutinee);
        let mut first = true;
        for arm in arms {
            match &arm.pattern {
                TypedPattern::Value(val) => {
                    let v = self.emit_expr_inline(val);
                    if first {
                        self.line(&format!("if ({} == {}) {{", scrut, v));
                        first = false;
                    } else {
                        self.line(&format!("else if ({} == {}) {{", scrut, v));
                    }
                }
                TypedPattern::Wildcard => {
                    self.line("else {");
                }
                _ => {
                    if first {
                        self.line("{");
                        first = false;
                    } else {
                        self.line("else {");
                    }
                }
            }
            self.indent();
            self.emit_block_body_with_result(&arm.body, result_var);
            self.dedent();
            self.line("}");
        }
    }

    fn emit_block_body_with_result(&mut self, block: &TypedBlock, result_var: Option<&str>) {
        for stmt in &block.statements {
            self.emit_statement(stmt);
        }
        if let Some(ref expr) = block.expr {
            match result_var {
                Some(var) if expr.ty != Type::Never => {
                    let val = self.emit_expr_inline(expr);
                    self.line(&format!("{} = {};", var, val));
                }
                _ => {
                    let s = self.emit_expr_to_stmt(expr);
                    if !s.is_empty() {
                        self.line(&s);
                    }
                }
            }
        }
    }
}
