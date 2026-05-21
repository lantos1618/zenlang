use super::*;

mod focused_modules;
mod support_helpers;

#[test]
fn typechecker_resolver_validation_post_pass_lives_in_focused_helper() {
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let post_pass = read("src/typechecker/resolver_validation/post_pass.rs");

    for helper in [
        "validate_no_extra_resolver_declaration_symbols",
        "validate_no_extra_resolver_local_symbols",
        "validate_resolver_behavior_association_lists",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver validation entry traversal should not own post-pass helper: {helper}"
        );
        assert!(
            post_pass.contains(&format!("fn {helper}")),
            "resolver validation post-pass helper should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/post_pass.rs\");"),
        "resolver validation should include focused post-pass validation"
    );
}

#[test]
fn typechecker_resolver_entry_local_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let locals = read("src/typechecker/resolver_validation/entry_locals.rs");

    for helper in [
        "require_resolver_callable_locals",
        "require_resolver_scoped_expr_locals",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own local helper: {helper}"
        );
        assert!(
            locals.contains(&format!("fn {helper}")),
            "resolver entry local helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 260,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_locals.rs\");"),
        "resolver validation should include focused entry-local helpers"
    );
}

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

#[test]
fn typechecker_resolver_pattern_local_traversal_lives_in_focused_helper() {
    let traversal = read("src/typechecker/resolver_validation/local_traversal.rs");
    let patterns = read("src/typechecker/resolver_validation/pattern_locals.rs");

    for helper in [
        "require_resolver_pattern_expr_locals",
        "require_resolver_pattern_locals",
        "require_resolver_pattern_binding",
    ] {
        assert!(
            !traversal.contains(&format!("fn {helper}")),
            "resolver local traversal should not own pattern-local helper: {helper}"
        );
        assert!(
            patterns.contains(&format!("fn {helper}")),
            "resolver pattern-local traversal should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/pattern_locals.rs\");"),
        "resolver validation should include focused pattern-local traversal"
    );
}

#[test]
fn typechecker_resolver_variant_metadata_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let types = read("src/typechecker/resolver_validation/metadata_types.rs");
    let variants = read("src/typechecker/resolver_validation/metadata_variants.rs");

    for helper in [
        "validate_resolver_variant_names",
        "validate_resolver_variant_payload",
        "validate_resolver_variant_owner_name",
        "validate_resolver_variant_visibility",
        "validate_resolver_variant_absent_other_metadata",
    ] {
        assert!(
            !types.contains(&format!("fn {helper}")),
            "resolver type metadata should not own variant metadata helper: {helper}"
        );
        assert!(
            variants.contains(&format!("fn {helper}")),
            "resolver variant metadata helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation/metadata_variants.rs\");"),
        "resolver validation should include focused variant metadata helper"
    );
}

#[test]
fn typechecker_resolver_absence_diagnostics_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let diagnostics = read("src/typechecker/resolver_validation/metadata_diagnostics.rs");
    let absence = read("src/typechecker/resolver_validation/metadata_absence.rs");

    for helper in [
        "validate_resolver_absent_value_signature_metadata",
        "validate_resolver_absent_type_parameter_metadata",
        "validate_resolver_absent_field_metadata",
        "validate_resolver_absent_variant_metadata",
        "validate_resolver_absent_behavior_association_metadata",
        "validate_resolver_absent_behavior_declaration_metadata",
        "validate_resolver_absent_mutability_metadata",
        "validate_resolver_absent_source_metadata",
    ] {
        assert!(
            !diagnostics.contains(&format!("fn {helper}")),
            "resolver metadata diagnostics should not own absence wrapper: {helper}"
        );
        assert!(
            absence.contains(&format!("fn {helper}")),
            "resolver absence diagnostics should live in focused helper: {helper}"
        );
    }

    assert!(
        diagnostics.lines().count() < 220,
        "resolver metadata diagnostics should stay focused on generic emitters"
    );
    assert!(
        root.contains("include!(\"resolver_validation/metadata_absence.rs\");"),
        "resolver validation should include focused absence diagnostics"
    );
}
