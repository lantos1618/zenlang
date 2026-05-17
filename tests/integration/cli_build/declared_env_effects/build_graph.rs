use std::process::Command;

use super::{assert_executable_command_accepts_declared_env_read, ExecutableCommandExpectation};

#[test]
fn build_graph_command_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_graph_command_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_graph_command_accepts_declared_env_read_for_multiple_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
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
    std::fs::write(
        tmp.path().join("tool.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write tool.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph build.zen");

    assert!(
        output.status.success(),
        "zen build-graph build.zen failed: stdout={}, stderr={}",
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
