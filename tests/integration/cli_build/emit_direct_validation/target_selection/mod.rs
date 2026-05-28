use super::super::support::{
    assert_no_build_dir, assert_zen_failure_contains, EMIT_ARGS, MAIN_TRUE, MAIN_ZERO,
};
mod ambiguity;
mod no_executable;

fn assert_emit_rejected_without_outputs(
    tmp: &tempfile::TempDir,
    output: &std::process::Output,
    expected: &str,
    output_reason: &str,
) {
    assert_zen_failure_contains(EMIT_ARGS, output, expected);
    assert_no_build_dir(tmp.path(), output_reason);
}
