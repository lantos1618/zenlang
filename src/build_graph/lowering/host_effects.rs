use super::{BuildTargetDslIdent, HostEffect, HostEffectResultVariant};
use crate::ast::{Expression, Pattern};

pub(super) fn declared_host_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::Match {
        scrutinee, arms, ..
    } = expr
    else {
        return None;
    };
    let has_fallback = arms
        .iter()
        .any(|arm| host_effect_arm_declares_fallback(&arm.pattern));
    has_fallback.then(|| host_effect(scrutinee)).flatten()
}

fn host_effect_arm_declares_fallback(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Wildcard { .. } | Pattern::Identifier { .. } => true,
        Pattern::Enum { variant, .. } => {
            variant.parse::<HostEffectResultVariant>() == Ok(HostEffectResultVariant::Err)
        }
        _ => false,
    }
}

pub(super) fn host_effect(expr: &Expression) -> Option<HostEffect> {
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
