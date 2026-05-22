use super::super::*;

#[test]
fn resolver_phase2_impl_behavior_method_metadata_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/impls.rs");
    let method_metadata = read("tests/resolver_phase2/impls/behavior_method_metadata.rs");

    for test_name in [
        "resolver_records_behavior_impl_methods_as_value_symbols",
        "resolver_records_behavior_impl_function_type_methods",
        "resolver_records_behavior_impl_method_body_locals",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_phase2 impls.rs should not own behavior method metadata test: {test_name}"
        );
        assert!(
            method_metadata.contains(&format!("fn {test_name}")),
            "behavior method metadata test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "resolver_phase2 impls.rs should stay focused on impl-edge and plain impl checks"
    );
    assert!(
        root.contains("mod behavior_method_metadata;"),
        "resolver_phase2 impls.rs should include focused behavior method metadata tests"
    );
}

#[test]
fn resolver_phase2_struct_metadata_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/struct_metadata.rs");
    let declarations = read("tests/resolver_phase2/struct_metadata/declarations.rs");
    let defaults = read("tests/resolver_phase2/struct_metadata/defaults.rs");
    let literals = read("tests/resolver_phase2/struct_metadata/literals.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 struct_metadata.rs should only route focused struct metadata modules"
    );
    for module in ["mod declarations;", "mod defaults;", "mod literals;"] {
        assert!(
            root.contains(module),
            "resolver_phase2 struct_metadata.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_rejects_unknown_struct_literal_fields"),
        "struct literal validation tests should live in literals.rs"
    );
    assert!(
        declarations.contains("fn resolver_records_struct_field_types"),
        "declarations.rs should cover struct field metadata"
    );
    assert!(
        defaults.contains("fn resolver_records_struct_field_default_locals"),
        "defaults.rs should cover struct field default local metadata"
    );
    assert!(
        literals.contains("fn resolver_rejects_missing_struct_literal_fields"),
        "literals.rs should cover struct literal field validation"
    );
}

#[test]
fn resolver_phase2_enum_metadata_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/enum_metadata.rs");
    let generic_payloads = read("tests/resolver_phase2/enum_metadata/generic_payloads.rs");
    let payloads = read("tests/resolver_phase2/enum_metadata/payloads.rs");
    let variant_shape = read("tests/resolver_phase2/enum_metadata/variant_shape.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 enum_metadata.rs should only route focused enum metadata modules"
    );
    for module in [
        r#"#[path = "enum_metadata/generic_payloads.rs"]"#,
        r#"#[path = "enum_metadata/payloads.rs"]"#,
        r#"#[path = "enum_metadata/variant_shape.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "resolver_phase2 enum_metadata.rs should include focused module path `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_records_enum_variant_payload_counts"),
        "enum payload metadata tests should live in payloads.rs"
    );
    assert!(
        generic_payloads.contains("fn resolver_records_generic_enum_variant_payload_types"),
        "generic_payloads.rs should cover generic enum payload metadata"
    );
    assert!(
        payloads.contains("fn resolver_records_enum_function_type_payloads"),
        "payloads.rs should cover concrete enum payload metadata"
    );
    assert!(
        variant_shape.contains("fn resolver_rejects_duplicate_variant_names_in_same_enum"),
        "variant_shape.rs should cover enum variant naming diagnostics"
    );
}

#[test]
fn resolver_phase2_expr_local_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/expr_locals.rs");
    let closures = read("tests/resolver_phase2/expr_locals/closures.rs");
    let local_symbols = read("tests/resolver_phase2/expr_locals/local_symbols.rs");
    let patterns = read("tests/resolver_phase2/expr_locals/patterns.rs");
    let value_references = read("tests/resolver_phase2/expr_locals/value_references.rs");
    let variant_expressions = read("tests/resolver_phase2/expr_locals/variant_expressions.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 expr_locals.rs should only route focused expression-local modules"
    );
    for module in [
        r#"#[path = "expr_locals/closures.rs"]"#,
        r#"#[path = "expr_locals/local_symbols.rs"]"#,
        r#"#[path = "expr_locals/patterns.rs"]"#,
        r#"#[path = "expr_locals/value_references.rs"]"#,
        r#"#[path = "expr_locals/variant_expressions.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "resolver_phase2 expr_locals.rs should include focused module path `{module}`"
        );
    }

    for test_name in [
        "resolver_records_parameter_and_local_symbols",
        "resolver_records_top_level_expr_locals",
        "resolver_records_same_name_locals_in_distinct_scopes",
        "resolver_rejects_duplicate_bindings_in_same_scope",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "local symbol resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            local_symbols.contains(&format!("fn {test_name}")),
            "local_symbols.rs should keep local symbol resolver test: {test_name}"
        );
    }

    for test_name in [
        "resolver_rejects_unknown_unqualified_function_calls",
        "resolver_rejects_unknown_local_identifier_references",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "value reference resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            value_references.contains(&format!("fn {test_name}")),
            "value_references.rs should keep value reference resolver test: {test_name}"
        );
    }

    for test_name in [
        "resolver_rejects_unknown_enum_variant_expressions",
        "resolver_rejects_missing_enum_variant_payload_expressions",
        "resolver_rejects_unexpected_enum_variant_payload_expressions",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "variant expression resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            variant_expressions.contains(&format!("fn {test_name}")),
            "variant_expressions.rs should keep variant expression resolver test: {test_name}"
        );
    }

    assert!(
        closures.contains("fn resolver_records_closure_locals"),
        "closures.rs should keep closure local resolver tests"
    );
    assert!(
        patterns.contains("fn resolver_records_pattern_locals"),
        "patterns.rs should keep pattern local resolver tests"
    );
}
