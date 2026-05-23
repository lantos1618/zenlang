use super::*;

mod expression_keywords;
mod spelling_splits;

fn parser_keyword_sources() -> String {
    [
        "src/parser/keywords.rs",
        "src/parser/keywords/behavior.rs",
        "src/parser/keywords/module_roots.rs",
        "src/parser/keywords/mutability.rs",
        "src/parser/keywords/pattern.rs",
        "src/parser/keywords/prefix.rs",
        "src/parser/keywords/this_methods.rs",
    ]
    .into_iter()
    .map(read)
    .collect::<Vec<_>>()
    .join("\n")
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

    let keywords = parser_keyword_sources();
    for required in [
        "enum ParserModuleRoot",
        "const ALL: &[ParserModuleRoot]",
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
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserModuleRoot")
            && uses_static_spelling_parser(&keywords),
        "parser module root spelling should use shared static spelling parsing"
    );
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

    let keywords = parser_keyword_sources();
    for required in [
        "enum ParserMutabilityKeyword",
        "const ALL: &[ParserMutabilityKeyword]",
        "self.consume_mutability_keyword()",
    ] {
        assert!(
            keywords.contains(required)
                || read("src/parser/declaration_types.rs").contains(required)
                || read("src/parser/declarations.rs").contains(required),
            "parser mutability spelling should live in ParserMutabilityKeyword: {required}"
        );
    }
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserMutabilityKeyword")
            && uses_static_spelling_parser(&keywords),
        "parser mutability spelling should use shared static spelling parsing"
    );
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

    let keywords = parser_keyword_sources();
    for required in [
        "enum ParserBehaviorKeyword",
        "const ALL: &[ParserBehaviorKeyword]",
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
    assert!(
        owns_static_spelling_from_str(&keywords, "ParserBehaviorKeyword")
            && uses_static_spelling_parser(&keywords),
        "parser behavior declaration spelling should use shared static spelling parsing"
    );
}
