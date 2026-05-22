use super::*;

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
    let keyword_module_roots = read("src/parser/keywords/module_roots.rs");
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
                || keyword_module_roots.contains(required)
                || module_roots.contains(required)
                || read("src/parser/atoms.rs").contains(required)
                || read("src/parser/import_declarations.rs").contains(required),
            "parser module root spelling should live in ParserModuleRoot: {required}"
        );
    }
}

#[test]
fn parser_module_root_spelling_lives_in_focused_keyword_helper() {
    let keywords = read("src/parser/keywords.rs");
    let module_roots = read("src/parser/keywords/module_roots.rs");

    assert!(
        !keywords.contains("enum ParserModuleRoot"),
        "parser keyword root should not own module-root spelling"
    );
    assert!(
        keywords.contains("mod module_roots;")
            && keywords.contains("pub(super) use module_roots::ParserModuleRoot;"),
        "parser keyword root should load and re-export the focused module-root helper"
    );

    for required in [
        "enum ParserModuleRoot",
        "const ALL: &[ParserModuleRoot]",
        "impl FromStr for ParserModuleRoot",
        ".find(|root| root.as_str() == value)",
        "fn join_module_parts",
    ] {
        assert!(
            module_roots.contains(required),
            "parser module-root spelling should live in focused keyword helper: {required}"
        );
    }
}
