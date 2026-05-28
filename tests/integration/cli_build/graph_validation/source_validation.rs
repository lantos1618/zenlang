use super::super::support::{
    assert_zen_failure_contains, run_build_zen_command, CHECK_ARGS, MAIN_TRUE,
};

#[test]
fn check_command_build_zen_typechecks_target_sources() {
    assert_check_rejects(
        &[r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#],
        &[("main.zen", MAIN_TRUE)],
        "return type mismatch: expected `i32`, found `bool`",
    );
}

#[test]
fn check_command_build_zen_rejects_missing_executable_source() {
    assert_check_rejects(
        &[r#"    b.add(Executable { name: "myapp", main: "missing.zen", out_dir: "build/" })"#],
        &[],
        "build graph target `myapp` source not found: missing.zen",
    );
}

#[test]
fn check_command_build_zen_rejects_missing_test_source() {
    assert_check_rejects(
        &[r#"    b.add(Test { name: "unit", root: "missing_test.zen" })"#],
        &[],
        "build graph target `unit` source not found: missing_test.zen",
    );
}

#[test]
fn check_command_build_zen_rejects_missing_library_source() {
    assert_check_rejects(
        &[r#"    b.add(Library { name: "core", exports: ["missing_lib.zen"] })"#],
        &[],
        "build graph target `core` source not found: missing_lib.zen",
    );
}

fn assert_check_rejects(targets: &[&str], files: &[(&str, &str)], expected: &str) {
    let (_, output) = run_build_zen_command(CHECK_ARGS, targets, files);
    assert_zen_failure_contains(CHECK_ARGS, &output, expected);
}
