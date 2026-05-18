use super::*;

#[test]
fn parser_type_declaration_suffixes_use_owned_keyword_enum() {
    let source = read("src/parser/declarations.rs");

    for forbidden in [
        r#"method_name == "impl""#,
        r#"method_name == "implements""#,
        r#"method_name == "requires""#,
        r#"method_name == "extends""#,
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

#[test]
fn typechecker_gated_methods_use_owned_action_enum() {
    let source = read("src/typechecker/expressions/method_call_support.rs");

    for forbidden in [
        r#""raise" => Some(Self::ResultRaise)"#,
        r#""await" => Some(Self::EffectAwait)"#,
        "value == Self::ResultRaise.as_str()",
        "value == Self::EffectAwait.as_str()",
        "from_method_name",
    ] {
        assert!(
            !source.contains(forbidden),
            "typechecker gated methods should use GatedMethod parsing/display, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        source.contains("method.parse::<GatedMethod>()"),
        "typechecker gated method dispatch should parse through GatedMethod"
    );
}

#[test]
fn cli_emit_json_modes_use_owned_mode_enum() {
    let source = read("src/cli.rs");

    assert!(
        source.contains("enum EmitJsonMode"),
        "emit-json command routing should use an owned EmitJsonMode enum"
    );
    assert!(
        source.contains("mode.parse::<EmitJsonMode>()"),
        "emit-json command routing should parse modes through EmitJsonMode"
    );
    assert!(
        source.contains("EmitJsonMode::usage()"),
        "emit-json usage should be generated from EmitJsonMode"
    );
    assert!(
        !source.contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>"),
        "emit-json usage should not duplicate the mode list as a raw string"
    );
}
