use super::super::*;

mod closures;
mod dispatch;
mod generic_methods;
mod leaf_forms;
mod return_flow;

#[test]
fn typechecker_expression_file_size_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions.rs");
    let closures =
        read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions/closures.rs");
    let dispatch =
        read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions/dispatch.rs");
    let generic_methods =
        read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions/generic_methods.rs");
    let leaf_forms =
        read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions/leaf_forms.rs");
    let return_flow =
        read("tests/docs_truth/repo_hygiene/file_size/typechecker_expressions/return_flow.rs");

    assert!(
        root.lines().count() < 80,
        "typechecker_expressions.rs should route focused expression file-size guard modules"
    );
    for module_name in [
        "closures",
        "dispatch",
        "generic_methods",
        "leaf_forms",
        "return_flow",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "typechecker_expressions.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        closures.contains("fn typechecker_closure_expression_checking_lives_in_focused_helper"),
        "closure expression guards should live in closures.rs"
    );
    assert!(
        dispatch.contains("fn typechecker_expression_dispatch_lives_in_focused_helper"),
        "expression dispatch guards should live in dispatch.rs"
    );
    assert!(
        generic_methods
            .contains("fn typechecker_generic_method_resolution_lives_in_focused_helper"),
        "generic method guards should live in generic_methods.rs"
    );
    assert!(
        leaf_forms.contains("fn typechecker_leaf_expression_forms_live_in_focused_helper"),
        "leaf expression guards should live in leaf_forms.rs"
    );
    assert!(
        return_flow.contains("fn typechecker_return_flow_helpers_live_in_focused_helper"),
        "return-flow guards should live in return_flow.rs"
    );
}
