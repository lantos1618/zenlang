use super::*;

#[test]
fn typechecker_aggregate_constructors_live_in_focused_helpers() {
    let root = read("src/typechecker/expressions.rs");
    let aggregate = read("src/typechecker/expressions/aggregate_constructors.rs");
    let struct_literal = read("src/typechecker/expressions/struct_literal.rs");
    let struct_type_args = read("src/typechecker/expressions/struct_literal/type_args.rs");
    let enum_variant = read("src/typechecker/expressions/enum_variant.rs");
    let enum_type_args = read("src/typechecker/expressions/enum_variant/type_args.rs");

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
        struct_literal.contains("mod type_args;"),
        "struct literal helper should include focused generic type-argument support"
    );
    for helper in [
        "resolve_struct_literal_type_args",
        "diagnose_struct_literal_type_arg_arity",
        "generic_struct_default_substitutions",
    ] {
        assert!(
            !struct_literal.contains(&format!("fn {helper}")),
            "struct_literal.rs should not own generic constructor helper: {helper}"
        );
        assert!(
            struct_type_args.contains(&format!("fn {helper}")),
            "generic struct constructor helper should live in type_args.rs: {helper}"
        );
    }
    assert!(
        struct_literal.lines().count() < 150,
        "struct_literal.rs should stay focused on field checking and default insertion"
    );
    assert!(
        enum_variant.contains("fn check_enum_variant_expr"),
        "enum variant checking should live in focused helper"
    );
    assert!(
        enum_variant.contains("mod type_args;"),
        "enum variant helper should include focused generic type-argument support"
    );
    for helper in [
        "resolve_enum_variant_type_args",
        "diagnose_enum_variant_type_arg_arity",
    ] {
        assert!(
            !enum_variant.contains(&format!("fn {helper}")),
            "enum_variant.rs should not own generic constructor helper: {helper}"
        );
        assert!(
            enum_type_args.contains(&format!("fn {helper}")),
            "generic enum constructor helper should live in type_args.rs: {helper}"
        );
    }
    assert!(
        enum_variant.lines().count() < 130,
        "enum_variant.rs should stay focused on payload validation and typed expression assembly"
    );
}
