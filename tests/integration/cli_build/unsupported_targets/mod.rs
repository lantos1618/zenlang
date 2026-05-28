use super::support::{
    assert_no_build_dir, assert_zen_failure_contains, run_build_zen_command,
    ALL_BUILD_ZEN_COMMAND_ARGS,
};
mod fields;
mod kinds;

fn assert_rejected_without_outputs(
    tmp: &tempfile::TempDir,
    output: &std::process::Output,
    args: &[&str],
    expected: String,
    reason: &str,
) {
    assert_zen_failure_contains(args, output, &expected);
    assert_no_build_dir(tmp.path(), reason);
}
