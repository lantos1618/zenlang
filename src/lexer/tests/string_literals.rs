use super::toks;
use crate::lexer::Token;

#[test]
fn plain_string() {
    assert_eq!(
        toks(r#""hello world""#),
        vec![Token::StringLiteral("hello world".into())]
    );
}

#[test]
fn escape_sequences() {
    assert_eq!(
        toks(r#""\n\t\\\"""#),
        vec![Token::StringLiteral("\n\t\\\"".into())]
    );
}

#[test]
fn hex_escape() {
    assert_eq!(toks(r#""\x41""#), vec![Token::StringLiteral("A".into())]);
}

#[test]
fn null_escape() {
    assert_eq!(toks(r#""\0""#), vec![Token::StringLiteral("\0".into())]);
}

#[test]
fn escaped_dollar_no_interpolation() {
    assert_eq!(
        toks(r#""\${not interpolated}""#),
        vec![Token::StringLiteral("${not interpolated}".into())]
    );
}

#[test]
fn string_interpolation_simple() {
    assert_eq!(
        toks(r#""hello ${name}!""#),
        vec![
            Token::StringChunk("hello ".into()),
            Token::InterpolationStart,
            Token::Identifier("name".into()),
            Token::InterpolationEnd,
            Token::StringChunk("!".into()),
        ]
    );
}

#[test]
fn string_interpolation_expr() {
    assert_eq!(
        toks(r#""result = ${a + b}""#),
        vec![
            Token::StringChunk("result = ".into()),
            Token::InterpolationStart,
            Token::Identifier("a".into()),
            Token::Plus,
            Token::Identifier("b".into()),
            Token::InterpolationEnd,
        ]
    );
}

#[test]
fn string_interpolation_call() {
    assert_eq!(
        toks(r#""${f(x)}""#),
        vec![
            Token::InterpolationStart,
            Token::Identifier("f".into()),
            Token::LParen,
            Token::Identifier("x".into()),
            Token::RParen,
            Token::InterpolationEnd,
        ]
    );
}

#[test]
fn string_interpolation_multiple() {
    assert_eq!(
        toks(r#""${a} and ${b}""#),
        vec![
            Token::InterpolationStart,
            Token::Identifier("a".into()),
            Token::InterpolationEnd,
            Token::StringChunk(" and ".into()),
            Token::InterpolationStart,
            Token::Identifier("b".into()),
            Token::InterpolationEnd,
        ]
    );
}
