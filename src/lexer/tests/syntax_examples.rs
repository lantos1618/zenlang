use super::*;

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
            Token::IntLiteral(0),
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
            Token::IntLiteral(42),
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
