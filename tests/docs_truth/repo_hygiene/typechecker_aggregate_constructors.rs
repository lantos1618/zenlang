use super::*;

#[test]
fn typechecker_aggregate_constructors_live_in_focused_helpers() {
    let root = read("src/typechecker/expressions.rs");
    let aggregate = read("src/typechecker/expressions/aggregate_constructors.rs");
    let struct_literal = read("src/typechecker/expressions/struct_literal.rs");
    let enum_variant = read("src/typechecker/expressions/enum_variant.rs");

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
}
