use std::collections::HashSet;

use crate::ast::types::Param;
use crate::ast::{Expression, Pattern, Statement};

/// Rewrite references to sibling module functions to their `<prefix>_` form.
///
/// When a stdlib namespace module is spliced into an importing program, its
/// functions are renamed `fn` → `prefix_fn`. This walks a function body and
/// rewrites every call/identifier that names a sibling function so recursion
/// and cross-calls keep resolving after the rename.
///
/// The walk is scope-aware: a name bound by a parameter, a `:=` local, a
/// closure parameter, or a match-arm pattern is *not* rewritten within its
/// scope, so a local that happens to share a sibling function's name keeps its
/// local meaning instead of being silently redirected to the function.
pub(super) fn rename_function_body(
    body: &mut Expression,
    names: &HashSet<String>,
    prefix: &str,
    params: &[Param],
) {
    let mut shadowed: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
    rewrite(body, names, prefix, &mut shadowed);
}

fn rewrite(
    expr: &mut Expression,
    names: &HashSet<String>,
    prefix: &str,
    shadowed: &mut HashSet<String>,
) {
    match expr {
        Expression::Identifier { name, .. } => maybe_prefix(name, names, prefix, shadowed),
        Expression::FunctionCall {
            name, module, args, ..
        } => {
            if module.is_none() {
                maybe_prefix(name, names, prefix, shadowed);
            }
            rename_each(args, names, prefix, shadowed);
        }
        Expression::MethodCall { receiver, args, .. } => {
            rewrite(receiver, names, prefix, shadowed);
            rename_each(args, names, prefix, shadowed);
        }
        Expression::BinaryOp { left, right, .. } => {
            rewrite(left, names, prefix, shadowed);
            rewrite(right, names, prefix, shadowed);
        }
        Expression::UnaryOp { operand, .. } => rewrite(operand, names, prefix, shadowed),
        Expression::MemberAccess { object, .. } => rewrite(object, names, prefix, shadowed),
        Expression::IndexAccess { object, index, .. } => {
            rewrite(object, names, prefix, shadowed);
            rewrite(index, names, prefix, shadowed);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                rewrite(value, names, prefix, shadowed);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                rewrite(payload, names, prefix, shadowed);
            }
        }
        Expression::ArrayLiteral { elements, .. } => rename_each(elements, names, prefix, shadowed),
        Expression::Match {
            scrutinee, arms, ..
        } => {
            rewrite(scrutinee, names, prefix, shadowed);
            for arm in arms {
                // Pattern bindings shadow within this arm's guard and body only.
                let mut bound = Vec::new();
                pattern_binds(&arm.pattern, &mut bound);
                let added = enter(shadowed, bound);
                if let Some(guard) = &mut arm.guard {
                    rewrite(guard, names, prefix, shadowed);
                }
                rewrite(&mut arm.body, names, prefix, shadowed);
                exit(shadowed, added);
            }
        }
        Expression::Loop { body, .. } => rewrite(body, names, prefix, shadowed),
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            rewrite(condition, names, prefix, shadowed);
            rewrite(then_body, names, prefix, shadowed);
            if let Some(else_body) = else_body {
                rewrite(else_body, names, prefix, shadowed);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            // A `:=` local shadows the rest of its block (statements after it
            // and the tail expr), but not its own initializer.
            let mut added = Vec::new();
            for statement in statements {
                rename_stmt_refs(statement, names, prefix, shadowed);
                if let Statement::VarDecl { name, .. } = statement {
                    added.extend(enter(shadowed, std::iter::once(name.clone())));
                }
            }
            if let Some(expr) = expr {
                rewrite(expr, names, prefix, shadowed);
            }
            exit(shadowed, added);
        }
        Expression::Closure { params, body, .. } => {
            let added = enter(shadowed, params.iter().map(|p| p.name.clone()));
            rewrite(body, names, prefix, shadowed);
            exit(shadowed, added);
        }
        Expression::Cast { expr, .. } => rewrite(expr, names, prefix, shadowed),
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let crate::ast::expressions::StringPart::Expr(expr) = part {
                    rewrite(expr, names, prefix, shadowed);
                }
            }
        }
        Expression::Defer { expr, .. } | Expression::Await { expr, .. } => {
            rewrite(expr, names, prefix, shadowed)
        }
        Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::LoopControl { .. } => {}
    }
}

fn rename_stmt_refs(
    statement: &mut Statement,
    names: &HashSet<String>,
    prefix: &str,
    shadowed: &mut HashSet<String>,
) {
    match statement {
        Statement::VarDecl { value, .. } => rewrite(value, names, prefix, shadowed),
        Statement::Assignment { target, value, .. } => {
            rewrite(target, names, prefix, shadowed);
            rewrite(value, names, prefix, shadowed);
        }
        Statement::Expression { expr, .. } => rewrite(expr, names, prefix, shadowed),
    }
}

