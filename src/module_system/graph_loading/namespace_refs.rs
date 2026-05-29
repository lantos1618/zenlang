use std::collections::HashSet;

use crate::ast::{Expression, Statement};

/// Rewrite references to sibling module functions to their `<prefix>_` form.
///
/// When a stdlib namespace module is spliced into an importing program, its
/// functions are renamed `fn` → `prefix_fn`. This walks a function body and
/// rewrites every call/identifier that names a sibling function so recursion
/// and cross-calls keep resolving after the rename. Local bindings that happen
/// to shadow a function name are not tracked; stdlib modules avoid that.
pub(super) fn rename_expr_refs(expr: &mut Expression, names: &HashSet<String>, prefix: &str) {
    match expr {
        Expression::Identifier { name, .. } => maybe_prefix(name, names, prefix),
        Expression::FunctionCall {
            name, module, args, ..
        } => {
            if module.is_none() {
                maybe_prefix(name, names, prefix);
            }
            rename_each(args, names, prefix);
        }
        Expression::MethodCall { receiver, args, .. } => {
            rename_expr_refs(receiver, names, prefix);
            rename_each(args, names, prefix);
        }
        Expression::BinaryOp { left, right, .. } => {
            rename_expr_refs(left, names, prefix);
            rename_expr_refs(right, names, prefix);
        }
        Expression::UnaryOp { operand, .. } => rename_expr_refs(operand, names, prefix),
        Expression::MemberAccess { object, .. } => rename_expr_refs(object, names, prefix),
        Expression::IndexAccess { object, index, .. } => {
            rename_expr_refs(object, names, prefix);
            rename_expr_refs(index, names, prefix);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                rename_expr_refs(value, names, prefix);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                rename_expr_refs(payload, names, prefix);
            }
        }
        Expression::ArrayLiteral { elements, .. } => rename_each(elements, names, prefix),
        Expression::Match {
            scrutinee, arms, ..
        } => {
            rename_expr_refs(scrutinee, names, prefix);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    rename_expr_refs(guard, names, prefix);
                }
                rename_expr_refs(&mut arm.body, names, prefix);
            }
        }
        Expression::Loop { body, .. } => rename_expr_refs(body, names, prefix),
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            rename_expr_refs(condition, names, prefix);
            rename_expr_refs(then_body, names, prefix);
            if let Some(else_body) = else_body {
                rename_expr_refs(else_body, names, prefix);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            for statement in statements {
                rename_stmt_refs(statement, names, prefix);
            }
            if let Some(expr) = expr {
                rename_expr_refs(expr, names, prefix);
            }
        }
        Expression::Closure { body, .. } => rename_expr_refs(body, names, prefix),
        Expression::Cast { expr, .. } => rename_expr_refs(expr, names, prefix),
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let crate::ast::expressions::StringPart::Expr(expr) = part {
                    rename_expr_refs(expr, names, prefix);
                }
            }
        }
        Expression::Defer { expr, .. } => rename_expr_refs(expr, names, prefix),
        Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::LoopControl { .. } => {}
    }
}

fn rename_stmt_refs(statement: &mut Statement, names: &HashSet<String>, prefix: &str) {
    match statement {
        Statement::VarDecl { value, .. } => rename_expr_refs(value, names, prefix),
        Statement::Assignment { target, value, .. } => {
            rename_expr_refs(target, names, prefix);
            rename_expr_refs(value, names, prefix);
        }
        Statement::Expression { expr, .. } => rename_expr_refs(expr, names, prefix),
    }
}

fn rename_each(exprs: &mut [Expression], names: &HashSet<String>, prefix: &str) {
    for expr in exprs {
        rename_expr_refs(expr, names, prefix);
    }
}

fn maybe_prefix(name: &mut String, names: &HashSet<String>, prefix: &str) {
    if names.contains(name.as_str()) {
        *name = format!("{prefix}_{name}");
    }
}
