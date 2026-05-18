#[path = "build_command.rs"]
mod build_command;
#[path = "direct_file.rs"]
mod direct_file;

use std::process::Command;

fn assert_executable_command_accepts_declared_env_read_with_unselected_targets(
    args: &[&str],
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
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
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen executable build graph command");

    assert!(
        output.status.success(),
        "{case_name}: zen executable build graph command failed: stdout={}, stderr={}",
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

fn assert_executable_command_accepts_declared_env_read_for_multiple_targets(
    args: &[&str],
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "app.zen", out_dir: "build/app/" }})
    b.add(Executable {{ name: "tool", main: "tool.zen", out_dir: "build/tool/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    for source in ["app.zen", "tool.zen"] {
        std::fs::write(
            tmp.path().join(source),
            r#"
main = () i32 {
    0
}
"#,
        )
        .unwrap_or_else(|err| panic!("write {source}: {err}"));
    }

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen executable build graph command");

    assert!(
        output.status.success(),
        "{case_name}: zen executable build graph command failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
    }
}
