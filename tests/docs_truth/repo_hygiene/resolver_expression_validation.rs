use super::*;

mod calls;
mod constructs;
mod dispatch;
mod traversal;

#[test]
fn resolver_expression_validation_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/resolver_expression_validation.rs");
    let calls = read("tests/docs_truth/repo_hygiene/resolver_expression_validation/calls.rs");
    let constructs =
        read("tests/docs_truth/repo_hygiene/resolver_expression_validation/constructs.rs");
    let dispatch = read("tests/docs_truth/repo_hygiene/resolver_expression_validation/dispatch.rs");
    let traversal =
        read("tests/docs_truth/repo_hygiene/resolver_expression_validation/traversal.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_expression_validation.rs should route focused hygiene guard modules"
    );
    for module_name in ["calls", "constructs", "dispatch", "traversal"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver_expression_validation.rs should include focused module: {module_name}"
        );
    }
    assert!(
        calls.contains("fn resolver_call_expression_validation_lives_in_focused_helper"),
        "call expression guards should live in resolver_expression_validation/calls.rs"
    );
    assert!(
        constructs.contains("fn resolver_aggregate_expression_validation_lives_in_focused_helper"),
        "construct guards should live in resolver_expression_validation/constructs.rs"
    );
    assert!(
        dispatch.contains("fn resolver_expression_dispatch_stays_as_category_router"),
        "dispatch guards should live in resolver_expression_validation/dispatch.rs"
    );
    assert!(
        traversal.contains("fn resolver_expression_traversal_lives_in_focused_helper"),
        "traversal guards should live in resolver_expression_validation/traversal.rs"
    );
}
