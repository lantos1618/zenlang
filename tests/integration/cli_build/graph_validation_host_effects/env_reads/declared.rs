use super::super::super::support::{
    assert_declared_env_read_mixed_target_check, assert_declared_env_read_single_executable_check,
    DECLARED_ENV_READ_FALLBACK_ARMS,
};

#[test]
fn check_command_build_zen_accepts_declared_env_read() {
    for fallback_arm in DECLARED_ENV_READ_FALLBACK_ARMS {
        assert_declared_env_read_single_executable_check(fallback_arm);
    }
}

#[test]
fn check_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    for fallback_arm in DECLARED_ENV_READ_FALLBACK_ARMS {
        assert_declared_env_read_mixed_target_check(fallback_arm);
    }
}
