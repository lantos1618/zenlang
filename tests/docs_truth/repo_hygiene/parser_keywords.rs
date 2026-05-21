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
    let module_roots = read("src/parser/atoms/module_roots.rs");

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

    let atoms = read("src/parser/atoms.rs");
    for forbidden in [
        "ParserModuleRoot::AtBuiltin",
        "ParserModuleRoot::AtStd",
        "module_parts",
    ] {
        assert!(
            !atoms.contains(forbidden),
            "parser atom dispatch should not own module-root parsing detail: {forbidden}"
        );
    }

    for helper in [
        "parse_builtin_module_call_expr",
        "parse_std_module_root_expr",
    ] {
        assert!(
            module_roots.contains(&format!("fn {helper}")),
            "module-root atom parsing should live in module_roots.rs: {helper}"
        );
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
                || module_roots.contains(required)
                || read("src/parser/atoms.rs").contains(required)
                || read("src/parser/import_declarations.rs").contains(required),
            "parser module root spelling should live in ParserModuleRoot: {required}"
        );
    }
}

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
