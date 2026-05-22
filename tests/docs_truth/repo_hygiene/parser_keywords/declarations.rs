use super::*;

#[test]
fn parser_mutability_keywords_use_owned_keyword_enum() {
    for path in [
        "src/parser/declaration_types.rs",
        "src/parser/declarations.rs",
    ] {
        let source = read(path);
        for forbidden in [r#"s == "mut""#] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse mutability through ParserMutabilityKeyword, not raw spelling checks: {forbidden}"
            );
        }
    }

    let keywords = read("src/parser/keywords.rs");
    for required in [
        "enum ParserMutabilityKeyword",
        "const ALL: &[ParserMutabilityKeyword]",
        "impl FromStr for ParserMutabilityKeyword",
        ".find(|keyword| keyword.as_str() == value)",
        "self.consume_mutability_keyword()",
    ] {
        assert!(
            keywords.contains(required)
                || read("src/parser/declaration_types.rs").contains(required)
                || read("src/parser/declarations.rs").contains(required),
            "parser mutability spelling should live in ParserMutabilityKeyword: {required}"
        );
    }
}

#[test]
fn parser_behavior_declaration_keyword_uses_owned_keyword_enum() {
    for path in [
        "src/parser/declarations.rs",
        "src/parser/behavior_declarations.rs",
    ] {
        let source = read(path);
        for forbidden in [
            r#"colon_is_followed_by_identifier("behavior")"#,
            r#"keyword != "behavior""#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse behavior declarations through ParserBehaviorKeyword, not raw spelling checks: {forbidden}"
            );
        }
    }

    let keywords = read("src/parser/keywords.rs");
    for required in [
        "enum ParserBehaviorKeyword",
        "const ALL: &[ParserBehaviorKeyword]",
        "impl FromStr for ParserBehaviorKeyword",
        ".find(|keyword| keyword.as_str() == value)",
        "keyword.parse::<ParserBehaviorKeyword>()",
        "ParserBehaviorKeyword::Behavior.as_str()",
    ] {
        assert!(
            keywords.contains(required)
                || read("src/parser/declarations.rs").contains(required)
                || read("src/parser/behavior_declarations.rs").contains(required),
            "parser behavior declaration spelling should live in ParserBehaviorKeyword: {required}"
        );
    }
}
