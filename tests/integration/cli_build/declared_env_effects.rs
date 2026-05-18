#[path = "declared_env_effects/build_graph.rs"]
mod build_graph;
#[path = "declared_env_effects/emit.rs"]
mod emit;
#[path = "declared_env_effects/executable.rs"]
mod executable;
#[path = "declared_env_effects/rejections.rs"]
mod rejections;
#[path = "declared_env_effects/test_command.rs"]
mod test_command;

use std::process::Command;

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
