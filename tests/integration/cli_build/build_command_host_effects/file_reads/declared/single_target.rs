use std::process::Command;

#[test]
fn build_command_build_zen_accepts_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_file_read_effects",
        true,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects",
        false,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects",
        false,
    );
}

fn assert_build_command_build_zen_accepts_declared_file_read_effects(
    fallback_arm: &str,
    case_name: &str,
    run_binary: bool,
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bin_path = tmp.path().join("build").join("myapp");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );

    if run_binary {
        let run = Command::new(&bin_path).output().expect("run built binary");
        assert!(
            run.status.success(),
            "built binary exited with {}",
            run.status
        );
    }
}
