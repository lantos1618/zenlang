use super::support::{
    assert_emit_c_source, assert_no_build_dir, assert_stdout_empty, assert_zen_failure_contains,
    assert_zen_success, run_build_zen_command, run_zen_in, write_file, EMIT_ARGS, MAIN_ZERO,
};
mod target_selection;

#[test]
fn emit_command_build_zen_ignores_unrelated_gated_test_source_errors() {
    let (tmp, output) = run_emit_build_zen(
        &[
            r#"    b.add(Test { name: "unit", root: "missing_test.zen" })"#,
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
        ],
        &[("app.zen", MAIN_ZERO)],
    );

    assert_zen_success(EMIT_ARGS, &output);
    assert_emit_c_source(&output);
    assert_no_build_dir(tmp.path(), "zen emit build.zen");
}

fn run_emit_build_zen(
    targets: &[&str],
    files: &[(&str, &str)],
) -> (tempfile::TempDir, std::process::Output) {
    run_build_zen_command(EMIT_ARGS, targets, files)
}

pub(super) fn assert_emit_command_rejects_without_outputs(
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", build_source);
    let output = run_zen_in(&tmp, EMIT_ARGS);

    assert_zen_failure_contains(EMIT_ARGS, &output, expected_diagnostic);
    assert_stdout_empty(
        &output,
        "zen emit build.zen should not write C source after target metadata validation fails",
    );
    assert_no_build_dir(tmp.path(), "zen emit build.zen");
}
