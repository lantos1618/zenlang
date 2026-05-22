use super::super::*;

mod expr_locals;
mod type_metadata;
mod value_metadata;

#[test]
fn resolver_phase2_file_size_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/resolver_phase2.rs");
    let type_metadata =
        read("tests/docs_truth/repo_hygiene/file_size/resolver_phase2/type_metadata.rs");
    let expr_locals =
        read("tests/docs_truth/repo_hygiene/file_size/resolver_phase2/expr_locals.rs");
    let value_metadata =
        read("tests/docs_truth/repo_hygiene/file_size/resolver_phase2/value_metadata.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_phase2.rs should route focused file-size guard modules"
    );
    for module_name in ["type_metadata", "expr_locals", "value_metadata"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver_phase2.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        type_metadata.contains("fn resolver_phase2_enum_metadata_tests_live_in_focused_modules"),
        "type metadata guards should live in resolver_phase2/type_metadata.rs"
    );
    assert!(
        expr_locals.contains("fn resolver_phase2_expr_local_tests_live_in_focused_modules"),
        "expression-local guards should live in resolver_phase2/expr_locals.rs"
    );
    assert!(
        value_metadata.contains("fn resolver_phase2_value_metadata_tests_live_in_focused_modules"),
        "value metadata guards should live in resolver_phase2/value_metadata.rs"
    );
}
