use super::*;

impl CEmitter {
    pub(super) fn emit_statement(&mut self, stmt: &TypedStatement) {
        match &stmt.kind {
            TypedStatementKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            } => {
                let c_ty = Self::c_type(ty);
                let val = self.emit_expr_inline(value);
                // For an immutable pointer binding, const the pointer itself
                // (`T* const p`), not the pointee (`const T* p`) — otherwise
                // assigning it into a non-const field discards qualifiers.
                let decl = if matches!(ty, Type::Function { .. }) {
                    // A function-typed binding needs the function-pointer
                    // declarator form `ret (*name)(params)`.
                    format!("{} = {};", c_declarator(ty, name), val)
                } else if !*mutable
                    && matches!(ty, Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_))
                {
                    format!("{} const {} = {};", c_ty, c_ident(name), val)
                } else {
                    let qualifier = c_const_qualifier(*mutable);
                    format!("{qualifier}{} {} = {};", c_ty, c_ident(name), val)
                };
                self.line(&decl);
            }
            TypedStatementKind::Expression(expr) => {
                self.emit_expr_statement(expr);
            }
        }
    }

    pub(super) fn emit_expr_statement(&mut self, expr: &TypedExpression) {
        let stmt = self.emit_expr_to_stmt(expr);
        if !stmt.is_empty() {
            self.line(&stmt);
        }
    }

    pub(super) fn emit_expr_to_stmt(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            TypedExprKind::Block(block) => {
                self.emit_block_body_with_result(block, None);
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

    pub(super) fn emit_expr_inline(&mut self, expr: &TypedExpression) -> String {
        match &expr.kind {
            TypedExprKind::IntLiteral(v) => format!("{}LL", v),
            TypedExprKind::FloatLiteral(v) => format_float(*v),
            TypedExprKind::StringLiteral(s) => {
                let escaped = c_escape_string(s);
                c_static_str_literal(&escaped)
            }
            TypedExprKind::BoolLiteral(b) => b.to_string(),
            TypedExprKind::Variable(name) => c_ident(name),

            TypedExprKind::BinaryOp { op, left, right } => {
                let l = self.emit_expr_inline(left);
                let r = self.emit_expr_inline(right);
                let op_str = op.symbol();
                format!("({} {} {})", l, op_str, r)
            }

            TypedExprKind::UnaryOp { op, operand } => {
                let o = self.emit_expr_inline(operand);
                format!("({}{})", op.symbol(), o)
            }

            TypedExprKind::FunctionCall { function, args } => {
                let name = c_func_ident(function);
                let str_args = self.extern_str_args.get(function).cloned();
                let arg_strs: Vec<_> = args
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let emitted = self.emit_expr_inline(a);
                        // Marshal a `Str` argument to an `@extern` param into a
                        // null-terminated `const char*` (zen_str.ptr).
                        match &str_args {
                            Some(positions) if positions.contains(&i) => format!("({}).ptr", emitted),
                            _ => emitted,
                        }
                    })
                    .collect();
                format!("{}({})", name, arg_strs.join(", "))
            }

            TypedExprKind::Intrinsic { name, args } => self.emit_intrinsic(name, args, &expr.ty),

            TypedExprKind::FieldAccess { object, field } => {
                let obj = self.emit_expr_inline(object);
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
                self.emit_struct_literal(type_name, fields)
            }

            TypedExprKind::EnumVariant {
                type_name,
                variant,
                payload,
            } => self.emit_enum_variant_literal(type_name, variant, payload.as_deref()),

            TypedExprKind::ArrayLiteral { elements } => self.emit_array_literal(elements),

            TypedExprKind::Cast { expr, to_type, .. } => {
                let e = self.emit_expr_inline(expr);
                let ty = Self::c_type(to_type);
                format!("(({}){})", ty, e)
            }

            TypedExprKind::Ref(inner) | TypedExprKind::MutRef(inner) => {
                let e = self.emit_expr_inline(inner);
                format!("(&{})", e)
            }
            TypedExprKind::Deref(inner) => {
                let e = self.emit_expr_inline(inner);
                format!("(*{})", e)
            }

            TypedExprKind::StringInterpolation { parts } => self.emit_string_interpolation(parts),

            TypedExprKind::Assign { target, value } => {
                let t = self.emit_expr_inline(target);
                let v = self.emit_expr_inline(value);
                format!("({} = {})", t, v)
            }

            TypedExprKind::Block(block) => {
                if block.statements.is_empty() {
                    if let Some(ref e) = block.expr {
                        return self.emit_expr_inline(e);
                    }
                }
                let tmp = self.fresh_tmp();
                let ty = Self::c_type(&block.ty);
                self.line(&format!("{} {};", ty, tmp));
                self.line("{");
                self.indent();
                self.emit_block_body_with_result(block, Some(&tmp));
                self.dedent();
                self.line("}");
                tmp
            }

            TypedExprKind::Match {
                scrutinee,
                arms,
                kind,
            } => {
                let tmp = self.fresh_tmp();
                let ty = Self::c_type(&expr.ty);
                self.line(&format!("{} {};", ty, tmp));
                self.emit_match(scrutinee, arms, kind, Some(&tmp));
                tmp
            }

            TypedExprKind::LoopControl { action, label } => format!("goto {label}_{action}"),

            // `@await` is not yet lowered to a state machine (ASYNC_PLAN.md
            // milestone 1). The typechecker rejects any program containing an
            // `@async` function before codegen runs, so this is unreachable for
            // an accepted program.
            TypedExprKind::Await { .. } => {
                unreachable!("async lowering not implemented; gated in typechecker")
            }
        }
    }
}
