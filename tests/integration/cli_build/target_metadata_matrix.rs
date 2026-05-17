use std::process::Command;

#[test]
fn build_zen_commands_reject_duplicate_library_target_fields() {
    assert_build_zen_commands_reject_library_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library {
        name: "core",
        name: "utils",
        exports: ["lib.zen"],
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Library` build target",
    );
}

#[test]
fn build_zen_commands_reject_missing_library_exports() {
    assert_build_zen_commands_reject_library_target_metadata(
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
fn build_zen_commands_reject_invalid_library_exports_type() {
    assert_build_zen_commands_reject_library_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: "lib.zen" })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must be an array of strings",
    );
}

#[test]
fn build_zen_commands_reject_empty_library_exports() {
    assert_build_zen_commands_reject_library_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: [] })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must contain at least one source",
    );
}

#[test]
fn build_zen_commands_reject_unknown_library_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["lib.zen"], output_dir: "build/lib/" })
    .Ok(b.config())
}
"#,
        "unknown field `output_dir` in `Library` build target",
    );
}

#[test]
fn build_zen_commands_reject_duplicate_executable_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
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
fn build_zen_commands_reject_missing_required_executable_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen" })
    .Ok(b.config())
}
"#,
        "missing required field `out_dir` in `Executable` build target",
    );
}

#[test]
fn build_zen_commands_reject_invalid_executable_target_field_types() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen", out_dir: 42 })
    .Ok(b.config())
}
"#,
        "field `out_dir` in `Executable` build target must be a string",
    );
}

#[test]
fn build_zen_commands_reject_duplicate_test_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        name: "integration",
        root: "test.zen",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Test` build target",
    );
}

#[test]
fn build_zen_commands_reject_missing_required_test_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit" })
    .Ok(b.config())
}
"#,
        "missing required field `root` or `root_source_file` in `Test` build target",
    );
}

#[test]
fn build_zen_commands_reject_invalid_test_target_field_types() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: 42 })
    .Ok(b.config())
}
"#,
        "field `root` in `Test` build target must be a string",
    );
}

#[test]
fn build_zen_commands_reject_unknown_test_target_fields() {
    assert_build_zen_commands_reject_build_graph_metadata(
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
fn build_zen_commands_reject_dynamic_target_adds() {
    assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    enabled = true
    enabled ?
        | true {
            b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
        }
        | false {
            b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
        }
    .Ok(b.config())
}
"#,
        "build targets must be added in the deterministic build graph body",
    );
}

fn assert_build_zen_commands_reject_library_target_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    assert_build_zen_commands_reject_build_graph_metadata(build_source, expected_diagnostic);
}

fn assert_build_zen_commands_reject_build_graph_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    for args in [
        &["build", "build.zen"][..],
        &["build.zen"][..],
        &["check", "build.zen"][..],
        &["test", "build.zen"][..],
        &["emit", "build.zen"][..],
        &["build-graph", "build.zen"][..],
    ] {
        assert_build_zen_command_rejects_library_target_metadata(
            args,
            build_source,
            expected_diagnostic,
        );
    }
}

fn assert_build_zen_command_rejects_library_target_metadata(
    args: &[&str],
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen command");

    assert!(
        !output.status.success(),
        "zen {args:?} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected library target metadata diagnostic `{expected_diagnostic}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should reject library target metadata before creating build outputs"
    );
}
