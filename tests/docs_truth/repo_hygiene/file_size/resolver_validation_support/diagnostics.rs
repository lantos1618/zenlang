use super::*;

#[test]
fn resolver_count_diagnostics_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let mixed = read("src/typechecker/resolver_validation_support/absence_diagnostics.rs");
    let counts = read("src/typechecker/resolver_validation_support/count_diagnostics.rs");

    for helper in [
        "CountValidation",
        "value_parameter_resolver_code",
        "field_resolver_code",
        "variant_payload_resolver_code",
    ] {
        assert!(
            !mixed.contains(helper),
            "absence_diagnostics.rs should not own count diagnostic helper: {helper}"
        );
        assert!(
            counts.contains(helper),
            "count diagnostic helper should live in focused helper: {helper}"
        );
    }

    assert!(
        mixed.lines().count() < 190,
        "absence_diagnostics.rs should stay focused on non-count diagnostics"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/count_diagnostics.rs\");"),
        "resolver validation support should include focused count diagnostics"
    );
}
