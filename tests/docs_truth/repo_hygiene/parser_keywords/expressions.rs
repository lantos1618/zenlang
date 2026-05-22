use super::*;

#[test]
fn parser_prefix_keywords_use_owned_keyword_enum() {
    let atoms = read("src/parser/atoms.rs");
    let keywords = read("src/parser/keywords.rs");

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
        "impl FromStr for ParserPrefixKeyword",
        ".find(|keyword| keyword.as_str() == value)",
        "name.parse::<ParserPrefixKeyword>()",
    ] {
        assert!(
            atoms.contains(required) || keywords.contains(required),
            "parser prefix keyword spelling should live in ParserPrefixKeyword: {required}"
        );
    }
}

#[test]
fn parser_pattern_keywords_use_owned_keyword_enum() {
    let patterns = read("src/parser/patterns.rs");
    let keywords = read("src/parser/keywords.rs");

    for forbidden in [r#"name == "true""#, r#"name == "false""#, r#"name == "_""#] {
        assert!(
            !patterns.contains(forbidden),
            "parser pattern keyword dispatch should use ParserPatternKeyword, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum ParserPatternKeyword",
        "const ALL: &[ParserPatternKeyword]",
        "impl FromStr for ParserPatternKeyword",
        ".find(|keyword| keyword.as_str() == value)",
        "name.parse::<ParserPatternKeyword>()",
    ] {
        assert!(
            patterns.contains(required) || keywords.contains(required),
            "parser pattern keyword spelling should live in ParserPatternKeyword: {required}"
        );
    }
}

#[test]
fn parser_this_methods_use_owned_method_enum() {
    let atoms = read("src/parser/atoms.rs");
    let keywords = read("src/parser/keywords.rs");

    for forbidden in [r#"method == "defer""#] {
        assert!(
            !atoms.contains(forbidden),
            "parser @this method dispatch should use ParserThisMethod, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum ParserThisMethod",
        "const ALL: &[ParserThisMethod]",
        "impl FromStr for ParserThisMethod",
        ".find(|method| method.as_str() == value)",
        "method.parse::<ParserThisMethod>()",
    ] {
        assert!(
            atoms.contains(required) || keywords.contains(required),
            "parser @this method spelling should live in ParserThisMethod: {required}"
        );
    }
}
