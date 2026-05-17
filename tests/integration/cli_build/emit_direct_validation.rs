use std::process::Command;

#[path = "emit_direct_validation/gated_dependencies.rs"]
mod gated_dependencies;
#[path = "emit_direct_validation/graph_only_libraries.rs"]
mod graph_only_libraries;
#[path = "emit_direct_validation/target_selection.rs"]
mod target_selection;

#[test]
fn emit_command_build_zen_ignores_unrelated_gated_test_source_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "missing_test.zen" })
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
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        output.status.success(),
        "zen emit build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let c_source = String::from_utf8_lossy(&output.stdout);
    assert!(
        c_source.contains("int32_t zen_main(void)"),
        "expected target C source, stdout={c_source}"
    );
    assert!(
        !tmp.path().join("build").join("app").exists(),
        "zen emit build.zen should not compile the target binary"
    );
}

#[test]
fn emit_command_build_zen_rejects_unknown_target_dependencies() {
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
    assert_emit_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` depends on unknown target `core`",
    );
}

#[test]
fn emit_command_build_zen_rejects_self_target_dependencies() {
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
    assert_emit_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` cannot depend on itself",
    );
}

#[test]
fn emit_command_build_zen_rejects_cyclic_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library {
        name: "core",
        exports: ["lib.zen"],
        dependencies: ["app"],
    })
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
    assert_emit_command_rejects_dependency_shape(
        tmp.path(),
        "build target dependency cycle includes `app`",
    );
}

#[test]
fn emit_command_build_zen_rejects_duplicate_target_fields() {
    assert_emit_command_rejects_target_metadata(
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
fn emit_command_build_zen_rejects_missing_required_target_fields() {
    assert_emit_command_rejects_target_metadata(
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
fn emit_command_build_zen_rejects_invalid_target_field_types() {
    assert_emit_command_rejects_target_metadata(
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

fn assert_emit_command_rejects_target_metadata(build_source: &str, expected_diagnostic: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        !output.status.success(),
        "zen emit build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_diagnostic),
        "expected target metadata diagnostic `{expected_diagnostic}`, stderr={stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "zen emit build.zen should not write C source after target metadata validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create outputs after target metadata validation fails"
    );
}

fn assert_emit_command_rejects_dependency_shape(
    project_dir: &std::path::Path,
    expected_diagnostic: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(project_dir)
        .output()
        .expect("run zen emit build.zen");

    assert!(
        !output.status.success(),
        "zen emit build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
        "zen emit build.zen should not create outputs after dependency validation fails"
    );
}
