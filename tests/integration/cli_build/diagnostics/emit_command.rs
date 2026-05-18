use std::process::Command;

#[test]
fn emit_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = super::write_imported_module_type_error_fixture(&tmp);

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit");

    super::assert_fails_with(
        &output,
        "zen emit",
        "return type mismatch: expected `i32`, found `bool`",
    );
}
