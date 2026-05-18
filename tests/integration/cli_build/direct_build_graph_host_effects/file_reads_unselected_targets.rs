use std::process::Command;

#[test]
fn direct_file_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets() {
    assert_direct_file_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_direct_file_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_direct_file_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_undeclared_file_read_effects_before_unselected_targets() {
    assert_direct_file_command_rejects_file_read_before_unselected_targets(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
        "undeclared",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_file_read_without_fallback_before_unselected_targets() {
    assert_direct_file_command_rejects_file_read_before_unselected_targets(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
        "missing-fallback",
    );
}

fn assert_direct_file_command_accepts_declared_file_read_effects_with_unselected_targets(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "app.zen", out_dir: "build/app/" }})
    b.add(Test {{ name: "unit", root: "missing_unit.zen" }})
    b.add(Library {{ name: "core", exports: ["lib.zen"] }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("build.targets"), "app\n").expect("write manifest");
    write_zero_main(tmp.path().join("app.zen"));
    write_library_source(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bin_path = tmp.path().join("build").join("app").join("app");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );
}

fn assert_direct_file_command_rejects_file_read_before_unselected_targets(
    build_source: &str,
    diagnostic_case: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    write_zero_main(tmp.path().join("app.zen"));
    write_library_source(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        !output.status.success(),
        "zen build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undeclared host effect: read file `build.targets`"),
        "expected {diagnostic_case} file read diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("missing_unit.zen"),
        "host-effect validation should run before unrelated test source handling, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "direct build.zen command should reject file effects before selected target execution"
    );
}

fn write_zero_main(path: impl AsRef<std::path::Path>) {
    std::fs::write(
        path,
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write main source");
}

fn write_library_source(tmp: &tempfile::TempDir) {
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");
}
