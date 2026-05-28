use super::dsl::{BUILDER_IDENT, ENV_METHOD, OS_FIELD, READ_FILE_METHOD};
use super::HostEffect;
use crate::ast::{Expression, Pattern};

pub(super) fn declared_host_effect(expr: &Expression) -> Option<HostEffect> {
    let Expression::Match {
        scrutinee, arms, ..
    } = expr
    else {
        return None;
    };
    let has_fallback = arms.iter().any(|arm| {
        matches!(
            &arm.pattern,
            Pattern::Wildcard { .. } | Pattern::Identifier { .. }
        ) || matches!(&arm.pattern, Pattern::Enum { variant, .. } if variant == "Err")
    });
    has_fallback.then(|| host_effect(scrutinee)).flatten()
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
    if !matches!(
        receiver.as_ref(),
        Expression::MemberAccess { object, field, .. }
            if field == OS_FIELD
                && matches!(
                    object.as_ref(),
                    Expression::Identifier { name, .. }
                        if name == BUILDER_IDENT
                )
    ) {
        return None;
    }
    let [Expression::StringLiteral { value: argument, .. }] = args.as_slice() else {
        return None;
    };
    match method.as_str() {
        ENV_METHOD => Some(HostEffect::ReadEnv(argument.clone())),
        READ_FILE_METHOD => Some(HostEffect::ReadFile(argument.clone())),
        _ => None,
    }
}
