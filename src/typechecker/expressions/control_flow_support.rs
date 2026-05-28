use crate::ast::expressions::MatchArm;

use super::*;

fn typed_bool_arm(value: bool, body: TypedExpression, span: Span) -> TypedMatchArm {
    TypedMatchArm {
        pattern: TypedPattern::Bool(value),
        body: typed_block_from_expr(body),
        span,
    }
}

impl TypeChecker {
    pub(super) fn check_match_expr(
        &mut self,
        scrutinee: &Expression,
        arms: &[MatchArm],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_scrutinee = self.check_expr(scrutinee)?;
        let mut typed_arms = Vec::new();
        let mut result_type = Type::Void;
        let mut saw_value_arm = false;
        let mut saw_never_arm = false;

        for arm in arms {
            let (typed_body, pattern) = self.with_scope(|checker| {
                checker.bind_pattern(&arm.pattern, &typed_scrutinee.ty);
                let typed_body = checker.check_expr(&arm.body)?;
                let pattern = checker.lower_pattern(&arm.pattern, &typed_scrutinee.ty);
                Ok((typed_body, pattern))
            })?;
            if typed_body.ty == Type::Never {
                saw_never_arm = true;
            } else if !saw_value_arm && typed_body.ty != Type::Void {
                result_type = typed_body.ty.clone();
                saw_value_arm = true;
            }
            typed_arms.push(TypedMatchArm {
                pattern,
                body: typed_block_from_expr(typed_body),
                span: arm.span,
            });
        }
        if !saw_value_arm && saw_never_arm {
            result_type = Type::Never;
        }

        let kind = self.determine_match_kind(&typed_scrutinee.ty, arms);
        if matches!(kind, MatchKind::EnumMatch) {
            self.check_enum_match_patterns(&typed_scrutinee.ty, arms, span);
        } else if matches!(kind, MatchKind::Conditional | MatchKind::ConditionalElse) {
            self.check_bool_match_patterns(arms, result_type != Type::Void, span);
        }

        typed_match_expr(typed_scrutinee, typed_arms, kind, result_type, span)
    }

    pub(super) fn check_if_expr(
        &mut self,
        condition: &Expression,
        then_body: &Expression,
        else_body: &Option<Box<Expression>>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_cond = self.check_expr(condition)?;
        let typed_then = self.check_expr(then_body)?;

        let ty = typed_then.ty.clone();
        let mut arms = vec![typed_bool_arm(true, typed_then, span)];
        if let Some(e) = else_body {
            arms.push(typed_bool_arm(false, self.check_expr(e)?, span));
        }

        let kind = if else_body.is_some() {
            MatchKind::ConditionalElse
        } else {
            MatchKind::Conditional
        };
        typed_match_expr(typed_cond, arms, kind, ty, span)
    }

    pub(super) fn check_loop_expr(
        &mut self,
        body: &Expression,
        control_label: &Option<String>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let kind = control_label
            .as_ref()
            .map_or(MatchKind::WhileLoop, |label| MatchKind::ControlledLoop {
                label: label.clone(),
            });
        typed_match_expr(
            typed_expr(TypedExprKind::BoolLiteral(true), Type::Bool, span),
            vec![typed_bool_arm(true, self.check_expr(body)?, span)],
            kind,
            Type::Void,
            span,
        )
    }
}
