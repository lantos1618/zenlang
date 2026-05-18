use super::assert_build_graph_command_accepts_declared_env_read_for_multiple_targets;

#[test]
fn build_graph_command_accepts_declared_env_read_for_multiple_targets() {
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| .Err { "default" }"#,
        "build_graph_command_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_graph_command_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| _ { "default" }"#,
        "build_graph_command_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_graph_command_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_build_graph_command_accepts_declared_env_read_for_multiple_targets(
        r#"| err { "default" }"#,
        "build_graph_command_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}
