use super::support::{
    assert_env_read_rejected, assert_stdout_empty, EMIT_ARGS, EXECUTABLE_ENV_READ_BASIC_CASES,
    EXECUTABLE_LIBRARY_ENV_READ_CASE, UNRELATED_TEST_ENV_READ_CASE,
};

const EMIT_LABEL: &str = "zen emit build.zen";

#[test]
fn emit_command_build_zen_rejects_undeclared_host_effects() {
    for case in [
        &EXECUTABLE_ENV_READ_BASIC_CASES[0],
        &EXECUTABLE_LIBRARY_ENV_READ_CASE,
        &UNRELATED_TEST_ENV_READ_CASE,
    ] {
        let output = assert_env_read_rejected(EMIT_ARGS, case, EMIT_LABEL);
        assert_emit_stdout_empty(&output);
    }
}

fn assert_emit_stdout_empty(output: &std::process::Output) {
    assert_stdout_empty(
        output,
        "emit should not write C source after graph validation fails",
    );
}
