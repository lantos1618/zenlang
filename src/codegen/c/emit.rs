use crate::ast::expressions::{BinaryOp, UnaryOp};

use super::*;

impl CEmitter {
    // ── Block body ────────────────────────────────────────────

    pub(super) fn emit_block_body(&mut self, block: &TypedBlock) {
        for stmt in &block.statements {
            self.emit_statement(stmt);
        }
        if let Some(ref expr) = block.expr {
            // Block's trailing expression — if in a function context, this is a return value.
            // For now, emit it as a standalone expression (the caller wraps it as needed).
            let val = self.emit_expr_to_stmt(expr);
            if !val.is_empty() {
                self.line(&val);
            }
        }
    }

    // ── Statements ────────────────────────────────────────────

    pub(super) fn emit_statement(&mut self, stmt: &TypedStatement) {
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

    // ── Expressions ───────────────────────────────────────────

    /// Emit an expression as a C statement (with semicolon).
    pub(super) fn emit_expr_to_stmt(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            // These emit multiple lines — handle specially
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
            TypedExprKind::Return(val) => match val {
                Some(v) if self.current_defers.is_empty() => {
                    format!("return {};", self.emit_expr_inline(v))
                }
                None if self.current_defers.is_empty() => "return;".into(),
                Some(v) => {
                    self.emit_return_with_current_defers(Some(v));
                    String::new()
                }
                None => {
                    self.emit_return_with_current_defers(None);
                    String::new()
                }
            },
            TypedExprKind::Break => "break;".into(),
            TypedExprKind::Continue => "continue;".into(),
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

    fn emit_return_with_current_defers(&mut self, value: Option<&TypedExpression>) {
        match value {
            Some(expr) => {
                let tmp = self.fresh_tmp();
                let ty = self.c_type(&expr.ty);
                let value = self.emit_expr_inline(expr);
                self.line(&format!("{} {} = {};", ty, tmp, value));
                self.emit_current_defers();
                self.line(&format!("return {};", tmp));
            }
            None => {
                self.emit_current_defers();
                self.line("return;");
            }
        }
    }

    fn emit_current_defers(&mut self) {
        let defers = self.current_defers.clone();
        for defer_expr in &defers {
            let s = self.emit_expr_to_stmt(defer_expr);
            if !s.is_empty() {
                self.line(&s);
            }
        }
    }

    /// Emit an expression as a C inline expression (no semicolon).
    pub(super) fn emit_expr_inline(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            TypedExprKind::IntLiteral(v) => format!("{}LL", v),
            TypedExprKind::FloatLiteral(v) => format_float(*v).to_string(),
            TypedExprKind::StringLiteral(s) => {
                let escaped = c_escape_string(s);
                format!("zen_str_from_literal(\"{}\")", escaped)
            }
            TypedExprKind::BoolLiteral(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            TypedExprKind::Variable(name) => c_ident(name),

            TypedExprKind::BinaryOp { op, left, right } => {
                let l = self.emit_expr_inline(left);
                let r = self.emit_expr_inline(right);
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::ShiftLeft => "<<",
                    BinaryOp::ShiftRight => ">>",
                };
                format!("({} {} {})", l, op_str, r)
            }

            TypedExprKind::UnaryOp { op, operand } => {
                let o = self.emit_expr_inline(operand);
                match op {
                    UnaryOp::Neg => format!("(-{})", o),
                    UnaryOp::Not => format!("(!{})", o),
                    UnaryOp::BitNot => format!("(~{})", o),
                }
            }

            TypedExprKind::FunctionCall { function, args } => {
                let name = c_func_ident(function);
                let arg_strs: Vec<_> = args.iter().map(|a| self.emit_expr_inline(a)).collect();
                format!("{}({})", name, arg_strs.join(", "))
            }

            TypedExprKind::FieldAccess { object, field } => {
                let obj = self.emit_expr_inline(object);
                // If object is a pointer, use ->
                match &object.ty {
                    Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_) => {
                        format!("{}->{}", obj, c_ident(field))
                    }
                    _ => format!("{}.{}", obj, c_ident(field)),
                }
            }

            TypedExprKind::IndexAccess { object, index } => {
                let obj = self.emit_expr_inline(object);
                let idx = self.emit_expr_inline(index);
                format!("{}[{}]", obj, idx)
            }

            TypedExprKind::StructLiteral { type_name, fields } => {
                let name = c_ident(type_name);
                let field_strs: Vec<_> = fields
                    .iter()
                    .map(|(fname, fval)| {
                        let v = self.emit_expr_inline(fval);
                        format!(".{} = {}", c_ident(fname), v)
                    })
                    .collect();
                format!("({}){{ {} }}", name, field_strs.join(", "))
            }

            TypedExprKind::EnumVariant {
                type_name,
                variant,
                payload,
            } => {
                let name = c_ident(type_name);
                let var = c_ident(variant);
                match payload {
                    None => {
                        format!("({}){{ .tag = {}_{} }}", name, name, var)
                    }
                    Some(val) => {
                        let v = self.emit_expr_inline(val);
                        format!(
                            "({}){{ .tag = {}_{}, .data.{} = {} }}",
                            name,
                            name,
                            var,
                            var.to_lowercase(),
                            v
                        )
                    }
                }
            }

            TypedExprKind::ArrayLiteral { elements } => {
                let elems: Vec<_> = elements.iter().map(|e| self.emit_expr_inline(e)).collect();
                format!("{{ {} }}", elems.join(", "))
            }

            TypedExprKind::Cast { expr, to_type, .. } => {
                let e = self.emit_expr_inline(expr);
                let ty = self.c_type(to_type);
                format!("(({}){})", ty, e)
            }

            TypedExprKind::Ref(inner) => {
                let e = self.emit_expr_inline(inner);
                format!("(&{})", e)
            }
            TypedExprKind::MutRef(inner) => {
                let e = self.emit_expr_inline(inner);
                format!("(&{})", e)
            }
            TypedExprKind::Deref(inner) => {
                let e = self.emit_expr_inline(inner);
                format!("(*{})", e)
            }

            TypedExprKind::StringInterpolation { parts } => self.emit_string_interpolation(parts),

            TypedExprKind::Intrinsic { name, args } => self.emit_intrinsic(name, args, &expr.ty),

            TypedExprKind::Assign { target, value } => {
                let t = self.emit_expr_inline(target);
                let v = self.emit_expr_inline(value);
                format!("({} = {})", t, v)
            }

            TypedExprKind::Block(block) => {
                // GCC statement expression: ({ stmts; expr; })
                // For simple blocks with just an expr, inline it
                if block.statements.is_empty() {
                    if let Some(ref e) = block.expr {
                        return self.emit_expr_inline(e);
                    }
                }
                // Complex blocks — use a temp var
                let tmp = self.fresh_tmp();
                let ty = self.c_type(&block.ty);
                self.line(&format!("{} {};", ty, tmp));
                self.line("{");
                self.indent();
                self.emit_block_body(block);
                self.dedent();
                self.line("}");
                tmp
            }

            TypedExprKind::Return(val) => {
                // Shouldn't appear as inline expression, but handle it
                match val {
                    Some(v) => {
                        let ve = self.emit_expr_inline(v);
                        format!("({{ return {}; 0; }})", ve)
                    }
                    None => "(({ return; 0; }))".into(),
                }
            }

            TypedExprKind::Match {
                scrutinee,
                arms,
                kind,
            } => {
                // For inline match, we need a temp variable
                let tmp = self.fresh_tmp();
                let ty = self.c_type(&expr.ty);
                self.line(&format!("{} {};", ty, tmp));
                self.emit_match(scrutinee, arms, kind, Some(&tmp));
                tmp
            }

            TypedExprKind::Closure { fn_name, .. } => {
                // For now, just reference the generated function name
                c_ident(fn_name)
            }

            TypedExprKind::Break => "break".into(),
            TypedExprKind::Continue => "continue".into(),
            TypedExprKind::Error => "/* error */".into(),
        }
    }

