use super::support::{assert_zen_failure_contains, run_zen_in, write_file};

#[test]
fn build_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "math.zen",
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    );

    write_file(
        &tmp,
        "main.zen",
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    );

    let args = ["build", "main.zen"];
    let output = run_zen_in(&tmp, &args);
    assert_zen_failure_contains(
        &args,
        &output,
        "return type mismatch: expected `i32`, found `bool`",
    );
}
