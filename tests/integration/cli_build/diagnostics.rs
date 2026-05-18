#[path = "diagnostics/check_command.rs"]
mod check_command;
#[path = "diagnostics/dedup.rs"]
mod dedup;
#[path = "diagnostics/emit_command.rs"]
mod emit_command;

fn write_imported_module_type_error_fixture(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");
    main_path
}

fn assert_fails_with(output: &std::process::Output, command_name: &str, expected: &str) {
    assert!(
        !output.status.success(),
        "{command_name} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "expected diagnostic `{expected}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