    // ── Match / conditionals ──────────────────────────────────

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

    fn emit_enum_match(
        &mut self,
        scrutinee: &TypedExpression,
        arms: &[TypedMatchArm],
        result_var: Option<&str>,
    ) {
        let scrut = self.emit_expr_inline(scrutinee);
        self.line(&format!("switch ({}.tag) {{", scrut));
        self.indent();
        for arm in arms {
            match &arm.pattern {
                TypedPattern::EnumVariant {
                    type_name,
                    variant,
                    bindings,
                } => {
                    let tag = format!("{}_{}", c_ident(type_name), c_ident(variant));
                    self.line(&format!("case {}: {{", tag));
                    self.indent();
                    // Bind payload fields
                    for (binding_name, binding_ty) in bindings {
                        let ty = self.c_type(binding_ty);
                        self.line(&format!(
                            "{} {} = {}.data.{};",
                            ty,
                            c_ident(binding_name),
                            scrut,
                            c_ident(variant).to_lowercase()
                        ));
                    }
                    self.emit_block_body_with_result(&arm.body, result_var);
                    self.line("break;");
                    self.dedent();
                    self.line("}");
                }
                TypedPattern::Wildcard => {
                    self.line("default: {");
                    self.indent();
                    self.emit_block_body_with_result(&arm.body, result_var);
                    self.line("break;");
                    self.dedent();
                    self.line("}");
                }
                _ => {}
            }
        }
        self.dedent();
        self.line("}");
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
