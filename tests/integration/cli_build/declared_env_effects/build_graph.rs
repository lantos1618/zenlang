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
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| .Err { "default" }"#,
        "build_graph_command_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_graph_command_accepts_declared_env_read_with_unselected_targets() {
    assert_build_graph_command_accepts_declared_env_read_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "build_graph_command_accepts_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read_with_unselected_targets() {
    assert_build_graph_command_accepts_declared_env_read_with_unselected_targets(
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read_with_unselected_targets() {
    assert_build_graph_command_accepts_declared_env_read_with_unselected_targets(
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read_with_unselected_targets",
    );
}

fn assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
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
        "{case_name}: zen build-graph build.zen failed: stdout={}, stderr={}",
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

fn assert_build_graph_command_accepts_declared_env_read_with_unselected_targets(
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
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build-graph build.zen failed: stdout={}, stderr={}",
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
