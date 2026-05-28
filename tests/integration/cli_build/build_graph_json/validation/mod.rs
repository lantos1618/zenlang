use super::super::support::{
    assert_zen_failure_contains, build_graph_source, run_zen_in, write_file,
};
mod dependencies;
mod deterministic_body;
mod unsupported_targets;

#[test]
fn emit_json_build_graph_rejects_target_field_errors() {
    for (target_add, expected) in [
        (
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        output_dir: "build/app/",
    })"#,
            "unknown field `output_dir` in `Executable` build target",
        ),
        (
            r#"    b.add(Test { name: "unit", root: "test.zen", out_dir: "build/tests/" })"#,
            "unknown field `out_dir` in `Test` build target",
        ),
        (
            r#"    b.add(Library { name: "core", exports: ["src/lib.zen"], output_dir: "build/lib/" })"#,
            "unknown field `output_dir` in `Library` build target",
        ),
        (
            r#"    b.add(Library {
        name: "core",
        name: "utils",
        exports: ["src/lib.zen"],
    })"#,
            "duplicate field `name` in `Library` build target",
        ),
        (
            r#"    b.add(Library { name: "core" })"#,
            "missing required field `exports` in `Library` build target",
        ),
        (
            r#"    b.add(Library { name: "core", exports: "src/lib.zen" })"#,
            "field `exports` in `Library` build target must be an array of strings",
        ),
        (
            r#"    b.add(Library { name: "core", exports: [] })"#,
            "field `exports` in `Library` build target must contain at least one source",
        ),
    ] {
        assert_emit_json_build_graph_error_contains(&[target_add], expected);
    }
}

#[test]
fn emit_json_build_graph_rejects_executable_target_metadata_errors() {
    use super::super::support::EXECUTABLE_TARGET_METADATA_CASES;

    for &(target, diagnostic) in EXECUTABLE_TARGET_METADATA_CASES {
        assert_emit_json_build_graph_error_contains(&[target], diagnostic);
    }
}

pub(super) fn assert_emit_json_build_graph_error_contains(targets: &[&str], expected: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", &build_graph_source(targets));
    let args = ["emit-json", "build-graph", "build.zen"];
    let output = run_zen_in(&tmp, &args);

    assert_zen_failure_contains(&args, &output, expected);
}
