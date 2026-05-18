use std::process::Command;

#[path = "validation/dependencies.rs"]
mod dependencies;
#[path = "validation/deterministic_body.rs"]
mod deterministic_body;
#[path = "validation/unsupported_targets.rs"]
mod unsupported_targets;

#[test]
fn emit_json_build_graph_rejects_unknown_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        output_dir: "build/app/",
    })
    .Ok(b.config())
}
"#,
        "unknown field `output_dir` in `Executable` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_unknown_test_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen", out_dir: "build/tests/" })
    .Ok(b.config())
}
"#,
        "unknown field `out_dir` in `Test` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_unknown_library_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["src/lib.zen"], output_dir: "build/lib/" })
    .Ok(b.config())
}
"#,
        "unknown field `output_dir` in `Library` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_missing_required_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
    })
    .Ok(b.config())
}
"#,
        "missing required field `out_dir` in `Executable` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_duplicate_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        name: "tool",
        main: "app.zen",
        out_dir: "build/app/",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Executable` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_invalid_target_field_types() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: 42,
    })
    .Ok(b.config())
}
"#,
        "field `out_dir` in `Executable` build target must be a string",
    );
}

#[test]
fn emit_json_build_graph_rejects_duplicate_library_target_fields() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library {
        name: "core",
        name: "utils",
        exports: ["src/lib.zen"],
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Library` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_missing_library_exports() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core" })
    .Ok(b.config())
}
"#,
        "missing required field `exports` in `Library` build target",
    );
}

#[test]
fn emit_json_build_graph_rejects_invalid_library_exports_type() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: "src/lib.zen" })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must be an array of strings",
    );
}

#[test]
fn emit_json_build_graph_rejects_empty_library_exports() {
    assert_emit_json_build_graph_error_contains(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: [] })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must contain at least one source",
    );
}

fn assert_emit_json_build_graph_error_contains(build_source: &str, expected: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(&build_path, build_source).expect("write build.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", build_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        !output.status.success(),
        "emit-json build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "expected {expected:?}, stderr={stderr}"
    );
}
