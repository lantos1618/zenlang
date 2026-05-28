use super::*;

pub(super) const POSTFIX_BP: u8 = 23;
pub(super) const PREFIX_BP: u8 = 21;

pub(super) struct InfixOperator {
    pub op: BinaryOp,
    pub l_bp: u8,
    pub r_bp: u8,
}

impl InfixOperator {
    const fn new(op: BinaryOp, l_bp: u8, r_bp: u8) -> Self {
        Self { op, l_bp, r_bp }
    }
}

pub(super) fn infix_operator(token: &Token) -> Option<InfixOperator> {
    match token {
        Token::Or => Some(InfixOperator::new(BinaryOp::Or, 1, 2)),
        Token::And => Some(InfixOperator::new(BinaryOp::And, 3, 4)),
        Token::Pipe => Some(InfixOperator::new(BinaryOp::BitOr, 5, 6)),
        Token::BitXor => Some(InfixOperator::new(BinaryOp::BitXor, 7, 8)),
        Token::BitAnd => Some(InfixOperator::new(BinaryOp::BitAnd, 9, 10)),
        Token::Eq => Some(InfixOperator::new(BinaryOp::Eq, 11, 12)),
        Token::NotEq => Some(InfixOperator::new(BinaryOp::NotEq, 11, 12)),
        Token::Lt => Some(InfixOperator::new(BinaryOp::Lt, 13, 14)),
        Token::Gt => Some(InfixOperator::new(BinaryOp::Gt, 13, 14)),
        Token::LtEq => Some(InfixOperator::new(BinaryOp::LtEq, 13, 14)),
        Token::GtEq => Some(InfixOperator::new(BinaryOp::GtEq, 13, 14)),
        Token::ShiftLeft => Some(InfixOperator::new(BinaryOp::ShiftLeft, 15, 16)),
        Token::ShiftRight => Some(InfixOperator::new(BinaryOp::ShiftRight, 15, 16)),
        Token::Plus => Some(InfixOperator::new(BinaryOp::Add, 17, 18)),
        Token::Minus => Some(InfixOperator::new(BinaryOp::Sub, 17, 18)),
        Token::Star => Some(InfixOperator::new(BinaryOp::Mul, 19, 20)),
        Token::Slash => Some(InfixOperator::new(BinaryOp::Div, 19, 20)),
        Token::Percent => Some(InfixOperator::new(BinaryOp::Mod, 19, 20)),
        _ => None,
    }
}

pub(super) fn first_char_is_upper(s: &str) -> bool {
    s.chars().next().is_some_and(char::is_uppercase)
}
