use super::super::*;

#[test]
fn lexer_operator_spelling_tables_have_one_owner() {
    let tokens = read("src/lexer/tokens.rs");

    for required in [
        "MULTI_CHAR_OPERATORS",
        "SINGLE_CHAR_TOKENS",
        "fn from_single_char",
    ] {
        assert!(
            tokens.contains(required),
            "lexer token spellings should be owned by Token: {required}"
        );
    }
    assert!(
        tokens.contains("Self::SINGLE_CHAR_TOKENS")
            && tokens.contains(".find(|(spelling, _)| *spelling == ch)"),
        "single-character token lookup should use Token::SINGLE_CHAR_TOKENS instead of a second hand-written match"
    );

    for path in ["src/lexer/scan.rs", "src/lexer/string_interpolation.rs"] {
        let source = read(path);
        for forbidden in [
            r#"("::=", Token::DeclareAssign"#,
            r#"("=>", Token::FatArrow"#,
            "'+' => Token::Plus",
            "'{' => Token::LBrace",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should use Token spelling helpers instead of owning token spelling tables: {forbidden}"
            );
        }
        assert!(
            source.contains("lex_non_string_token")
                || source.contains("lex_multi_char_operator")
                || source.contains("lex_single_char_token"),
            "{path} should use shared lexer token spelling helpers"
        );
    }
}

#[test]
fn lexer_keyword_and_at_token_spellings_have_one_owner() {
    let tokens = read("src/lexer/tokens.rs");
    let scan = read("src/lexer/scan.rs");

    for required in [
        "KEYWORDS",
        "AT_TOKENS",
        "fn from_keyword",
        "fn from_at_name",
    ] {
        assert!(
            tokens.contains(required),
            "lexer keyword and @ token spellings should be owned by Token: {required}"
        );
    }
    for required in ["Token::from_keyword(&word)", "Token::from_at_name(&word)"] {
        assert!(
            scan.contains(required),
            "lexer scanner should delegate keyword/@ token lookup to Token: {required}"
        );
    }
    for forbidden in [
        r#""pub" => Token::Pub"#,
        "crate::root_spelling::STD_ROOT => Token::AtStd",
        "crate::root_spelling::BUILTIN_ROOT_NAME => Token::AtBuiltin",
        r#""this" => Token::AtThis"#,
        r#""export" => Token::AtExport"#,
    ] {
        assert!(
            !scan.contains(forbidden),
            "lexer scanner should not own keyword/@ token spelling: {forbidden}"
        );
    }
}
