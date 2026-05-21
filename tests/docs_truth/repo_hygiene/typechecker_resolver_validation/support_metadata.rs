use super::*;

#[test]
fn typechecker_resolver_expected_value_symbols_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let monolith = read("src/typechecker/resolver_validation_support/expected_symbols.rs");
    let focused = read("src/typechecker/resolver_validation_support/expected_value_symbols.rs");

    for helper in [
        "ExpectedValueSignature",
        "ExpectedValueSymbol",
        "ExpectedParameter",
        "ExpectedReturnMetadata",
        "ValueParameterValidation",
        "ReturnValidation",
    ] {
        assert!(
            !monolith.contains(&format!("struct {helper}")),
            "expected_symbols.rs should not own value-symbol helper: {helper}"
        );
        assert!(
            focused.contains(&format!("struct {helper}")),
            "expected value-symbol helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation_support/expected_value_symbols.rs\");"),
        "resolver validation support should include focused expected value-symbol helpers"
    );
}

#[test]
fn typechecker_resolver_local_scope_support_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let field_variant = read("src/typechecker/resolver_validation_support/field_variant_scope.rs");
    let local_scope = read("src/typechecker/resolver_validation_support/local_scope.rs");

    for helper in ["ResolverScopeCursor", "ResolverLocalScope"] {
        assert!(
            !field_variant.contains(&format!("struct {helper}")),
            "field_variant_scope.rs should not own resolver local scope helper: {helper}"
        );
        assert!(
            local_scope.contains(&format!("struct {helper}")),
            "resolver local scope helper should live in focused helper: {helper}"
        );
    }

    assert!(
        field_variant.lines().count() < 240,
        "field/variant metadata support should stay focused on metadata helpers"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/local_scope.rs\");"),
        "resolver validation support should include focused local-scope helper"
    );
}

#[test]
fn typechecker_resolver_type_parameter_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let constructors =
        read("src/typechecker/resolver_validation_support/type_info_constructors.rs");
    let type_params =
        read("src/typechecker/resolver_validation_support/resolver_type_parameters.rs");

    for helper in [
        "type_param_bounds_from_resolver_refs",
        "resolver_type_param_bounds",
        "resolver_type_param_names",
        "resolver_type_parameter_metadata",
    ] {
        assert!(
            !constructors.contains(&format!("fn {helper}")),
            "type_info_constructors.rs should not own resolver type-parameter helper: {helper}"
        );
        assert!(
            type_params.contains(&format!("fn {helper}")),
            "resolver type-parameter helper should live in focused helper: {helper}"
        );
    }

    assert!(
        constructors.lines().count() < 240,
        "type info constructors should stay focused on constructing environment records"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/resolver_type_parameters.rs\");"),
        "resolver validation support should include focused resolver type-parameter helpers"
    );
}
