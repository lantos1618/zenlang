use crate::ast::{Expression, MatchArm, Statement};

use super::host_effects::{declared_host_effect, host_effect};
use super::targets::build_target_from_builder_add;
use super::{
    unsupported_build_script, BuildGraphError, BuildGraphInput, BuildTargetInput, HostEffect,
};

#[derive(Default)]
struct BuildProgramLowering {
    targets: Vec<BuildTargetInput>,
    declared_host_effects: Vec<HostEffect>,
    used_host_effects: Vec<HostEffect>,
    error: Option<BuildGraphError>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuildTargetAddContext {
    StaticGraphBody,
    DynamicExpression,
}

pub(super) fn build_graph_input_from_body(
    body: &Expression,
) -> Result<BuildGraphInput, BuildGraphError> {
    let mut lowering = BuildProgramLowering::default();
    lowering.collect_expr(body, BuildTargetAddContext::StaticGraphBody);
    if let Some(error) = lowering.error {
        return Err(error);
    }
    Ok(BuildGraphInput {
        targets: lowering.targets,
        declared_host_effects: lowering.declared_host_effects,
        used_host_effects: lowering.used_host_effects,
    })
}

impl BuildProgramLowering {
    fn collect_expr(&mut self, expr: &Expression, add_context: BuildTargetAddContext) {
        if let Some(effect) = host_effect(expr) {
            self.used_host_effects.push(effect);
        }
        if let Some(effect) = declared_host_effect(expr) {
            self.declared_host_effects.push(effect);
        }
        match (add_context, build_target_from_builder_add(expr)) {
            (_, Ok(None)) => {}
            (BuildTargetAddContext::DynamicExpression, _) => {
                self.error.get_or_insert_with(|| {
                    unsupported_build_script("build targets must be added in the deterministic build graph body")
                });
            }
            (BuildTargetAddContext::StaticGraphBody, Ok(Some(target))) => {
                self.targets.push(target);
            }
            (BuildTargetAddContext::StaticGraphBody, Err(error)) => {
                self.error.get_or_insert(error);
            }
        }

        match expr {
            Expression::BinaryOp { left, right, .. }
            | Expression::IndexAccess {
                object: left,
                index: right,
                ..
            } => {
                self.collect_dynamic_expr(left);
                self.collect_dynamic_expr(right);
            }
            Expression::UnaryOp { operand: child, .. }
            | Expression::MemberAccess { object: child, .. }
            | Expression::Loop { body: child, .. }
            | Expression::Closure { body: child, .. }
            | Expression::Cast { expr: child, .. }
            | Expression::Defer { expr: child, .. }
            | Expression::Await { expr: child, .. } => self.collect_dynamic_expr(child),
            Expression::FunctionCall { args, .. }
            | Expression::ArrayLiteral { elements: args, .. } => self.collect_dynamic_exprs(args),
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_expr(receiver, add_context);
                self.collect_dynamic_exprs(args);
            }
            Expression::StructLiteral { fields, .. } => {
                self.collect_dynamic_exprs(fields.iter().map(|(_, field)| field));
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.collect_dynamic_expr(payload);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.collect_dynamic_expr(scrutinee);
                for MatchArm { guard, body, .. } in arms {
                    self.collect_dynamic_exprs(guard.iter().chain(std::iter::once(body)));
                }
            }
            Expression::If {
                condition,
                then_body: body,
                else_body,
                ..
            } => {
                self.collect_dynamic_expr(condition);
                self.collect_dynamic_expr(body);
                if let Some(else_body) = else_body {
                    self.collect_dynamic_expr(else_body);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.collect_statement(statement, add_context);
                }
                if let Some(expr) = expr {
                    self.collect_expr(expr, add_context);
                }
            }
            Expression::StringInterpolation { parts, .. } => {
                self.collect_dynamic_exprs(parts.iter().filter_map(|part| match part {
                    crate::ast::StringPart::Expr(expr) => Some(expr),
                    crate::ast::StringPart::Literal(_) => None,
                }));
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::LoopControl { .. } => {}
        }
    }

    fn collect_statement(&mut self, statement: &Statement, add_context: BuildTargetAddContext) {
        match statement {
            Statement::Expression { expr: value, .. } => self.collect_expr(value, add_context),
            Statement::VarDecl { value, .. } => self.collect_dynamic_expr(value),
            Statement::Assignment { target, value, .. } => {
                self.collect_dynamic_expr(target);
                self.collect_dynamic_expr(value);
            }
        }
    }

    fn collect_dynamic_expr(&mut self, expr: &Expression) {
        self.collect_expr(expr, BuildTargetAddContext::DynamicExpression);
    }

    fn collect_dynamic_exprs<'a>(&mut self, exprs: impl IntoIterator<Item = &'a Expression>) {
        for expr in exprs {
            self.collect_dynamic_expr(expr);
        }
    }
}
