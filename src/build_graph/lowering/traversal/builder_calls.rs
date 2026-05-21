use crate::ast::Expression;

use super::super::dsl::BuildTargetDslIdent;

pub(super) fn is_builder_add_call(expr: &Expression) -> bool {
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
