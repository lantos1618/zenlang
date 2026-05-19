use super::*;

#[test]
fn typechecker_binary_op_checking_lives_in_focused_helper() {
    let root = read("src/typechecker/resolve.rs");
    let binary_ops = read("src/typechecker/resolve_binary_ops.rs");
    let module = read("src/typechecker/mod.rs");

    for helper in [
        "check_binary_op",
        "check_arithmetic_binary_op",
        "check_logical_binary_op",
        "check_bitwise_binary_op",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "type resolution root should not own binary operator helper: {helper}"
        );
        assert!(
            binary_ops.contains(&format!("fn {helper}")),
            "binary operator checking should live in focused helper: {helper}"
        );
    }

    assert!(
        module.contains("mod resolve_binary_ops;"),
        "typechecker root should include focused binary operator checking module"
    );
}
