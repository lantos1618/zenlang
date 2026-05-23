use crate::ast::expressions::{BinaryOp, UnaryOp};

pub(super) fn c_binary_op(op: BinaryOp) -> &'static str {
    op.symbol()
}

pub(super) fn c_unary_op(op: UnaryOp) -> &'static str {
    op.symbol()
}
