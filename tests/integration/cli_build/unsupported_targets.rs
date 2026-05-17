use std::process::Command;

#[test]
fn build_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build", "build.zen"], "Package");
}

#[test]
fn build_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build", "build.zen"], "Link");
}

#[test]
fn direct_file_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build.zen"], "Package");
}

#[test]
fn direct_file_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build.zen"], "Link");
}

#[test]
fn check_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["check", "build.zen"], "Package");
}

#[test]
fn check_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["check", "build.zen"], "Link");
}

#[test]
fn test_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["test", "build.zen"], "Package");
}

#[test]
fn test_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["test", "build.zen"], "Link");
}

#[test]
fn emit_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["emit", "build.zen"], "Package");
}

#[test]
fn emit_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["emit", "build.zen"], "Link");
}

#[test]
fn build_graph_command_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(
        &["build-graph", "build.zen"],
        "Package",
    );
}

#[test]
fn build_graph_command_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build-graph", "build.zen"], "Link");
}

#[test]
fn build_zen_commands_reject_package_fields() {
    assert_build_zen_commands_reject_gated_target_field("packages", r#"["std"]"#);
}

#[test]
fn build_zen_commands_reject_link_fields() {
    assert_build_zen_commands_reject_gated_target_field("link", r#"["m"]"#);
}

fn assert_build_zen_command_rejects_unsupported_target_kind(args: &[&str], target_kind: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})
    .Ok(b.config())
}}
"#
        ),
    )
    .expect("write build.zen");

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
        String::from_utf8_lossy(&output.stderr).contains(&format!(
            "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
        )),
        "expected unsupported target diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should reject unsupported target kinds before creating build outputs"
    );
}

fn assert_build_zen_commands_reject_gated_target_field(field: &str, value: &str) {
    for args in [
        &["build", "build.zen"][..],
        &["build.zen"][..],
        &["check", "build.zen"][..],
        &["test", "build.zen"][..],
        &["emit", "build.zen"][..],
        &["build-graph", "build.zen"][..],
    ] {
        assert_build_zen_command_rejects_gated_target_field(args, field, value);
    }
}

fn assert_build_zen_command_rejects_gated_target_field(args: &[&str], field: &str, value: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})
    .Ok(b.config())
}}
"#
        ),
    )
    .expect("write build.zen");

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
        String::from_utf8_lossy(&output.stderr).contains(&format!(
            "unsupported field `{field}` in `Executable` build target; package/link semantics are gated"
        )),
        "expected gated field diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should reject gated target fields before creating build outputs"
    );
}
