use super::super::support::{run_zen_in, write_file};

#[test]
fn check_command_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "bad_resolver_ref.zen",
        r#"
main = () i32 {
    missing_local
}
"#,
    );

    let args = ["check", "bad_resolver_ref.zen"];
    let output = run_zen_in(&tmp, &args);

    super::assert_fails_with(&output, &args, "unknown value symbol 'missing_local'");
}

#[test]
fn check_command_reports_imported_module_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "math.zen",
        r#"
add = (a: i32, b: i32) i32 {
    a + b
}

broken = () i32 {
    missing_dep_local
}
@export({ add, broken })
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

    let args = ["check", "main.zen"];
    let output = run_zen_in(&tmp, &args);

    super::assert_fails_with(&output, &args, "unknown value symbol 'missing_dep_local'");
}

#[test]
fn check_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = super::write_imported_module_type_error_fixture(&tmp);

    let args = ["check", main_path];
    let output = run_zen_in(&tmp, &args);

    super::assert_fails_with(
        &output,
        &args,
        "return type mismatch: expected `i32`, found `bool`",
    );
}
