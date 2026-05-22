use super::*;

#[test]
fn typechecker_gated_methods_use_owned_action_enum() {
    let root = read("src/typechecker/expressions.rs");
    let source = read("src/typechecker/expressions/method_call_support.rs");
    let gated_methods = read("src/typechecker/expressions/gated_methods.rs");

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
        !source.contains("enum GatedMethod") && !source.contains("const ALL: &[GatedMethod]"),
        "method call resolution should not own gated method enum details"
    );
    assert!(
        gated_methods.contains("enum GatedMethod"),
        "gated_methods.rs should own the gated method enum"
    );
    assert!(
        gated_methods.contains("const ALL: &[GatedMethod]"),
        "gated_methods.rs should keep an enum-owned static table"
    );
    assert!(
        gated_methods.contains("GatedMethod::ALL")
            && gated_methods.contains(".iter()")
            && gated_methods.contains(".copied()")
            && gated_methods.contains(".find(|method| method.as_str() == value)"),
        "typechecker gated method parsing should use the enum-owned static table"
    );
    assert!(
        root.contains("mod gated_methods;"),
        "expression checker root should include the focused gated methods module"
    );
    assert!(
        source.lines().count() < 220,
        "method_call_support.rs should stay focused on method and UFC resolution"
    );
}
