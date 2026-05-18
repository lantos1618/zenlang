use super::super::{
    assert_executable_command_accepts_declared_env_read, ExecutableCommandExpectation,
};
use super::{
    assert_executable_command_accepts_declared_env_read_for_multiple_targets,
    assert_executable_command_accepts_declared_env_read_with_unselected_targets,
};

#[test]
fn build_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_declared_env_read_with_unselected_targets() {
    assert_executable_command_accepts_declared_env_read_with_unselected_targets(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets() {
    assert_executable_command_accepts_declared_env_read_with_unselected_targets(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets() {
    assert_executable_command_accepts_declared_env_read_with_unselected_targets(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets",
    );
}
