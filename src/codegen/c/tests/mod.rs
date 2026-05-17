use super::*;

mod program_generation;
mod type_mapping;

fn dummy() -> crate::error::Span {
    crate::error::Span::dummy()
}

fn texpr(kind: TypedExprKind, ty: Type) -> TypedExpression {
    TypedExpression {
        kind,
        ty,
        span: dummy(),
    }
}

mod expression_emission;
mod helpers;
