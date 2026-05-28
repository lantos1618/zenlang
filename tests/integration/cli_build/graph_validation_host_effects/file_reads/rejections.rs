use super::super::super::support::{
    assert_check_file_read_rejected_before_source_validation,
    missing_fallback_single_executable_file_read_graph,
    missing_fallback_unselected_file_read_graph, undeclared_single_executable_file_read_graph,
    undeclared_unselected_file_read_graph, CHECK_ARGS,
};

#[test]
fn check_command_build_zen_rejects_file_read_host_effects_before_source_validation() {
    for source in [
        undeclared_single_executable_file_read_graph(),
        undeclared_unselected_file_read_graph(),
        missing_fallback_single_executable_file_read_graph(),
        missing_fallback_unselected_file_read_graph(),
    ] {
        assert_check_file_read_rejected_before_source_validation(CHECK_ARGS, source);
    }
}
