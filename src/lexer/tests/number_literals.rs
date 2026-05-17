use super::toks;
use crate::lexer::Token;

#[test]
fn integers() {
    assert_eq!(
        toks("42 0 1_000"),
        vec![
            Token::IntLiteral(42),
            Token::IntLiteral(0),
            Token::IntLiteral(1000)
        ]
    );
}

#[test]
fn floats() {
    assert_eq!(
        toks("3.14 0.0"),
        vec![Token::FloatLiteral(3.14), Token::FloatLiteral(0.0)]
    );
}

#[test]
fn hex_binary_octal() {
    assert_eq!(
        toks("0xFF 0b1010 0o777 0xDE_AD"),
        vec![
            Token::IntLiteral(0xFF),
            Token::IntLiteral(0b1010),
            Token::IntLiteral(0o777),
            Token::IntLiteral(0xDEAD),
        ]
    );
}

#[test]
fn float_vs_range() {
    assert_eq!(toks("3.14"), vec![Token::FloatLiteral(3.14)]);
    assert_eq!(
        toks("3..10"),
        vec![Token::IntLiteral(3), Token::DotDot, Token::IntLiteral(10)]
    );
}
