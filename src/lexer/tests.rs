use super::*;

mod number_literals;
mod string_literals;

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
fn zen_function_def() {
    assert_eq!(
        toks("add = (a: i32, b: i32) i32 {"),
        vec![
            Token::Identifier("add".into()),
            Token::Assign,
            Token::LParen,
            Token::Identifier("a".into()),
            Token::Colon,
            Token::Identifier("i32".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::Identifier("i32".into()),
            Token::RParen,
            Token::Identifier("i32".into()),
            Token::LBrace,
        ]
    );
}

#[test]
fn zen_import() {
    assert_eq!(
        toks("{ io } = std"),
        vec![
            Token::LBrace,
            Token::Identifier("io".into()),
            Token::RBrace,
            Token::Assign,
            Token::Identifier("std".into()),
        ]
    );
}

#[test]
fn zen_declare_assign() {
    assert_eq!(
        toks("i ::= 0"),
        vec![
            Token::Identifier("i".into()),
            Token::DeclareAssign,
            Token::IntLiteral(0)
        ]
    );
}

#[test]
fn zen_const_assign() {
    assert_eq!(
        toks("x := 42"),
        vec![
            Token::Identifier("x".into()),
            Token::ConstAssign,
            Token::IntLiteral(42)
        ]
    );
}

#[test]
fn zen_ufc_chain() {
    assert_eq!(
        toks("5.double().add_ten()"),
        vec![
            Token::IntLiteral(5),
            Token::Dot,
            Token::Identifier("double".into()),
            Token::LParen,
            Token::RParen,
            Token::Dot,
            Token::Identifier("add_ten".into()),
            Token::LParen,
            Token::RParen,
        ]
    );
}

#[test]
fn zen_pattern_match() {
    assert_eq!(
        toks("x ?\n    | true { 1 }\n    | false { 0 }"),
        vec![
            Token::Identifier("x".into()),
            Token::Question,
            Token::Pipe,
            Token::Identifier("true".into()),
            Token::LBrace,
            Token::IntLiteral(1),
            Token::RBrace,
            Token::Pipe,
            Token::Identifier("false".into()),
            Token::LBrace,
            Token::IntLiteral(0),
            Token::RBrace,
        ]
    );
}

#[test]
fn method_call_on_int() {
    assert_eq!(
        toks("5.double()"),
        vec![
            Token::IntLiteral(5),
            Token::Dot,
            Token::Identifier("double".into()),
            Token::LParen,
            Token::RParen,
        ]
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
