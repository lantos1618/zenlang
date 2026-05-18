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
            TypedExprKind::Break => "break;".into(),
            TypedExprKind::Continue => "continue;".into(),
            TypedExprKind::LoopControl { action, label } => {
                format!("goto {label}_{action};")
            }
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

    /// Emit an expression as a C inline expression (no semicolon).
    pub(super) fn emit_expr_inline(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            TypedExprKind::IntLiteral(v) => format!("{}LL", v),
            TypedExprKind::FloatLiteral(v) => format_float(*v).to_string(),
            TypedExprKind::StringLiteral(s) => {
                let escaped = c_escape_string(s);
                c_static_str_literal(&escaped)
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
            TypedExprKind::LoopControl { action, label } => format!("goto {label}_{action}"),
            TypedExprKind::Error => "/* error */".into(),
        }
    }
}
