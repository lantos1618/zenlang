fn expected_resolver_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match expr {
        Expression::BinaryOp { left, right, .. } => {
            expected_resolver_expr_locals(left, scope_cursor, locals, expected);
            expected_resolver_expr_locals(right, scope_cursor, locals, expected);
        }
        Expression::UnaryOp { operand, .. } => {
            expected_resolver_expr_locals(operand, scope_cursor, locals, expected);
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MethodCall { receiver, args, .. } => {
            expected_resolver_expr_locals(receiver, scope_cursor, locals, expected);
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MemberAccess { object, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
        }
        Expression::IndexAccess { object, index, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
            expected_resolver_expr_locals(index, scope_cursor, locals, expected);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                expected_resolver_expr_locals(payload, scope_cursor, locals, expected);
            }
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                expected_resolver_expr_locals(element, scope_cursor, locals, expected);
            }
        }
        Expression::Match {
            scrutinee, arms, ..
        } => {
            expected_resolver_expr_locals(scrutinee, scope_cursor, locals, expected);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    expected_resolver_pattern_expr_locals(
                        &arm.pattern,
                        guard,
                        scope_cursor,
                        locals,
                        expected,
                    );
                }
                expected_resolver_pattern_expr_locals(
                    &arm.pattern,
                    &arm.body,
                    scope_cursor,
                    locals,
                    expected,
                );
            }
        }
        Expression::WhileLoop {
            condition, body, ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::Loop { body, .. } => {
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(then_body, scope_cursor, locals, expected);
            if let Some(else_body) = else_body {
                expected_resolver_child_expr_locals(else_body, scope_cursor, locals, expected);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            expected_resolver_block_locals(
                statements,
                expr.as_deref(),
                scope_cursor,
                locals,
                expected,
            );
        }
        Expression::Closure { params, body, .. } => {
            expected_resolver_closure_locals(params, body, scope_cursor, locals, expected);
        }
        Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let ast::StringPart::Expr(expr) = part {
                    expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
                }
            }
        }
        Expression::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        Expression::Identifier { .. }
        | Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::Break { .. }
        | Expression::Continue { .. }
        | Expression::LoopControl { .. }
        | Expression::Error { .. } => {}
    }
}
