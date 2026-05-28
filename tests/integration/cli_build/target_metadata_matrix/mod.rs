use super::support::{
    assert_build_zen_rejected, build_graph_source, BUILD_ZEN_VALIDATION_ARGS,
    EXECUTABLE_TARGET_METADATA_CASES, LIBRARY_TARGET_METADATA_CASES, TEST_TARGET_METADATA_CASES,
};
mod deterministic_body;

#[test]
fn build_zen_commands_reject_target_metadata_errors() {
    for cases in [
        EXECUTABLE_TARGET_METADATA_CASES,
        LIBRARY_TARGET_METADATA_CASES,
        TEST_TARGET_METADATA_CASES,
    ] {
        for &(build_body, expected_diagnostic) in cases {
            assert_build_zen_commands_reject_build_graph_metadata(build_body, expected_diagnostic);
        }
    }
}

fn assert_build_zen_commands_reject_build_graph_metadata(
    build_body: &str,
    expected_diagnostic: &str,
) {
    let build_source = build_graph_source(&[build_body]);
    for args in BUILD_ZEN_VALIDATION_ARGS {
        assert_build_zen_rejected(args, &build_source, expected_diagnostic, "zen build.zen");
    }
    super::emit_direct_validation::assert_emit_command_rejects_without_outputs(
        &build_source,
        expected_diagnostic,
    );
}
