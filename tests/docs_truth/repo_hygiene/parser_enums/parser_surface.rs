use super::*;

#[test]
fn parser_type_declaration_suffixes_use_owned_keyword_enum() {
    let source = read("src/parser/declarations.rs");
    let suffix_forms = read("src/parser/declarations/suffix_forms.rs");
    let dispatch_source = format!("{source}\n{suffix_forms}");
    let ast_declarations = read("src/ast/declarations.rs");
    let type_keywords = read("src/ast/declarations/type_keywords.rs");
    let keyword_source = format!("{ast_declarations}\n{type_keywords}");

    for forbidden in [
        r#"method_name == "impl""#,
        r#"method_name == "implements""#,
        r#"method_name == "requires""#,
        r#"method_name == "extends""#,
        r#"method_name == "derive""#,
        r#"matches!(method_name.as_str(), "implements" | "requires" | "extends")"#,
    ] {
        assert!(
            !dispatch_source.contains(forbidden),
            "parser type declaration suffix dispatch should use TypeDeclarationKeyword, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        dispatch_source.contains("TypeDeclarationKeyword"),
        "parser type declaration suffix dispatch should use TypeDeclarationKeyword"
    );

    for forbidden in [
        "value == Self::Impl.as_str()",
        "value == Self::Implements.as_str()",
        "value == Self::Requires.as_str()",
        "value == Self::Extends.as_str()",
    ] {
        assert!(
            !keyword_source.contains(forbidden),
            "TypeDeclarationKeyword parsing should use the enum-owned static table, not raw if-chain spelling checks: {forbidden}"
        );
    }

    for required in [
        "pub const ALL: &[TypeDeclarationKeyword]",
        ".find(|keyword| keyword.as_str() == value)",
    ] {
        assert!(
            keyword_source.contains(required),
            "TypeDeclarationKeyword spelling should parse through its static table: {required}"
        );
    }
    assert!(
        !ast_declarations.contains("pub enum TypeDeclarationKeyword"),
        "declaration AST root should not own parser-facing type declaration keyword spelling"
    );
    assert!(
        ast_declarations.contains("mod type_keywords;")
            && ast_declarations.contains("pub use type_keywords::TypeDeclarationKeyword;"),
        "declaration AST root should re-export the focused type keyword helper"
    );
    assert!(
        ast_declarations.lines().count() < 210,
        "declarations.rs should stay focused on declaration shapes and accessors"
    );
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
