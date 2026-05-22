use super::*;

#[test]
fn resolver_local_validation_tests_stay_split_by_local_surface() {
    let root = read("src/typechecker/tests/resolver_locals.rs");
    let absence = read("src/typechecker/tests/resolver_locals/absence.rs");
    let body_locals = read("src/typechecker/tests/resolver_locals/body_locals.rs");
    let expression_body_locals =
        read("src/typechecker/tests/resolver_locals/body_locals/expression_bodies.rs");
    let parameters = read("src/typechecker/tests/resolver_locals/parameters.rs");
    let scope_metadata = read("src/typechecker/tests/resolver_locals/scope_metadata.rs");
    let var_decls = read("src/typechecker/tests/resolver_locals/var_decls.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_locals.rs should only route focused resolver-local tests"
    );
    for module in [
        "mod absence;",
        "mod body_locals;",
        "mod parameters;",
        "mod scope_metadata;",
        "mod var_decls;",
    ] {
        assert!(
            root.contains(module),
            "resolver_locals.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "fn check_program_with_symbols_requires_resolver_parameter_locals",
        "fn check_program_with_symbols_requires_resolver_var_decl_locals",
        "fn check_program_with_symbols_validates_resolver_local_visibility_and_source",
    ] {
        assert!(
            !root.contains(test_name),
            "concrete resolver-local test `{test_name}` should live in a focused child module"
        );
    }
    assert!(
        parameters.contains("fn check_program_with_symbols_requires_resolver_parameter_locals"),
        "parameters.rs should cover required resolver parameter locals"
    );
    assert!(
        parameters.contains(
            "fn check_program_with_symbols_validates_resolver_parameter_local_mutability"
        ),
        "parameters.rs should cover parameter local mutability metadata"
    );
    assert!(
        var_decls.contains("fn check_program_with_symbols_requires_resolver_var_decl_locals"),
        "var_decls.rs should cover required resolver var-declaration locals"
    );
    assert!(
        var_decls
            .contains("fn check_program_with_symbols_validates_resolver_var_decl_local_mutability"),
        "var_decls.rs should cover var-declaration local mutability metadata"
    );
    assert!(
        scope_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_local_visibility_and_source"
        ),
        "scope_metadata.rs should cover resolver local visibility and source metadata"
    );
    assert!(
        scope_metadata.contains("fn check_program_with_symbols_rejects_extra_resolver_locals"),
        "scope_metadata.rs should cover extra resolver local rejection"
    );
    assert!(
        scope_metadata
            .contains("fn check_program_with_symbols_validates_resolver_local_mutability_by_scope"),
        "scope_metadata.rs should cover scope-specific resolver local mutability"
    );
    assert!(
        absence.contains(
            "fn check_program_with_symbols_validates_resolver_local_absent_type_metadata"
        ),
        "absence.rs should keep resolver local absence metadata tests"
    );
    assert!(
        !body_locals.contains("fn check_program_with_symbols_requires_resolver_closure_locals"),
        "body_locals.rs should route concrete body-local traversal tests"
    );
    assert!(
        expression_body_locals
            .contains("fn check_program_with_symbols_requires_resolver_closure_locals"),
        "expression_bodies.rs should keep closure body-local traversal tests"
    );
}

#[test]
fn resolver_body_local_tests_stay_split_by_body_surface() {
    let root = read("src/typechecker/tests/resolver_locals/body_locals.rs");
    let default_bodies =
        read("src/typechecker/tests/resolver_locals/body_locals/default_bodies.rs");
    let expression_bodies =
        read("src/typechecker/tests/resolver_locals/body_locals/expression_bodies.rs");
    let pattern_bodies =
        read("src/typechecker/tests/resolver_locals/body_locals/pattern_bodies.rs");

    assert!(
        root.lines().count() < 80,
        "body_locals.rs should only route focused resolver body-local tests"
    );
    for module in [
        "mod default_bodies;",
        "mod expression_bodies;",
        "mod pattern_bodies;",
    ] {
        assert!(
            root.contains(module),
            "body_locals.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_requires_resolver_pattern_locals",
        "check_program_with_symbols_requires_resolver_top_level_expr_locals",
        "check_program_with_symbols_requires_resolver_struct_field_default_locals",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "body_locals.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        pattern_bodies.contains("fn check_program_with_symbols_requires_resolver_pattern_locals"),
        "pattern_bodies.rs should cover pattern-bound resolver locals"
    );
    assert!(
        expression_bodies
            .contains("fn check_program_with_symbols_requires_resolver_top_level_expr_locals")
            && expression_bodies
                .contains("fn check_program_with_symbols_requires_resolver_closure_locals")
            && expression_bodies.contains(
                "fn check_program_with_symbols_validates_resolver_closure_parameter_mutability"
            ),
        "expression_bodies.rs should cover top-level and closure body locals"
    );
    assert!(
        default_bodies.contains(
            "fn check_program_with_symbols_requires_resolver_struct_field_default_locals"
        ) && default_bodies
            .contains("fn check_program_with_symbols_requires_resolver_behavior_default_locals"),
        "default_bodies.rs should cover resolver locals inside default bodies"
    );
}
