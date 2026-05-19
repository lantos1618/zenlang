use super::*;

#[test]
fn typechecker_state_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let state = read("src/typechecker/state.rs");

    for helper in ["TypeChecker", "Default"] {
        assert!(
            !root.contains(&format!("impl {helper} for TypeChecker"))
                && !root.contains(&format!("pub struct {helper}")),
            "typechecker root should not own state helper: {helper}"
        );
    }
    assert!(
        !root.contains("pub fn new() -> Self"),
        "typechecker root should not own TypeChecker::new"
    );
    assert!(
        state.contains("pub struct TypeChecker"),
        "TypeChecker state should live in focused helper"
    );
    assert!(
        state.contains("impl Default for TypeChecker"),
        "TypeChecker default impl should live in focused helper"
    );
    assert!(
        state.contains("pub fn new() -> Self"),
        "TypeChecker constructor should live in focused helper"
    );
    assert!(
        root.contains("include!(\"state.rs\");"),
        "typechecker root should include focused state helper"
    );
}