fn rename_each(
    exprs: &mut [Expression],
    names: &HashSet<String>,
    prefix: &str,
    shadowed: &mut HashSet<String>,
) {
    for expr in exprs {
        rewrite(expr, names, prefix, shadowed);
    }
}

/// Collect the names a pattern binds. When unsure (e.g. a struct field with no
/// explicit sub-pattern is shorthand for binding that field), we include the
/// name: over-shadowing only skips a rename, while under-shadowing is the bug.
fn pattern_binds(pattern: &Pattern, out: &mut Vec<String>) {
    match pattern {
        Pattern::Identifier { name, .. } => out.push(name.clone()),
        Pattern::Struct { fields, .. } => {
            for (field, sub) in fields {
                match sub {
                    Some(sub) => pattern_binds(sub, out),
                    None => out.push(field.clone()),
                }
            }
        }
        Pattern::Enum { payload, .. } => {
            if let Some(payload) = payload {
                pattern_binds(payload, out);
            }
        }
        Pattern::Wildcard { .. }
        | Pattern::Literal { .. }
        | Pattern::BoolTrue { .. }
        | Pattern::BoolFalse { .. } => {}
    }
}

/// Insert `new` names into the shadow set, returning only those actually added
/// (so a nested binding that re-shadows an already-shadowed name doesn't remove
/// the outer one on scope exit).
fn enter(shadowed: &mut HashSet<String>, new: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut added = Vec::new();
    for name in new {
        if shadowed.insert(name.clone()) {
            added.push(name);
        }
    }
    added
}

fn exit(shadowed: &mut HashSet<String>, added: Vec<String>) {
    for name in added {
        shadowed.remove(&name);
    }
}

fn maybe_prefix(
    name: &mut String,
    names: &HashSet<String>,
    prefix: &str,
    shadowed: &HashSet<String>,
) {
    if names.contains(name.as_str()) && !shadowed.contains(name.as_str()) {
        *name = format!("{prefix}_{name}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Declaration;

    /// Parse `source`, run the namespace rename over the body of the function
    /// named `target`, and return every identifier/call name in the result.
    fn renamed_names(source: &str, target: &str) -> Vec<String> {
        let tokens = crate::lexer::tokenize(source, 0).expect("tokenize");
        let program = crate::parser::parse(tokens, 0).expect("parse");
        let siblings: HashSet<String> = program
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Function { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        let (mut body, params) = program
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Function {
                    name, body, params, ..
                } if name == target => Some((body.clone(), params.clone())),
                _ => None,
            })
            .expect("target function");
        rename_function_body(&mut body, &siblings, "p", &params);
        let mut out = Vec::new();
        collect_names(&body, &mut out);
        out
    }

    fn collect_names(expr: &Expression, out: &mut Vec<String>) {
        match expr {
            Expression::Identifier { name, .. } => out.push(name.clone()),
            Expression::FunctionCall { name, args, .. } => {
                out.push(name.clone());
                args.iter().for_each(|a| collect_names(a, out));
            }
            Expression::BinaryOp { left, right, .. } => {
                collect_names(left, out);
                collect_names(right, out);
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for s in statements {
                    match s {
                        Statement::VarDecl { value, .. } => collect_names(value, out),
                        Statement::Expression { expr, .. } => collect_names(expr, out),
                        Statement::Assignment { target, value, .. } => {
                            collect_names(target, out);
                            collect_names(value, out);
                        }
                    }
                }
                if let Some(e) = expr {
                    collect_names(e, out);
                }
            }
            Expression::Closure { body, .. } => collect_names(body, out),
            _ => {}
        }
    }

    #[test]
    fn sibling_calls_are_prefixed_but_shadowing_param_is_not() {
        let names = renamed_names(
            "helper = (x: i64) i64 { x }\n\
             caller = (helper: i64) i64 {\n\
                 other()\n\
                 helper + 1\n\
             }\n\
             other = () i64 { 0 }\n",
            "caller",
        );
        // `other()` is a genuine sibling call -> prefixed.
        assert!(names.contains(&"p_other".to_string()), "names={names:?}");
        // `helper` is the parameter -> must keep its local meaning.
        assert!(names.contains(&"helper".to_string()), "names={names:?}");
        assert!(
            !names.contains(&"p_helper".to_string()),
            "shadowing param was wrongly prefixed: {names:?}"
        );
    }

    #[test]
    fn local_binding_shadows_sibling_for_rest_of_block() {
        let names = renamed_names(
            "helper = (x: i64) i64 { x }\n\
             caller = (n: i64) i64 {\n\
                 helper = n + 1\n\
                 helper + 2\n\
             }\n",
            "caller",
        );
        // The `:=` local named `helper` shadows the sibling function from its
        // declaration onward, so the tail `helper` stays local.
        assert!(names.contains(&"helper".to_string()), "names={names:?}");
        assert!(
            !names.contains(&"p_helper".to_string()),
            "local binding was wrongly prefixed: {names:?}"
        );
    }
}
