use super::*;

#[test]
fn typechecker_resolver_validation_post_pass_lives_in_focused_helper() {
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let post_pass = read("src/typechecker/resolver_validation/post_pass.rs");

    for helper in [
        "validate_no_extra_resolver_declaration_symbols",
        "validate_no_extra_resolver_local_symbols",
        "validate_resolver_behavior_association_lists",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver validation entry traversal should not own post-pass helper: {helper}"
        );
        assert!(
            post_pass.contains(&format!("fn {helper}")),
            "resolver validation post-pass helper should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/post_pass.rs\");"),
        "resolver validation should include focused post-pass validation"
    );
}
