use std::process::Command;

#[test]
fn build_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build", main_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build");

    assert!(
        !output.status.success(),
        "zen build unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
