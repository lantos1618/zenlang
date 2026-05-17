use crate::ast::{Declaration, Expression, MatchArm, Program, Statement};

#[path = "lowering/dsl.rs"]
mod dsl;
#[path = "lowering/targets.rs"]
mod targets;

use dsl::{BuildTargetDslIdent, HostEffectResultVariant};
#[cfg(test)]
use dsl::{BuildTargetDslKind, BuildTargetField};
use targets::build_target_from_builder_add;

impl BuildGraph {
    pub fn from_build_program(program: &Program) -> Result<Self, BuildGraphError> {
        let build_body = program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                Declaration::Function { name, body, .. }
                    if name == BuildTargetDslIdent::Build.as_str() =>
                {
                    Some(body)
                }
                _ => None,
            })
            .ok_or(BuildGraphError::MissingBuildFunction)?;

        let mut lowering = BuildProgramLowering::default();
        lowering.collect_expr(build_body);
        Self::from_input(lowering.into_input()?)
    }
}

#[derive(Default)]
struct BuildProgramLowering {
    targets: Vec<BuildTargetInput>,
    declared_host_effects: Vec<HostEffect>,
    used_host_effects: Vec<HostEffect>,
    error: Option<BuildGraphError>,
}

impl BuildProgramLowering {
    fn into_input(self) -> Result<BuildGraphInput, BuildGraphError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(BuildGraphInput {
            targets: self.targets,
            declared_host_effects: self.declared_host_effects,
            used_host_effects: self.used_host_effects,
        })
    }

    fn collect_expr(&mut self, expr: &Expression) {
        if let Some(effect) = host_effect(expr) {
            self.used_host_effects.push(effect);
        }
        if let Some(effect) = declared_host_effect(expr) {
            self.declared_host_effects.push(effect);
        }
        match build_target_from_builder_add(expr) {
            Ok(Some(target)) => self.targets.push(target),
            Ok(None) => {}
            Err(error) => {
                if self.error.is_none() {
                    self.error = Some(error);
                }
            }
        }

        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.collect_expr(left);
                self.collect_expr(right);
            }
            Expression::UnaryOp { operand, .. } => self.collect_expr(operand),
            Expression::FunctionCall { args, .. }
            | Expression::ArrayLiteral { elements: args, .. } => {
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_expr(receiver);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            Expression::MemberAccess { object, .. } => self.collect_expr(object),
            Expression::IndexAccess { object, index, .. } => {
                self.collect_expr(object);
                self.collect_expr(index);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, field) in fields {
                    self.collect_expr(field);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.collect_expr(payload);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.collect_expr(scrutinee);
                for MatchArm { guard, body, .. } in arms {
                    if let Some(guard) = guard {
                        self.collect_expr(guard);
                    }
                    self.collect_expr(body);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            }
            | Expression::If {
                condition,
                then_body: body,
                ..
            } => {
                self.collect_expr(condition);
                self.collect_expr(body);
                if let Expression::If {
                    else_body: Some(else_body),
                    ..
                } = expr
                {
                    self.collect_expr(else_body);
                }
            }
            Expression::Loop { body, .. } => self.collect_expr(body),
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.collect_statement(statement);
                }
                if let Some(expr) = expr {
                    self.collect_expr(expr);
                }
            }
            Expression::Closure { body, .. } => self.collect_expr(body),
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.collect_expr(expr)
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let crate::ast::StringPart::Expr(expr) = part {
                        self.collect_expr(expr);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.collect_expr(start);
                self.collect_expr(end);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::LoopControl { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn collect_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VarDecl { value, .. } | Statement::Expression { expr: value, .. } => {
                self.collect_expr(value);
            }
            Statement::Assignment { target, value, .. } => {
                self.collect_expr(target);
                self.collect_expr(value);
            }
            Statement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.collect_statement(stmt);
                }
            }
        }
    }
}

fn declared_host_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::Match {
        scrutinee, arms, ..
    } = expr
    else {
        return None;
    };
    let has_fallback = arms.iter().any(|arm| host_effect_arm_declares_fallback(&arm.pattern));
    has_fallback.then(|| host_effect(scrutinee)).flatten()
}

fn host_effect_arm_declares_fallback(pattern: &crate::ast::Pattern) -> bool {
    match pattern {
        crate::ast::Pattern::Wildcard { .. } | crate::ast::Pattern::Identifier { .. } => true,
        crate::ast::Pattern::Enum { variant, .. } => {
            variant.parse::<HostEffectResultVariant>() == Ok(HostEffectResultVariant::Err)
        }
        _ => false,
    }
}

fn host_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::MethodCall {
        receiver,
        method,
        args,
        ..
    } = expr
    else {
        return None;
    };
    if !is_builder_os(receiver) {
        return None;
    }
    let [Expression::StringLiteral { value: argument, .. }] = args.as_slice() else {
        return None;
    };
    match method.as_str() {
        method if method == BuildTargetDslIdent::Env.as_str() => {
            Some(HostEffect::ReadEnv(argument.clone()))
        }
        method if method == BuildTargetDslIdent::ReadFile.as_str() => {
            Some(HostEffect::ReadFile(argument.clone()))
        }
        _ => None,
    }
}

fn is_builder_os(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::MemberAccess { object, field, .. }
            if field == BuildTargetDslIdent::Os.as_str()
                && matches!(
                    object.as_ref(),
                    Expression::Identifier { name, .. }
                        if name == BuildTargetDslIdent::Builder.as_str()
                )
    )
}
