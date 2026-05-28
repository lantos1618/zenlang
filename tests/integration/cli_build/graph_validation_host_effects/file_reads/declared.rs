use super::super::super::support::{
    assert_declared_file_read_mixed_target_check,
    assert_declared_file_read_single_executable_check, DECLARED_FILE_READ_FALLBACK_ARMS,
};

#[test]
fn check_command_build_zen_accepts_declared_file_read_effects() {
    for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
        assert_declared_file_read_single_executable_check(fallback_arm);
    }
}

#[test]
fn check_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets() {
    for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
        assert_declared_file_read_mixed_target_check(fallback_arm);
    }
}
