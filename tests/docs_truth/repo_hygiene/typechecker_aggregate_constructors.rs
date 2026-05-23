use super::*;

#[test]
fn typechecker_aggregate_constructors_live_in_focused_helpers() {
    let root = read("src/typechecker/expressions.rs");
    let aggregate = read("src/typechecker/expressions/aggregate_constructors.rs");
    let struct_literal = read("src/typechecker/expressions/struct_literal.rs");
    let enum_variant = read("src/typechecker/expressions/enum_variant.rs");
    let struct_literal_tests = read("src/typechecker/tests/core_semantics/struct_literals.rs");
    let struct_default_tests =
        read("src/typechecker/tests/core_semantics/struct_literal_defaults.rs");
    let enum_variant_tests =
        read("src/typechecker/tests/core_semantics/enum_assignment_and_modules.rs");

    assert!(
        root.contains("mod struct_literal;"),
        "expression checker root should include focused struct literal module"
    );
    assert!(
        root.contains("mod enum_variant;"),
        "expression checker root should include focused enum variant module"
    );
    assert!(
        !aggregate.contains("fn check_struct_literal_expr"),
        "aggregate_constructors.rs should not own struct literal checking"
    );
    assert!(
        !aggregate.contains("fn check_enum_variant_expr"),
        "aggregate_constructors.rs should not own enum variant checking"
    );
    assert!(
        struct_literal.contains("fn check_struct_literal_expr"),
        "struct literal checking should live in focused helper"
    );
    assert!(
        enum_variant.contains("fn check_enum_variant_expr"),
        "enum variant checking should live in focused helper"
    );
    for test_name in [
        "struct_literal_missing_field_is_error",
        "struct_literal_field_type_mismatch_is_error",
    ] {
        assert!(
            struct_literal_tests.contains(&format!("fn {test_name}")),
            "struct literal focused checker should keep behavioral coverage: {test_name}"
        );
    }
    assert!(
        struct_default_tests
            .contains("fn generic_struct_literal_uses_substituted_default_for_omitted_field"),
        "struct literal focused checker should keep generic default behavior coverage"
    );
    for test_name in [
        "enum_variant_unknown_variant_is_error",
        "enum_variant_payload_type_mismatch_is_error",
    ] {
        assert!(
            enum_variant_tests.contains(&format!("fn {test_name}")),
            "enum variant focused checker should keep behavioral coverage: {test_name}"
        );
    }
}
