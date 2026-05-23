use super::super::*;

#[test]
fn cli_file_read_host_effect_tests_share_fixture_writers() {
    let support = read("tests/integration/cli_build/support.rs");
    let build_command_single = read(
        "tests/integration/cli_build/build_command_host_effects/file_reads/declared/single_target.rs",
    );

    assert!(
        support.contains("fn write_single_executable_file_read_graph("),
        "CLI build host-effect tests should share single-target file-read graph setup"
    );
    assert!(
        !build_command_single.contains("std::fs::write("),
        "build-command single-target file-read tests should not repeat fixture file writes"
    );
    assert!(
        build_command_single.contains("write_single_executable_file_read_graph("),
        "build-command single-target file-read tests should use the shared file-read graph helper"
    );
}
