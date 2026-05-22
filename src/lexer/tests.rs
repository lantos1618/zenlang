use super::*;

mod core_tokens;
mod number_literals;
mod operators;
mod spans;
mod string_literals;
mod syntax_examples;
mod trivia;

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
