use super::*;

#[test]
fn arithmetic_operators() {
    assert_eq!(
        toks("+ - * / %"),
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent
        ]
    );
}

#[test]
fn comparison_operators() {
    assert_eq!(
        toks("== != < > <= >="),
        vec![
            Token::Eq,
            Token::NotEq,
            Token::Lt,
            Token::Gt,
            Token::LtEq,
            Token::GtEq
        ]
    );
}

#[test]
fn logical_operators() {
    assert_eq!(toks("&& || !"), vec![Token::And, Token::Or, Token::Not]);
}

#[test]
fn bitwise_operators() {
    assert_eq!(
        toks("& ^ ~ << >>"),
        vec![
            Token::BitAnd,
            Token::BitXor,
            Token::Tilde,
            Token::ShiftLeft,
            Token::ShiftRight
        ]
    );
}

#[test]
fn assignment_operators() {
    assert_eq!(
        toks("= := ::="),
        vec![Token::Assign, Token::ConstAssign, Token::DeclareAssign]
    );
}

#[test]
fn dot_operators() {
    assert_eq!(
        toks(". .. ..="),
        vec![Token::Dot, Token::DotDot, Token::DotDotEq]
    );
}

#[test]
fn arrows() {
    assert_eq!(toks("-> =>"), vec![Token::Arrow, Token::FatArrow]);
}

#[test]
fn pipe_and_question() {
    assert_eq!(
        toks("| ? ||"),
        vec![Token::Pipe, Token::Question, Token::Or]
    );
}
