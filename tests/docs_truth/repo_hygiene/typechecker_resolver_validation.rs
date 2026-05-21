use super::*;

mod entry_helpers;
mod focused_modules;
mod local_traversal;
mod support_helpers;
mod support_metadata;

#[test]
fn typechecker_resolver_validation_root_stays_as_router() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation.rs");

    assert!(
        root.lines().count() < 40,
        "typechecker_resolver_validation.rs should only route focused guard modules"
    );
    for module in [
        "mod entry_helpers;",
        "mod focused_modules;",
        "mod local_traversal;",
        "mod support_helpers;",
        "mod support_metadata;",
    ] {
        assert!(
            root.contains(module),
            "typechecker_resolver_validation.rs should include focused module `{module}`"
        );
    }
    let moved_marker = ["src/typechecker", "resolver_validation", "entry_symbols.rs"].join("/");
    assert!(
        !root.contains(&moved_marker),
        "entry traversal checks should live in focused helper modules"
    );
}
