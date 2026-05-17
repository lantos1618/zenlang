use std::process::Command;

#[path = "graph_validation_test_command_validation/gated_dependencies.rs"]
mod gated_dependencies;
#[path = "graph_validation_test_command_validation/graph_only_libraries.rs"]
mod graph_only_libraries;

#[test]
fn test_command_build_zen_rejects_graph_without_test_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph test execution requires at least one test target"),
        "expected no test target diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not create outputs for an executable-only graph"
    );
}

#[test]
fn test_command_build_zen_accepts_library_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["lib.zen"] })
    b.add(Test { name: "unit", root: "test.zen", dependencies: ["core"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        output.status.success(),
        "zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        tmp.path().join("build").join("tests").join("unit").exists(),
        "expected test binary output to exist"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test unit passed"),
        "expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_command_build_zen_rejects_unknown_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "test.zen",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target `unit` depends on unknown target `core`",
    );
}

#[test]
fn test_command_build_zen_rejects_self_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "test.zen",
        dependencies: ["unit"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target `unit` cannot depend on itself",
    );
}

#[test]
fn test_command_build_zen_rejects_cyclic_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "unit.zen",
        dependencies: ["integration"],
    })
    b.add(Test {
        name: "integration",
        root: "integration.zen",
        dependencies: ["unit"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target dependency cycle includes `integration`",
    );
}

#[test]
fn test_command_build_zen_rejects_duplicate_test_target_fields() {
    assert_test_command_rejects_target_metadata(
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
fn test_command_build_zen_rejects_missing_required_test_target_fields() {
    assert_test_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
    })
    .Ok(b.config())
}
"#,
        "missing required field `root` or `root_source_file` in `Test` build target",
    );
}

#[test]
fn test_command_build_zen_rejects_invalid_test_target_field_types() {
    assert_test_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: 42,
    })
    .Ok(b.config())
}
"#,
        "field `root` in `Test` build target must be a string",
    );
}

fn assert_test_command_rejects_target_metadata(build_source: &str, expected_diagnostic: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_diagnostic),
        "expected target metadata diagnostic `{expected_diagnostic}`, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not create outputs after target metadata validation fails"
    );
}

fn assert_test_command_rejects_dependency_shape(
    project_dir: &std::path::Path,
    expected_diagnostic: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(project_dir)
        .output()
        .expect("run zen test build.zen");

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected dependency diagnostic `{expected_diagnostic}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !project_dir.join("build").exists(),
        "test command should not create outputs after dependency validation fails"
    );
}

#[test]
fn test_command_build_zen_ignores_unrelated_gated_executable_source_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        output.status.success(),
        "zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        tmp.path().join("build").join("tests").join("unit").exists(),
        "expected test output to exist"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test unit passed"),
        "expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}
