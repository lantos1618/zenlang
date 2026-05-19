use super::*;

#[test]
fn parser_type_declaration_suffixes_use_owned_keyword_enum() {
    let source = read("src/parser/declarations.rs");
    let ast_declarations = read("src/ast/declarations.rs");

    for forbidden in [
        r#"method_name == "impl""#,
        r#"method_name == "implements""#,
        r#"method_name == "requires""#,
        r#"method_name == "extends""#,
        r#"method_name == "derive""#,
        r#"matches!(method_name.as_str(), "implements" | "requires" | "extends")"#,
    ] {
        assert!(
            !source.contains(forbidden),
            "parser type declaration suffix dispatch should use TypeDeclarationKeyword, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        source.contains("TypeDeclarationKeyword"),
        "parser type declaration suffix dispatch should use TypeDeclarationKeyword"
    );

    for forbidden in [
        "value == Self::Impl.as_str()",
        "value == Self::Implements.as_str()",
        "value == Self::Requires.as_str()",
        "value == Self::Extends.as_str()",
    ] {
        assert!(
            !ast_declarations.contains(forbidden),
            "TypeDeclarationKeyword parsing should use the enum-owned static table, not raw if-chain spelling checks: {forbidden}"
        );
    }

    for required in [
        "pub const ALL: &[TypeDeclarationKeyword]",
        ".find(|keyword| keyword.as_str() == value)",
    ] {
        assert!(
            ast_declarations.contains(required),
            "TypeDeclarationKeyword spelling should parse through its static table: {required}"
        );
    }
}

#[test]
fn parser_loop_control_calls_use_owned_action_enum() {
    for path in [
        "src/parser/expressions.rs",
        "src/parser/expressions/suffixes.rs",
    ] {
        let source = read(path);
        for forbidden in [
            r#"name.as_str() == "done""#,
            r#"name.as_str() == "next""#,
            r#"match name.as_str()"#,
            r#""done" => Expression::LoopControl"#,
            r#""next" => Expression::LoopControl"#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse loop control calls through LoopControlAction, not raw spelling checks: {forbidden}"
            );
        }
    }

    let suffixes = read("src/parser/expressions/suffixes.rs");
    assert!(
        suffixes.contains("name.parse::<LoopControlAction>()"),
        "parser loop-control suffix handling should parse through LoopControlAction"
    );
}
