use crate::ast::expressions::MatchArm;

use super::*;

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
            self.push_scope();
            self.bind_pattern(&arm.pattern, &typed_scrutinee.ty);
            let typed_body = self.check_expr(&arm.body)?;
            if typed_body.ty == Type::Never {
                saw_never_arm = true;
            } else if !saw_value_arm && typed_body.ty != Type::Void {
                result_type = typed_body.ty.clone();
                saw_value_arm = true;
            }
            let pattern = self.lower_pattern(&arm.pattern, &typed_scrutinee.ty);
            self.pop_scope();
            typed_arms.push(TypedMatchArm {
                pattern,
                body: TypedBlock {
                    ty: typed_body.ty.clone(),
                    span: typed_body.span,
                    statements: Vec::new(),
                    expr: Some(Box::new(typed_body)),
                },
                span: arm.span,
            });
        }
        if !saw_value_arm && saw_never_arm {
            result_type = Type::Never;
        }

        let kind = self.determine_match_kind(&typed_scrutinee.ty, arms);
        if matches!(kind, MatchKind::EnumMatch) {
            self.check_enum_match_patterns(&typed_scrutinee.ty, arms);
            self.check_match_exhaustiveness(&typed_scrutinee.ty, arms, span);
        } else if matches!(kind, MatchKind::Conditional | MatchKind::ConditionalElse) {
            self.check_bool_match_patterns(arms, result_type != Type::Void, span);
        }

        Ok(TypedExpression {
            kind: TypedExprKind::Match {
                scrutinee: Box::new(typed_scrutinee),
                arms: typed_arms,
                kind,
            },
            ty: result_type,
            span,
        })
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
        let typed_else = match else_body {
            Some(e) => Some(Box::new(self.check_expr(e)?)),
            None => None,
        };

        let ty = typed_then.ty.clone();
        let then_block = TypedBlock {
            ty: typed_then.ty.clone(),
            span: typed_then.span,
            statements: Vec::new(),
            expr: Some(Box::new(typed_then)),
        };
        let else_arm = typed_else.map(|e| TypedMatchArm {
            pattern: TypedPattern::Bool(false),
            body: TypedBlock {
                ty: e.ty.clone(),
                span: e.span,
                statements: Vec::new(),
                expr: Some(e),
            },
            span,
        });

        let mut arms = vec![TypedMatchArm {
            pattern: TypedPattern::Bool(true),
            body: then_block,
            span,
        }];
        if let Some(ea) = else_arm {
            arms.push(ea);
        }

        Ok(TypedExpression {
            kind: TypedExprKind::Match {
                scrutinee: Box::new(typed_cond),
                arms,
                kind: if else_body.is_some() {
                    MatchKind::ConditionalElse
                } else {
                    MatchKind::Conditional
                },
            },
            ty,
            span,
        })
    }

    pub(super) fn check_while_loop_expr(
        &mut self,
        condition: &Expression,
        body: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_cond = self.check_expr(condition)?;
        let typed_body = self.check_expr(body)?;
        let body_block = TypedBlock {
            ty: typed_body.ty.clone(),
            span: typed_body.span,
            statements: Vec::new(),
            expr: Some(Box::new(typed_body)),
        };

        Ok(TypedExpression {
            kind: TypedExprKind::Match {
                scrutinee: Box::new(typed_cond),
                arms: vec![TypedMatchArm {
                    pattern: TypedPattern::Bool(true),
                    body: body_block,
                    span,
                }],
                kind: MatchKind::WhileLoop,
            },
            ty: Type::Void,
            span,
        })
    }

    pub(super) fn check_loop_expr(
        &mut self,
        body: &Expression,
        control_label: &Option<String>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_body = self.check_expr(body)?;
        let body_block = TypedBlock {
            ty: typed_body.ty.clone(),
            span: typed_body.span,
            statements: Vec::new(),
            expr: Some(Box::new(typed_body)),
        };

        Ok(TypedExpression {
            kind: TypedExprKind::Match {
                scrutinee: Box::new(TypedExpression {
                    kind: TypedExprKind::BoolLiteral(true),
                    ty: Type::Bool,
                    span,
                }),
                arms: vec![TypedMatchArm {
                    pattern: TypedPattern::Bool(true),
                    body: body_block,
                    span,
                }],
                kind: control_label
                    .as_ref()
                    .map(|label| MatchKind::ControlledLoop {
                        label: label.clone(),
                    })
                    .unwrap_or(MatchKind::WhileLoop),
            },
            ty: Type::Void,
            span,
        })
    }
}
