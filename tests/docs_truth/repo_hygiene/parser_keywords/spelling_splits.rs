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
