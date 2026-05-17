use std::process::Command;

#[test]
fn check_command_validates_build_zen_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write main.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        output.status.success(),
        "zen check build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 build targets"),
        "expected build graph check summary, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_command_build_zen_accepts_library_only_graph_validation() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["lib.zen"] })
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        output.status.success(),
        "zen check build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 build targets"),
        "expected build graph check summary, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen check build.zen should validate library-only graphs without creating build outputs"
    );
}

#[test]
fn check_command_build_zen_rejects_unknown_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_check_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` depends on unknown target `core`",
    );
}

#[test]
fn check_command_build_zen_rejects_self_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["app"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_check_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` cannot depend on itself",
    );
}

#[test]
fn check_command_build_zen_rejects_cyclic_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })
    b.add(Executable {
        name: "tool",
        main: "tool.zen",
        out_dir: "build/tool/",
        dependencies: ["app"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_check_command_rejects_dependency_shape(
        tmp.path(),
        "build target dependency cycle includes `app`",
    );
}

#[test]
fn check_command_build_zen_rejects_duplicate_library_target_fields() {
    assert_check_command_rejects_library_target_metadata(
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
fn check_command_build_zen_rejects_missing_library_exports() {
    assert_check_command_rejects_library_target_metadata(
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
fn check_command_build_zen_rejects_invalid_library_exports_type() {
    assert_check_command_rejects_library_target_metadata(
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
fn check_command_build_zen_rejects_empty_library_exports() {
    assert_check_command_rejects_library_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: [] })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must contain at least one source",
    );
}

fn assert_check_command_rejects_library_target_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
        "zen check build.zen should not create outputs after library target metadata validation fails"
    );
}

fn assert_check_command_rejects_dependency_shape(
    project_dir: &std::path::Path,
    expected_diagnostic: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(project_dir)
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
        "zen check build.zen should not create outputs after dependency validation fails"
    );
}

#[test]
fn check_command_build_zen_typechecks_target_sources() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write main.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected target source type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_build_zen_rejects_missing_executable_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "myapp", main: "missing.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `myapp` source not found: missing.zen"),
        "expected missing source diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_build_zen_rejects_missing_test_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "missing_test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `unit` source not found: missing_test.zen"),
        "expected missing source diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_build_zen_rejects_missing_library_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["missing_lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `core` source not found: missing_lib.zen"),
        "expected missing source diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
