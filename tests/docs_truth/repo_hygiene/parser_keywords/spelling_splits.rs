use super::*;

#[test]
fn parser_keyword_spelling_impls_live_in_focused_helpers() {
    let root = read("src/parser/keywords.rs");

    for module in [
        "behavior",
        "module_roots",
        "mutability",
        "pattern",
        "prefix",
        "this_methods",
    ] {
        assert!(
            root.contains(&format!("mod {module};")),
            "parser keyword root should include focused spelling module: {module}"
        );
    }

    for (path, enum_name) in [
        ("src/parser/keywords/behavior.rs", "ParserBehaviorKeyword"),
        ("src/parser/keywords/module_roots.rs", "ParserModuleRoot"),
        (
            "src/parser/keywords/mutability.rs",
            "ParserMutabilityKeyword",
        ),
        ("src/parser/keywords/pattern.rs", "ParserPatternKeyword"),
        ("src/parser/keywords/prefix.rs", "ParserPrefixKeyword"),
        ("src/parser/keywords/this_methods.rs", "ParserThisMethod"),
    ] {
        let source = read(path);
        assert!(
            !root.contains(&format!("impl {enum_name}")),
            "parser keyword root should not own spelling impl for {enum_name}"
        );
        assert!(
            !root.contains(&format!("impl FromStr for {enum_name}")),
            "parser keyword root should not own parsing impl for {enum_name}"
        );
        assert!(
            source.contains(&format!("impl {enum_name}")),
            "focused parser keyword helper should own spelling impl for {enum_name}"
        );
        assert!(
            source.contains(&format!("impl FromStr for {enum_name}")),
            "focused parser keyword helper should own parsing impl for {enum_name}"
        );
    }

    assert!(
        root.lines().count() < 80,
        "parser keyword root should stay focused on keyword enum definitions"
    );
}

#[test]
fn parser_expression_keyword_guards_live_in_focused_module() {
    let root = read("tests/docs_truth/repo_hygiene/parser_keywords.rs");
    let expression_keywords =
        read("tests/docs_truth/repo_hygiene/parser_keywords/expression_keywords.rs");

    for test_name in [
        "parser_prefix_keywords_use_owned_keyword_enum",
        "parser_pattern_keywords_use_owned_keyword_enum",
        "parser_this_methods_use_owned_method_enum",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "parser_keywords.rs should not own expression keyword guard: {test_name}"
        );
        assert!(
            expression_keywords.contains(&format!("fn {test_name}")),
            "expression keyword guard should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 150,
        "parser_keywords.rs should stay focused on module root, declaration keyword, and shared keyword source guards"
    );
    assert!(
        root.contains("mod expression_keywords;"),
        "parser_keywords.rs should include focused expression keyword guards"
    );
}
