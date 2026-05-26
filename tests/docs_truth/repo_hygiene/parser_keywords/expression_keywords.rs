use super::*;

#[test]
fn parser_prefix_keywords_use_owned_keyword_enum() {
    let atoms = read("src/parser/atoms.rs");
    let keywords = parser_keyword_sources();

    for forbidden in [
        "match name.as_str()",
        r#""true" =>"#,
        r#""false" =>"#,
        r#""return" =>"#,
        r#""break" =>"#,
        r#""continue" =>"#,
        r#""loop" =>"#,
        r#""cast" =>"#,
    ] {
        assert!(
            !atoms.contains(forbidden),
            "parser prefix keyword dispatch should use ParserPrefixKeyword, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum ParserPrefixKeyword",
        "const ALL: &[ParserPrefixKeyword]",
        "name.parse::<ParserPrefixKeyword>()",
    ] {
        assert!(
            atoms.contains(required) || keywords.contains(required),
            "parser prefix keyword spelling should live in ParserPrefixKeyword: {required}"
        );
    }
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserPrefixKeyword")
            && uses_static_spelling_parser(&keywords),
        "parser prefix keyword spelling should use shared static spelling parsing"
    );
}

#[test]
fn parser_pattern_keywords_use_owned_keyword_enum() {
    let patterns = read("src/parser/patterns.rs");
    let keywords = parser_keyword_sources();

    for forbidden in [r#"name == "true""#, r#"name == "false""#, r#"name == "_""#] {
        assert!(
            !patterns.contains(forbidden),
            "parser pattern keyword dispatch should use ParserPatternKeyword, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum ParserPatternKeyword",
        "const ALL: &[ParserPatternKeyword]",
        "name.parse::<ParserPatternKeyword>()",
    ] {
        assert!(
            patterns.contains(required) || keywords.contains(required),
            "parser pattern keyword spelling should live in ParserPatternKeyword: {required}"
        );
    }
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserPatternKeyword")
            && uses_static_spelling_parser(&keywords),
        "parser pattern keyword spelling should use shared static spelling parsing"
    );
}

#[test]
fn parser_this_methods_use_owned_method_enum() {
    let atoms = read("src/parser/atoms.rs");
    let keywords = parser_keyword_sources();

    for forbidden in [r#"method == "defer""#] {
        assert!(
            !atoms.contains(forbidden),
            "parser @this method dispatch should use ParserThisMethod, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum ParserThisMethod",
        "const ALL: &[ParserThisMethod]",
        "method.parse::<ParserThisMethod>()",
    ] {
        assert!(
            atoms.contains(required) || keywords.contains(required),
            "parser @this method spelling should live in ParserThisMethod: {required}"
        );
    }
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserThisMethod")
            && uses_static_spelling_parser(&keywords),
        "parser @this method spelling should use shared static spelling parsing"
    );
}
