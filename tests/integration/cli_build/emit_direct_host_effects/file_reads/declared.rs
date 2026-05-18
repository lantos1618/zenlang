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
