use super::*;

#[test]
fn delimiters() {
    assert_eq!(
        toks("(){}[]"),
        vec![
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
        ]
    );
}

#[test]
fn separators() {
    assert_eq!(
        toks(",;:"),
        vec![Token::Comma, Token::Semicolon, Token::Colon]
    );
}

#[test]
fn identifiers_and_pub() {
    assert_eq!(
        toks("hello pub world _foo _"),
        vec![
            Token::Identifier("hello".into()),
            Token::Pub,
            Token::Identifier("world".into()),
            Token::Identifier("_foo".into()),
            Token::Identifier("_".into()),
        ]
    );
}

#[test]
fn at_tokens() {
    assert_eq!(
        toks("@std @builtin @this @export"),
        vec![
            Token::AtStd,
            Token::AtBuiltin,
            Token::AtThis,
            Token::AtExport
        ]
    );
}

#[test]
fn unknown_at_token() {
    assert_eq!(toks("@custom"), vec![Token::Identifier("@custom".into())]);
}
