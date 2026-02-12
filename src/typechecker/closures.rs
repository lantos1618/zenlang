//! Capture analysis for closures — walks typed expression trees to find
//! references to outer-scope variables.

use std::collections::{HashMap, HashSet};

use crate::ast::typed::*;

/// Walk a typed expression tree and collect references to outer-scope variables.
pub(super) fn collect_captures(
    expr: &TypedExpression,
    params: &HashSet<String>,
    outer_vars: &HashMap<String, Type>,
    captures: &mut Vec<Capture>,
    seen: &mut HashSet<String>,
) {
    match &expr.kind {
        TypedExprKind::Variable(name) => {
            if !params.contains(name) && outer_vars.contains_key(name) && seen.insert(name.clone())
            {
                captures.push(Capture {
                    name: name.clone(),
                    ty: outer_vars[name].clone(),
                    by_ref: false,
                });
            }
        }
        TypedExprKind::BinaryOp { left, right, .. } => {
            collect_captures(left, params, outer_vars, captures, seen);
            collect_captures(right, params, outer_vars, captures, seen);
        }
        TypedExprKind::UnaryOp { operand, .. } => {
            collect_captures(operand, params, outer_vars, captures, seen);
        }
        TypedExprKind::FunctionCall { args, .. } => {
            for arg in args {
                collect_captures(arg, params, outer_vars, captures, seen);
            }
        }
        TypedExprKind::FieldAccess { object, .. } => {
            collect_captures(object, params, outer_vars, captures, seen);
        }
        TypedExprKind::IndexAccess { object, index } => {
            collect_captures(object, params, outer_vars, captures, seen);
            collect_captures(index, params, outer_vars, captures, seen);
        }
        TypedExprKind::StructLiteral { fields, .. } => {
            for (_, val) in fields {
                collect_captures(val, params, outer_vars, captures, seen);
            }
        }
        TypedExprKind::EnumVariant {
            payload: Some(p), ..
        } => {
            collect_captures(p, params, outer_vars, captures, seen);
        }
        TypedExprKind::ArrayLiteral { elements } => {
            for e in elements {
                collect_captures(e, params, outer_vars, captures, seen);
            }
        }
        TypedExprKind::Match {
            scrutinee, arms, ..
        } => {
            collect_captures(scrutinee, params, outer_vars, captures, seen);
            for arm in arms {
                collect_captures_block(&arm.body, params, outer_vars, captures, seen);
            }
        }
        TypedExprKind::Cast { expr, .. } => {
            collect_captures(expr, params, outer_vars, captures, seen);
        }
        TypedExprKind::Ref(inner) | TypedExprKind::MutRef(inner) | TypedExprKind::Deref(inner) => {
            collect_captures(inner, params, outer_vars, captures, seen);
        }
        TypedExprKind::Assign { target, value } => {
            collect_captures(target, params, outer_vars, captures, seen);
            collect_captures(value, params, outer_vars, captures, seen);
        }
        TypedExprKind::Block(block) => {
            collect_captures_block(block, params, outer_vars, captures, seen);
        }
        TypedExprKind::Return(Some(v)) => {
            collect_captures(v, params, outer_vars, captures, seen);
        }
        TypedExprKind::StringInterpolation { parts } => {
            for part in parts {
                if let TypedStringPart::Expr(e) = part {
                    collect_captures(e, params, outer_vars, captures, seen);
                }
            }
        }
        TypedExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_captures(arg, params, outer_vars, captures, seen);
            }
        }
        _ => {}
    }
}

pub(super) fn collect_captures_block(
    block: &TypedBlock,
    params: &HashSet<String>,
    outer_vars: &HashMap<String, Type>,
    captures: &mut Vec<Capture>,
    seen: &mut HashSet<String>,
) {
    for stmt in &block.statements {
        match &stmt.kind {
            TypedStatementKind::VarDecl { value, .. } => {
                collect_captures(value, params, outer_vars, captures, seen);
            }
            TypedStatementKind::Expression(e) => {
                collect_captures(e, params, outer_vars, captures, seen);
            }
        }
    }
    if let Some(ref e) = block.expr {
        collect_captures(e, params, outer_vars, captures, seen);
    }
}
