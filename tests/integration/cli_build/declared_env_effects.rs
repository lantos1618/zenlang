use std::process::Command;

#[test]
fn build_command_build_zen_accepts_declared_env_read_with_fallback() {
    let tmp = executable_graph_with_declared_env_read();

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
    assert!(
        tmp.path().join("build").join("app").exists(),
        "expected build output after declared env effect"
    );
}

#[test]
fn direct_file_command_build_zen_accepts_declared_env_read_with_fallback() {
    let tmp = executable_graph_with_declared_env_read();

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        output.status.success(),
        "zen build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        tmp.path().join("build").join("app").exists(),
        "expected build output after declared env effect"
    );
}

#[test]
fn build_graph_command_accepts_declared_env_read_with_fallback() {
    let tmp = executable_graph_with_declared_env_read();

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
    assert!(
        tmp.path().join("build").join("app").exists(),
        "expected build output after declared env effect"
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read() {
    assert_build_graph_command_accepts_declared_env_read(
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read",
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read() {
    assert_build_graph_command_accepts_declared_env_read(
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read",
    );
}

#[test]
fn emit_command_build_zen_accepts_declared_env_read_with_fallback() {
    let tmp = executable_graph_with_declared_env_read();

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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("int32_t zen_main(void)"),
        "expected C output after declared env effect, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs"
    );
}

#[test]
fn test_command_build_zen_accepts_declared_env_read_with_fallback() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
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
        "zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build", "build.zen"],
        "build command should not start after graph validation fails",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build.zen"],
        "direct build.zen command should not start after graph validation fails",
    );
}

#[test]
fn build_graph_command_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build-graph", "build.zen"],
        "build-graph command should not start after graph validation fails",
    );
}

#[test]
fn check_command_build_zen_rejects_env_read_without_fallback_before_source_validation() {
    let tmp = executable_graph_with_env_read_without_fallback("missing.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert_env_read_without_fallback_failed(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("source not found"),
        "host-effect validation should run before source validation, stderr={stderr}"
    );
}

#[test]
fn emit_command_build_zen_rejects_env_read_without_fallback() {
    let tmp = executable_graph_with_env_read_without_fallback("main.zen");
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

    assert_env_read_without_fallback_failed(&output);
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert_env_read_without_fallback_failed(&output);
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}

fn executable_graph_with_declared_env_read() -> tempfile::TempDir {
    executable_graph_with_declared_env_read_fallback(r#"| .Err { "default" }"#)
}

fn assert_build_graph_command_accepts_declared_env_read(fallback_arm: &str, case_name: &str) {
    let tmp = executable_graph_with_declared_env_read_fallback(fallback_arm);

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
    assert!(
        tmp.path().join("build").join("app").exists(),
        "{case_name}: expected build output after declared env effect"
    );
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
