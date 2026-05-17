use super::*;

/// Infix operator binding powers (left, right).
pub(super) fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Or => Some((1, 2)),
        Token::And => Some((3, 4)),
        Token::Pipe => Some((5, 6)),
        Token::BitXor => Some((7, 8)),
        Token::BitAnd => Some((9, 10)),
        Token::Eq | Token::NotEq => Some((11, 12)),
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Some((13, 14)),
        Token::ShiftLeft | Token::ShiftRight => Some((15, 16)),
        Token::Plus | Token::Minus => Some((17, 18)),
        Token::Star | Token::Slash | Token::Percent => Some((19, 20)),
        _ => None,
    }
}

/// Postfix binding power for `.`, `[`, `(`.
pub(super) fn postfix_bp() -> (u8, u8) {
    (23, 24)
}

/// Prefix binding power for `-`, `!`, `~`.
pub(super) fn prefix_bp() -> u8 {
    21
}

pub(super) fn first_char_is_upper(s: &str) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}
