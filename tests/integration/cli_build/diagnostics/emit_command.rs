use super::super::support::run_zen_in;

#[test]
fn emit_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = super::write_imported_module_type_error_fixture(&tmp);

    let args = ["emit", main_path];
    let output = run_zen_in(&tmp, &args);

    super::assert_fails_with(
        &output,
        &args,
        "return type mismatch: expected `i32`, found `bool`",
    );
}
