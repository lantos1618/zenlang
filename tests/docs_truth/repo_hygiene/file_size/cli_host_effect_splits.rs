use super::super::*;

#[test]
fn cli_file_read_host_effect_tests_share_fixture_writers() {
    let support = read("tests/integration/cli_build/support.rs");
    let file_read_support = read("tests/integration/cli_build/support/file_read_graphs.rs");
    let build_command_single = read(
        "tests/integration/cli_build/build_command_host_effects/file_reads/declared/single_target.rs",
    );
    let build_command_multiple = read(
        "tests/integration/cli_build/build_command_host_effects/file_reads/declared/multiple_targets.rs",
    );

    assert!(
        support.contains("mod file_read_graphs;"),
        "CLI build support should split file-read graph helpers into a focused module"
    );
    assert!(
        file_read_support.contains("fn write_file_read_executable_graph("),
        "CLI build host-effect tests should share file-read graph setup"
    );
    assert!(
        !build_command_single.contains("std::fs::write("),
        "build-command single-target file-read tests should not repeat fixture file writes"
    );
    assert!(
        build_command_single.contains("write_single_executable_file_read_graph("),
        "build-command single-target file-read tests should use the shared file-read graph helper"
    );
    assert!(
        !build_command_multiple.contains("std::fs::write("),
        "build-command multiple-target file-read tests should not repeat fixture file writes"
    );
    assert!(
        build_command_multiple.contains("write_multiple_executable_file_read_graph("),
        "build-command multiple-target file-read tests should use the shared file-read graph helper"
    );
}
