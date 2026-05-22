use super::*;

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
