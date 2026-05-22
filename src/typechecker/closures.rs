//! Capture analysis for closures — walks typed expression trees to find
//! references to outer-scope variables.
#![allow(clippy::result_large_err)]

use std::collections::{HashMap, HashSet};

use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

impl TypeChecker {
    pub(super) fn check_closure_expr(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let outer_vars: HashMap<String, Type> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.vars.iter())
            .map(|(name, var)| (name.clone(), var.ty.clone()))
            .collect();

        self.push_scope();
        let mut param_types = Vec::new();
        let mut param_names = HashSet::new();
        for param in params {
            let ty = self.resolve_type(&param.ty);
            self.define_var_with_mutability(&param.name, ty.clone(), param.mutable);
            param_types.push(ty);
            param_names.insert(param.name.clone());
        }
        let typed_body = self.check_expr(body)?;
        self.pop_scope();

        let ret_type = if let Some(return_type) = return_type {
            self.resolve_type(return_type)
        } else {
            typed_body.ty.clone()
        };

        let mut captures = Vec::new();
        let mut seen = HashSet::new();
        collect_captures(
            &typed_body,
            &param_names,
            &outer_vars,
            &mut captures,
            &mut seen,
        );

        let fn_name = format!("__closure_{}_{}", span.start, span.end);
        let env_type = if captures.is_empty() {
            String::new()
        } else {
            format!("__env_{}_{}", span.start, span.end)
        };

        let fn_type = Type::Function {
            params: param_types,
            ret: Box::new(ret_type),
        };

        Ok(TypedExpression {
            kind: TypedExprKind::Closure {
                fn_name,
                env_type,
                captures,
            },
            ty: fn_type,
            span,
        })
    }
}

/// Walk a typed expression tree and collect references to outer-scope variables.
pub(super) fn collect_captures(
    expr: &TypedExpression,
    params: &HashSet<String>,
    outer_vars: &HashMap<String, Type>,
    captures: &mut Vec<Capture>,
    seen: &mut HashSet<String>,
) {
    match &expr.kind {
        TypedExprKind::Variable(name)
            if !params.contains(name)
                && outer_vars.contains_key(name)
                && seen.insert(name.clone()) =>
        {
            captures.push(Capture {
                name: name.clone(),
                ty: outer_vars[name].clone(),
                by_ref: false,
            });
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
