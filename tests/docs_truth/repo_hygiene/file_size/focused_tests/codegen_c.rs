use super::*;

#[test]
fn expression_emission_tests_stay_split_by_expression_form() {
    let root = read("src/codegen/c/tests/expression_emission.rs");
    let literals_and_ops = read("src/codegen/c/tests/expression_emission/literals_and_ops.rs");
    let calls_and_access = read("src/codegen/c/tests/expression_emission/calls_and_access.rs");
    let statements = read("src/codegen/c/tests/expression_emission/statements.rs");

    assert!(
        root.lines().count() < 80,
        "expression_emission.rs should only route focused expression-emission test modules"
    );
    for module in [
        "mod calls_and_access;",
        "mod literals_and_ops;",
        "mod statements;",
    ] {
        assert!(
            root.contains(module),
            "expression_emission.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn emit_function_call"),
        "function-call emission tests should live in calls_and_access.rs"
    );
    assert!(
        literals_and_ops.contains("fn emit_string_literal"),
        "literals_and_ops.rs should cover string literal expression emission"
    );
    assert!(
        calls_and_access.contains("fn emit_struct_literal"),
        "calls_and_access.rs should cover aggregate/access expression emission"
    );
    assert!(
        statements.contains("fn emit_var_decl_mutable_vs_const"),
        "statements.rs should cover statement emission behavior"
    );
}
