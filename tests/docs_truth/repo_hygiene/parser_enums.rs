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
    assert!(
        source.contains("const ALL: &[GatedMethod]"),
        "typechecker gated methods should keep an enum-owned static table"
    );
    assert!(
        source.contains("GatedMethod::ALL")
            && source.contains(".iter()")
            && source.contains(".copied()")
            && source.contains(".find(|method| method.as_str() == value)"),
        "typechecker gated method parsing should use the enum-owned static table"
    );
}

#[test]
fn typechecker_gated_intrinsics_use_owned_name_enum() {
    let gated = read("src/typechecker/gated_intrinsics.rs");
    let calls = read("src/typechecker/expressions/call_support.rs");

    for forbidden in [
        r#"name == "async_enqueue""#,
        r#"name == "async_yield""#,
        r#"name == "raw_allocate""#,
        r#"name == "raw_deallocate""#,
        r#"name == "raw_reallocate""#,
        r#"name == "type_match""#,
        r#"match name"#,
        r#""async_enqueue" =>"#,
        r#""async_yield" =>"#,
        r#""raw_allocate" =>"#,
        r#""raw_deallocate" =>"#,
        r#""raw_reallocate" =>"#,
        r#""type_match" =>"#,
    ] {
        assert!(
            !calls.contains(forbidden),
            "typechecker gated intrinsic dispatch should use GatedIntrinsic, not raw spelling checks: {forbidden}"
        );
    }
    for required in [
        "enum GatedIntrinsic",
        "const ALL: &[GatedIntrinsic]",
        "pub(super) const ASYNC_ENQUEUE: &'static str = \"async_enqueue\"",
        "pub(super) const ASYNC_YIELD: &'static str = \"async_yield\"",
        "pub(super) const RAW_ALLOCATE: &'static str = \"raw_allocate\"",
        "pub(super) const RAW_DEALLOCATE: &'static str = \"raw_deallocate\"",
        "pub(super) const RAW_REALLOCATE: &'static str = \"raw_reallocate\"",
        "pub(super) const TYPE_MATCH: &'static str = \"type_match\"",
        "pub(super) const fn gate_message(self) -> &'static str",
        ".find(|intrinsic| intrinsic.as_str() == name)",
    ] {
        assert!(
            gated.contains(required),
            "gated intrinsic spelling should live in GatedIntrinsic: {required}"
        );
    }
    assert!(
        calls.contains("GatedIntrinsic::from_name(name)") && calls.contains("gated.gate_message()"),
        "function-call checking should route gated intrinsics through GatedIntrinsic"
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
        source.contains("fn gate_message(self) -> Option<&'static str>"),
        "emit-json gated diagnostics should be owned by EmitJsonMode"
    );
    assert!(
        source.contains("mode.gate_message()"),
        "emit-json command routing should read gated diagnostics from EmitJsonMode"
    );
    assert!(
        !source.contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>"),
        "emit-json usage should not duplicate the mode list as a raw string"
    );
}
