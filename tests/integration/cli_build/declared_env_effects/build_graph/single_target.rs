use super::super::{
    assert_executable_command_accepts_declared_env_read, ExecutableCommandExpectation,
};

#[test]
fn build_graph_command_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_graph_command_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build-graph", "build.zen"],
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}
