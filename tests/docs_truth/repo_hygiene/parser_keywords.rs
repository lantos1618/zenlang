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

#[test]
fn parser_module_roots_use_owned_root_enum() {
    for path in ["src/parser/atoms.rs", "src/parser/import_declarations.rs"] {
        let source = read(path);
        for forbidden in [
            r#""@builtin".to_string()"#,
            r#""@std".to_string()"#,
            r#"format!("@std.{}""#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should construct parser module roots through ParserModuleRoot, not raw root spelling: {forbidden}"
            );
        }
    }

    let keywords = read("src/parser/keywords.rs");
    for required in [
        "enum ParserModuleRoot",
        "const ALL: &[ParserModuleRoot]",
        "impl FromStr for ParserModuleRoot",
        ".find(|root| root.as_str() == value)",
        "ParserModuleRoot::AtBuiltin.as_str().to_string()",
        "ParserModuleRoot::AtStd.join_module_parts(&module_parts)",
    ] {
        assert!(
            keywords.contains(required)
                || read("src/parser/atoms.rs").contains(required)
                || read("src/parser/import_declarations.rs").contains(required),
            "parser module root spelling should live in ParserModuleRoot: {required}"
        );
    }
}
