use super::*;

#[test]
fn struct_literal_default_tests_live_in_focused_helper() {
    let struct_literals = read("src/typechecker/tests/core_semantics/struct_literals.rs");
    let defaults = read("src/typechecker/tests/core_semantics/struct_literal_defaults.rs");
    let module = read("src/typechecker/tests/core_semantics.rs");

    assert!(
        struct_literals.lines().count() < 180,
        "struct_literals.rs should stay focused on struct literal error cases"
    );
    assert!(
        !struct_literals.contains("struct_literal_uses_default_for_omitted_field"),
        "struct literal default tests should live in struct_literal_defaults.rs"
    );
    assert!(
        defaults.contains("struct_literal_uses_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover defaulted field omission"
    );
    assert!(
        defaults.contains("generic_struct_literal_uses_substituted_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover generic default substitution"
    );
    assert!(
        module.contains("mod struct_literal_defaults;"),
        "core_semantics.rs should include the focused struct_literal_defaults module"
    );
}

#[test]
fn type_helper_tests_stay_split_by_semantic_surface() {
    let root = read("src/typechecker/tests/core_semantics/type_helpers.rs");
    let compatibility = read("src/typechecker/tests/core_semantics/type_helpers/compatibility.rs");
    let literal_coercion =
        read("src/typechecker/tests/core_semantics/type_helpers/literal_coercion.rs");
    let resolution = read("src/typechecker/tests/core_semantics/type_helpers/resolution.rs");
    let substitution = read("src/typechecker/tests/core_semantics/type_helpers/substitution.rs");

    assert!(
        root.lines().count() < 80,
        "type_helpers.rs should only route focused type-helper tests"
    );
    for module in [
        "mod compatibility;",
        "mod literal_coercion;",
        "mod resolution;",
        "mod substitution;",
    ] {
        assert!(
            root.contains(module),
            "type_helpers.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn types_compatible_basics"),
        "type compatibility tests should live in compatibility.rs"
    );
    assert!(
        compatibility.contains("fn static_string_literal_does_not_implicitly_allocate_string"),
        "compatibility.rs should cover StaticString/String allocation boundaries"
    );
    assert!(
        literal_coercion.contains("fn literal_coercion_in_var_decl"),
        "literal_coercion.rs should cover declaration literal coercion"
    );
    assert!(
        resolution.contains("fn infer_type_args_basic"),
        "resolution.rs should cover generic type-argument inference"
    );
    assert!(
        substitution
            .contains("fn substitute_type_preserves_function_type_arguments_in_nested_generics"),
        "substitution.rs should cover nested generic function-type substitution"
    );
}
