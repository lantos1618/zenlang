use super::*;

#[test]
fn generic_constructor_diagnostic_tests_stay_split_by_aggregate_kind() {
    let root = read("tests/generic_diagnostics/constructors.rs");
    let structs = read("tests/generic_diagnostics/constructors/structs.rs");
    let enums = read("tests/generic_diagnostics/constructors/enums.rs");

    assert!(
        root.lines().count() < 60,
        "constructors.rs should route focused generic constructor diagnostic modules"
    );
    assert!(
        !root.contains("#[test]"),
        "constructors.rs should not own concrete generic constructor diagnostic tests"
    );
    for module in [
        r#"#[path = "constructors/structs.rs"]"#,
        r#"#[path = "constructors/enums.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "constructors.rs should include focused module path `{module}`"
        );
    }

    assert!(
        structs.contains("fn generic_struct_type_arg_arity_is_error"),
        "structs.rs should cover generic struct constructor arity diagnostics"
    );
    assert!(
        structs.contains("fn nongeneric_struct_constructor_type_args_are_error"),
        "structs.rs should cover non-generic struct constructor type-arg diagnostics"
    );
    assert!(
        enums.contains("fn generic_enum_type_arg_arity_is_error"),
        "enums.rs should cover generic enum constructor arity diagnostics"
    );
    assert!(
        enums.contains("fn nongeneric_enum_constructor_type_args_are_error"),
        "enums.rs should cover non-generic enum constructor type-arg diagnostics"
    );
}
