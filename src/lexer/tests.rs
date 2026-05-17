use super::*;

mod number_literals;
mod string_literals;
mod syntax_examples;

/// Tokenise and return just token variants, filtering out Newline/EOF.
fn toks(src: &str) -> Vec<Token> {
    tokenize(src, 0)
        .unwrap()
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| !matches!(t, Token::Newline | Token::EOF))
        .collect()
}

/// Tokenise and return ALL tokens including Newline/EOF.
fn toks_all(src: &str) -> Vec<Token> {
    tokenize(src, 0)
        .unwrap()
        .into_iter()
        .map(|(t, _)| t)
        .collect()
}

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
fn spans_basic() {
    let tokens = tokenize("ab cd", 1).unwrap();
    assert_eq!(tokens[0].1, Span::new(1, 0, 2));
    assert_eq!(tokens[1].1, Span::new(1, 3, 5));
}

#[test]
fn spans_multichar_operator() {
    let tokens = tokenize("::=", 0).unwrap();
    assert_eq!(tokens[0].0, Token::DeclareAssign);
    assert_eq!(tokens[0].1, Span::new(0, 0, 3));
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
