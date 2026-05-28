use super::super::super::support::{
    assert_env_read_rejected, CHECK_ARGS, EXECUTABLE_ENV_READ_BASIC_CASES,
    MISSING_SOURCE_ENV_READ_CASE, TYPE_MISMATCH_EXECUTABLE_ENV_READ_CASE,
};

#[test]
fn check_command_build_zen_rejects_undeclared_host_effects() {
    assert_env_read_rejected(
        CHECK_ARGS,
        &EXECUTABLE_ENV_READ_BASIC_CASES[0],
        "zen check build.zen",
    );
}

#[test]
fn check_command_build_zen_rejects_undeclared_host_effects_before_source_validation() {
    assert_env_read_rejected(
        CHECK_ARGS,
        &MISSING_SOURCE_ENV_READ_CASE,
        "zen check build.zen",
    );
}

#[test]
fn check_command_build_zen_rejects_undeclared_host_effects_before_target_typechecking() {
    assert_env_read_rejected(
        CHECK_ARGS,
        &TYPE_MISMATCH_EXECUTABLE_ENV_READ_CASE,
        "zen check build.zen",
    );
}
