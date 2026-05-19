use super::super::*;

#[test]
fn v1_spec_records_build_graph_static_string_and_backlog_status() {
    let spec = read("docs/V1_SPEC.md");

    for required in [
        "build.zen",
        "zen test build.zen",
        "test target execution",
        "Executable target dependencies compile",
        "dependency cycles",
        "dependency-ordered multi-executable build tests",
        "Deterministic build graph compiles executable and test targets",
        "Accepted Syntax Forms",
        "Test Evidence",
        "StaticString",
        "baked into the program",
        "String literals as baked `StaticString`",
        "interpolation as non-owning `StaticString` views",
        "only literal text is guaranteed",
        "baked program storage",
        "interpolation must not imply allocator-backed",
        "allocator-backed `String`",
        "dynamic_string_type_is_rejected_as_allocator_backed_gate",
        "Source-level `String` use currently reports a gated",
        "Planned Positive Test",
        "Planned Negative Test",
    ] {
        assert!(
            spec.contains(required),
            "docs/V1_SPEC.md is missing build graph or string ownership text: {required}"
        );
    }

    assert!(
        !spec.contains("Deterministic build graph creates one executable target"),
        "docs/V1_SPEC.md still describes the build.zen backlog as single-executable only"
    );
    assert!(
        !spec.contains("String literals and interpolation as `StaticString`"),
        "docs/V1_SPEC.md should not imply interpolation is baked program storage"
    );

    let backlog = spec
        .split("## Required Test Backlog")
        .nth(1)
        .expect("V1 spec should contain required test backlog");
    assert!(
        !backlog.contains("| `build.zen` |"),
        "docs/V1_SPEC.md should not list constrained build.zen execution as only planned backlog"
    );
    assert!(
        !backlog.contains("| AST traversal |"),
        "docs/V1_SPEC.md should not list AST traversal in the no-minimum-proof backlog after AST JSON boundary tests exist"
    );
    assert!(
        !backlog.contains("| Behavior association |"),
        "docs/V1_SPEC.md should not treat explicit behavior association proving-ground coverage as unproven backlog"
    );
    assert!(
        backlog.contains("| Generated/fallback behavior association |"),
        "docs/V1_SPEC.md should keep generated/fallback behavior association in the required backlog"
    );
}
