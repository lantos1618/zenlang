use super::super::*;

mod codegen_c;
mod typechecker;

#[test]
fn focused_tests_root_stays_as_router() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/focused_tests.rs");

    assert!(
        root.lines().count() < 40,
        "focused_tests.rs should only route focused file-size guard modules"
    );
    for module in ["mod codegen_c;", "mod typechecker;"] {
        assert!(
            root.contains(module),
            "focused_tests.rs should include focused module `{module}`"
        );
    }
    let moved_path_marker = ["src/typechecker/tests", "resolver_collection"].join("/");
    assert!(
        !root.contains(&moved_path_marker),
        "typechecker-focused guards should live below focused_tests/typechecker/"
    );
}
