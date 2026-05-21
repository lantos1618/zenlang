use super::*;

pub(super) fn push_typed_global(
    globals: &mut Vec<TypedGlobal>,
    typed_expr: TypedExpression,
    span: Span,
) {
    if let TypedExprKind::Block(block) = &typed_expr.kind {
        if block.expr.is_none() && block.statements.len() == 1 {
            if let TypedStatementKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            } = &block.statements[0].kind
            {
                globals.push(TypedGlobal {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: value.clone(),
                    mutable: *mutable,
                    span,
                });
                return;
            }
        }
    }

    globals.push(TypedGlobal {
        name: "__top_level__".into(),
        ty: typed_expr.ty.clone(),
        value: typed_expr,
        mutable: false,
        span,
    });
}
