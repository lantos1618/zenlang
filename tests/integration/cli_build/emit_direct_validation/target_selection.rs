use std::process::Command;

#[path = "target_selection/ambiguity.rs"]
mod ambiguity;
#[path = "target_selection/no_executable.rs"]
mod no_executable;

fn run_emit_build_zen(
    build_source: &str,
    files: &[(&str, String)],
) -> (tempfile::TempDir, std::process::Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    for (path, source) in files {
        std::fs::write(tmp.path().join(path), source).expect("write source file");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    (tmp, output)
}

fn assert_emit_rejected_without_outputs(
    tmp: &tempfile::TempDir,
    output: &std::process::Output,
    expected: &str,
    output_reason: &str,
) {
    assert!(
        !output.status.success(),
        "zen emit build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "expected {output_reason} diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs when {output_reason}"
    );
}

fn main_source(value: &str) -> String {
    format!(
        r#"
main = () i32 {{
    {value}
}}
"#,
    )
}
