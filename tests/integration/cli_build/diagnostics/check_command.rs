use std::process::Command;

#[test]
fn check_command_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    missing_local
}
"#,
    )
    .expect("write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    super::assert_fails_with(&output, "zen check", "unknown value symbol 'missing_local'");
}

#[test]
fn check_command_reports_imported_module_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    missing_dep_local
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
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    super::assert_fails_with(
        &output,
        "zen check",
        "unknown value symbol 'missing_dep_local'",
    );
}

#[test]
fn check_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = super::write_imported_module_type_error_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    super::assert_fails_with(
        &output,
        "zen check",
        "return type mismatch: expected `i32`, found `bool`",
    );
}
