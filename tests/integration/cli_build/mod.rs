mod build_command;
mod build_graph_json;
mod build_graph_json_declared_env;
mod build_graph_json_host_effects;
mod declared_env_effects;
mod diagnostics;
mod emit_direct;
mod emit_direct_host_effects;
mod emit_direct_library_dependencies;
mod emit_direct_validation;
mod frontend_json;
mod graph_validation;
mod graph_validation_host_effects;
mod graph_validation_test_command;
mod graph_validation_test_command_validation;
mod legacy_graph_command;
mod library_execution_gates;
mod support;
mod target_metadata_matrix;
mod unsupported_targets;
mod validation_rejections;

#[test]
fn executable_build_commands_compile_targets() {
    for args in support::EXECUTABLE_ARGS {
        support::assert_single_executable_build(args);
        support::assert_multiple_executable_build(args);
        support::assert_dependent_executable_build_order(args);
    }
}
