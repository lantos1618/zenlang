use super::*;

#[test]
fn newlines_are_tokens() {
    assert_eq!(
        toks_all("a\nb\n"),
        vec![
            Token::Identifier("a".into()),
            Token::Newline,
            Token::Identifier("b".into()),
            Token::Newline,
            Token::EOF,
        ]
    );
}

#[test]
fn line_comment() {
    assert_eq!(
        toks_all("a // comment\nb"),
        vec![
            Token::Identifier("a".into()),
            Token::Newline,
            Token::Identifier("b".into()),
            Token::EOF,
        ]
    );
}

#[test]
fn block_comment() {
    assert_eq!(
        toks("a /* comment */ b"),
        vec![Token::Identifier("a".into()), Token::Identifier("b".into())]
    );
}

#[test]
fn nested_block_comments() {
    assert_eq!(
        toks("a /* outer /* inner */ still */ b"),
        vec![Token::Identifier("a".into()), Token::Identifier("b".into())]
    );
}

#[test]
fn empty_source() {
    assert_eq!(toks_all(""), vec![Token::EOF]);
}

#[test]
fn only_whitespace() {
    assert_eq!(toks_all("   \t  "), vec![Token::EOF]);
}

#[test]
fn consecutive_newlines() {
    assert_eq!(
        toks_all("\n\n"),
        vec![Token::Newline, Token::Newline, Token::EOF]
    );
}
