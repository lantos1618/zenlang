use std::process::Command;

#[test]
fn emit_command_build_zen_accepts_declared_file_read_effects() {
    assert_emit_command_accepts_declared_file_read_effect(
        r#"| .Err { "default" }"#,
        "emit_command_build_zen_accepts_declared_file_read_effects",
    );
}

#[test]
fn emit_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects() {
    assert_emit_command_accepts_declared_file_read_effect(
        r#"| _ { "default" }"#,
        "emit_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects",
    );
}

#[test]
fn emit_command_build_zen_accepts_identifier_fallback_declared_file_read_effects() {
    assert_emit_command_accepts_declared_file_read_effect(
        r#"| err { "default" }"#,
        "emit_command_build_zen_accepts_identifier_fallback_declared_file_read_effects",
    );
}

fn assert_emit_command_accepts_declared_file_read_effect(fallback_arm: &str, case_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("build.targets"), "myapp\n").expect("write manifest");
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
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen emit build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let c_source = String::from_utf8_lossy(&output.stdout);
    assert!(
        c_source.contains("int32_t zen_main(void)"),
        "{case_name}: expected target C source, stdout={c_source}"
    );
    assert!(
        !tmp.path().join("build").join("myapp").exists(),
        "zen emit build.zen should not compile the target binary"
    );
}

#[test]
fn emit_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets() {
    assert_emit_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "emit_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn emit_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_emit_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| _ { "default" }"#,
        "emit_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn emit_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_emit_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| err { "default" }"#,
        "emit_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

fn assert_emit_command_accepts_declared_file_read_effects_with_unselected_targets(
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
    b.add(Test {{ name: "unit", root: "unit.zen" }})
    b.add(Library {{ name: "core", exports: ["lib.zen"] }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("build.targets"), "app\n").expect("write manifest");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    std::fs::write(
        tmp.path().join("unit.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write unit.zen");
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
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen emit build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let c_source = String::from_utf8_lossy(&output.stdout);
    assert!(
        c_source.contains("int32_t zen_main(void)"),
        "expected target C source, stdout={c_source}"
    );
    assert!(
        !tmp.path().join("build").join("app").join("app").exists(),
        "zen emit build.zen should not compile the target binary"
    );
}

#[test]
fn emit_command_build_zen_rejects_undeclared_file_read_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file read diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn emit_command_build_zen_rejects_undeclared_file_read_effects_before_unselected_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["missing_lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
        stderr.contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file read diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("missing_app.zen")
            && !stderr.contains("missing_unit.zen")
            && !stderr.contains("missing_lib.zen"),
        "emit should reject undeclared file reads before target source validation, stderr={stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn emit_command_build_zen_rejects_file_read_without_fallback_before_unselected_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["missing_lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
        stderr.contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file read diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("missing_app.zen")
            && !stderr.contains("missing_unit.zen")
            && !stderr.contains("missing_lib.zen"),
        "emit should reject file reads without fallback before target source validation, stderr={stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn emit_command_build_zen_rejects_file_read_without_fallback() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file read diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}
