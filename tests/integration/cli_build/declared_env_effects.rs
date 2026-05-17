#[path = "declared_env_effects/rejections.rs"]
mod rejections;

use std::process::Command;

#[test]
fn build_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        output.status.success(),
        "zen build build.zen failed: stdout={}, stderr={}",
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

#[test]
fn direct_file_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn emit_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| .Err { "default" }"#,
        "emit_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::EmitStdout,
    );
}

#[test]
fn emit_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| _ { "default" }"#,
        "emit_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::EmitStdout,
    );
}

#[test]
fn emit_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| err { "default" }"#,
        "emit_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::EmitStdout,
    );
}

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
fn test_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_test_command_accepts_declared_env_read(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_env_read_with_fallback",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_test_command_accepts_declared_env_read(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_env_read",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_test_command_accepts_declared_env_read(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_env_read",
    );
}

fn assert_test_command_accepts_declared_env_read(fallback_arm: &str, case_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Test {{ name: "unit", root: "test.zen" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

enum ExecutableCommandExpectation {
    BuildOutput,
    EmitStdout,
}

fn assert_executable_command_accepts_declared_env_read(
    args: &[&str],
    fallback_arm: &str,
    case_name: &str,
    expectation: ExecutableCommandExpectation,
) {
    let tmp = executable_graph_with_declared_env_read_fallback(fallback_arm);

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen build graph command");

    assert!(
        output.status.success(),
        "{case_name}: zen command failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    match expectation {
        ExecutableCommandExpectation::BuildOutput => assert!(
            tmp.path().join("build").join("app").exists(),
            "{case_name}: expected build output after declared env effect"
        ),
        ExecutableCommandExpectation::EmitStdout => {
            assert!(
                String::from_utf8_lossy(&output.stdout).contains("int32_t zen_main(void)"),
                "{case_name}: expected C output after declared env effect, stdout={}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(
                !tmp.path().join("build").exists(),
                "{case_name}: zen emit build.zen should not create build outputs"
            );
        }
    }
}

fn executable_graph_with_declared_env_read_fallback(fallback_arm: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "main.zen", out_dir: "build/" }})
    .Ok(b.config())
}}
"#,
        ),
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
    tmp
}

fn assert_env_read_without_fallback_is_rejected(args: &[&str], build_message: &str) {
    let tmp = executable_graph_with_env_read_without_fallback("main.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen build graph command");

    assert_env_read_without_fallback_failed(&output);
    assert!(!tmp.path().join("build").exists(), "{build_message}");
}

fn assert_env_read_without_fallback_failed(output: &std::process::Output) {
    assert!(
        !output.status.success(),
        "zen command unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared env read diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn executable_graph_with_env_read_without_fallback(main_source: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
    b.add(Executable {{ name: "app", main: "{main_source}", out_dir: "build/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    tmp
}
