use super::*;

mod ast_nodes;
mod casts;
mod smoke_fixtures;
mod stdlib;

#[test]
fn removed_syntax_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/removed_syntax.rs");
    let ast_nodes = read("tests/docs_truth/repo_hygiene/removed_syntax/ast_nodes.rs");
    let casts = read("tests/docs_truth/repo_hygiene/removed_syntax/casts.rs");
    let smoke_fixtures = read("tests/docs_truth/repo_hygiene/removed_syntax/smoke_fixtures.rs");
    let stdlib = read("tests/docs_truth/repo_hygiene/removed_syntax/stdlib.rs");

    assert!(
        root.lines().count() < 80,
        "removed_syntax.rs should route focused removed-syntax guard modules"
    );
    for module_name in ["ast_nodes", "casts", "smoke_fixtures", "stdlib"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "removed_syntax.rs should include focused module: {module_name}"
        );
    }
    assert!(
        ast_nodes.contains("fn source_ast_no_longer_has_return_expression_nodes"),
        "AST-node removal guards should live in ast_nodes.rs"
    );
    assert!(
        casts.contains("fn public_cast_fixture_uses_prefix_cast_syntax"),
        "cast syntax guards should live in casts.rs"
    );
    assert!(
        smoke_fixtures.contains("fn root_smoke_fixtures_do_not_use_removed_or_gated_syntax"),
        "root smoke fixture guards should live in smoke_fixtures.rs"
    );
    assert!(
        stdlib.contains("fn promoted_stdlib_modules_do_not_use_removed_or_gated_syntax"),
        "stdlib removed-syntax guards should live in stdlib.rs"
    );
}
