use super::*;

#[test]
fn typechecker_statement_binding_helpers_live_in_focused_module() {
    let statements = read("src/typechecker/statements.rs");
    let bindings = read("src/typechecker/statements/bindings.rs");

    assert!(
        statements.contains("mod bindings;"),
        "statement checker should load focused binding and assignment helper"
    );
    for helper in [
        "check_var_decl_statement",
        "check_assignment_statement",
        "check_assignment_target",
    ] {
        assert!(
            !statements.contains(&format!("fn {helper}")),
            "statement dispatch should not own binding helper: {helper}"
        );
        assert!(
            bindings.contains(&format!("fn {helper}")),
            "binding and assignment helper should live in bindings.rs: {helper}"
        );
    }
    assert!(
        statements.lines().count() < 120,
        "statements.rs should stay focused on statement dispatch and block scope handling"
    );
}
