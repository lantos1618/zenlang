use super::*;

#[test]
fn lexer_string_interpolation_lives_in_focused_helper() {
    let strings = read("src/lexer/strings.rs");
    let interpolation = read("src/lexer/string_interpolation.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        strings.lines().count() < 160,
        "strings.rs should stay focused on literal string scanning"
    );
    assert!(
        !strings.contains("fn lex_interpolation_body"),
        "string interpolation body scanning should live in string_interpolation.rs"
    );
    assert!(
        interpolation.contains("fn lex_interpolation_body"),
        "string_interpolation.rs should scan interpolation bodies"
    );
    assert!(
        interpolation.contains("fn lex_next_no_skip"),
        "string_interpolation.rs should own no-skip token scanning for interpolation bodies"
    );
    assert!(
        lexer_module.contains("mod string_interpolation;"),
        "lexer module should include the focused string_interpolation helper"
    );
}

#[test]
fn lexer_number_scanning_lives_in_focused_helper() {
    let scan = read("src/lexer/scan.rs");
    let numbers = read("src/lexer/numbers.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        scan.lines().count() < 220,
        "scan.rs should stay focused on token dispatch and small token scanners"
    );
    assert!(
        !scan.contains("fn lex_prefixed_int"),
        "prefixed integer scanning should live in numbers.rs"
    );
    assert!(
        !scan.contains("fn eat_digits"),
        "digit scanning should live in numbers.rs"
    );
    assert!(
        numbers.contains("pub(super) fn lex_number"),
        "numbers.rs should own number token scanning"
    );
    assert!(
        numbers.contains("fn lex_prefixed_int"),
        "numbers.rs should own prefixed integer scanning"
    );
    assert!(
        lexer_module.contains("mod numbers;"),
        "lexer module should include the focused number scanning helper"
    );
}
