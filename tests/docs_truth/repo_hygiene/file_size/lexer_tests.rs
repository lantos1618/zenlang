use super::super::*;

#[test]
fn lexer_tests_stay_split_by_token_surface() {
    let root = read("src/lexer/tests.rs");
    let core_tokens = read("src/lexer/tests/core_tokens.rs");
    let operators = read("src/lexer/tests/operators.rs");
    let spans = read("src/lexer/tests/spans.rs");
    let trivia = read("src/lexer/tests/trivia.rs");

    assert!(
        root.lines().count() < 80,
        "lexer tests root should only route focused token test modules and shared helpers"
    );
    for module in [
        "mod core_tokens;",
        "mod number_literals;",
        "mod operators;",
        "mod spans;",
        "mod string_literals;",
        "mod syntax_examples;",
        "mod trivia;",
    ] {
        assert!(
            root.contains(module),
            "lexer tests root should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn delimiters") && !root.contains("fn arithmetic_operators"),
        "lexer tests root should not own concrete token-family tests"
    );

    assert!(
        core_tokens.contains("fn delimiters")
            && core_tokens.contains("fn identifiers_and_pub")
            && core_tokens.contains("fn at_tokens"),
        "core_tokens.rs should cover delimiters, identifiers, and @ tokens"
    );
    assert!(
        operators.contains("fn arithmetic_operators")
            && operators.contains("fn comparison_operators")
            && operators.contains("fn pipe_and_question"),
        "operators.rs should cover operator token families"
    );
    assert!(
        spans.contains("fn spans_basic") && spans.contains("fn spans_multichar_operator"),
        "spans.rs should cover lexer span expectations"
    );
    assert!(
        trivia.contains("fn line_comment")
            && trivia.contains("fn block_comment")
            && trivia.contains("fn consecutive_newlines"),
        "trivia.rs should cover whitespace, newlines, and comments"
    );
}
