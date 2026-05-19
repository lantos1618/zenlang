use crate::ast::{Declaration, Expression, MatchArm, Program, Statement};

#[path = "lowering/dsl.rs"]
mod dsl;
#[path = "lowering/target_fields.rs"]
mod target_fields;
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
        lowering.collect_expr(build_body, BuildTargetAddContext::StaticGraphBody);
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuildTargetAddContext {
    StaticGraphBody,
    DynamicExpression,
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

    fn collect_expr(&mut self, expr: &Expression, add_context: BuildTargetAddContext) {
        if let Some(effect) = host_effect(expr) {
            self.used_host_effects.push(effect);
        }
        if let Some(effect) = declared_host_effect(expr) {
            self.declared_host_effects.push(effect);
        }
        if add_context == BuildTargetAddContext::DynamicExpression
            && is_builder_add_call(expr)
            && self.error.is_none()
        {
            self.error = Some(BuildGraphError::UnsupportedBuildScript(
                "build targets must be added in the deterministic build graph body".to_string(),
            ));
        }
        match build_target_from_builder_add(expr) {
            Ok(Some(target)) => {
                if add_context == BuildTargetAddContext::StaticGraphBody {
                    self.targets.push(target);
                }
            }
            Ok(None) => {}
            Err(error) => {
                if self.error.is_none() {
                    self.error = Some(error);
                }
            }
        }

        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.collect_expr(left, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(right, BuildTargetAddContext::DynamicExpression);
            }
            Expression::UnaryOp { operand, .. } => {
                self.collect_expr(operand, BuildTargetAddContext::DynamicExpression)
            }
            Expression::FunctionCall { args, .. }
            | Expression::ArrayLiteral { elements: args, .. } => {
                for arg in args {
                    self.collect_expr(arg, BuildTargetAddContext::DynamicExpression);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_expr(receiver, add_context);
                for arg in args {
                    self.collect_expr(arg, BuildTargetAddContext::DynamicExpression);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.collect_expr(object, BuildTargetAddContext::DynamicExpression)
            }
            Expression::IndexAccess { object, index, .. } => {
                self.collect_expr(object, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(index, BuildTargetAddContext::DynamicExpression);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, field) in fields {
                    self.collect_expr(field, BuildTargetAddContext::DynamicExpression);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.collect_expr(payload, BuildTargetAddContext::DynamicExpression);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.collect_expr(scrutinee, BuildTargetAddContext::DynamicExpression);
                for MatchArm { guard, body, .. } in arms {
                    if let Some(guard) = guard {
                        self.collect_expr(guard, BuildTargetAddContext::DynamicExpression);
                    }
                    self.collect_expr(body, BuildTargetAddContext::DynamicExpression);
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
                self.collect_expr(condition, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(body, BuildTargetAddContext::DynamicExpression);
                if let Expression::If {
                    else_body: Some(else_body),
                    ..
                } = expr
                {
                    self.collect_expr(else_body, BuildTargetAddContext::DynamicExpression);
                }
            }
            Expression::Loop { body, .. } => {
                self.collect_expr(body, BuildTargetAddContext::DynamicExpression)
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
            Expression::Closure { body, .. } => {
                self.collect_expr(body, BuildTargetAddContext::DynamicExpression)
            }
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.collect_expr(expr, BuildTargetAddContext::DynamicExpression)
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let crate::ast::StringPart::Expr(expr) = part {
                        self.collect_expr(expr, BuildTargetAddContext::DynamicExpression);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.collect_expr(start, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(end, BuildTargetAddContext::DynamicExpression);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::LoopControl { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn collect_statement(&mut self, statement: &Statement, add_context: BuildTargetAddContext) {
        match statement {
            Statement::Expression { expr: value, .. } => {
                self.collect_expr(value, add_context);
            }
            Statement::VarDecl { value, .. } => {
                self.collect_expr(value, BuildTargetAddContext::DynamicExpression);
            }
            Statement::Assignment { target, value, .. } => {
                self.collect_expr(target, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(value, BuildTargetAddContext::DynamicExpression);
            }
            Statement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.collect_statement(stmt, add_context);
                }
            }
        }
    }
}

fn is_builder_add_call(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::MethodCall { receiver, method, .. }
            if method == BuildTargetDslIdent::Add.as_str()
                && matches!(
                    receiver.as_ref(),
                    Expression::Identifier { name, .. }
                        if name == BuildTargetDslIdent::Builder.as_str()
                )
    )
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
    match method.parse::<BuildTargetDslIdent>() {
        Ok(BuildTargetDslIdent::Env) => Some(HostEffect::ReadEnv(argument.clone())),
        Ok(BuildTargetDslIdent::ReadFile) => Some(HostEffect::ReadFile(argument.clone())),
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
