use super::support::{
    assert_dependency_shape_rejections, assert_direct_gated_executable_dependency_rejected,
    assert_direct_gated_test_dependency_rejected, assert_env_read_rejected,
    assert_executable_build_accepts_library_dependency,
    assert_executable_build_ignores_unrelated_test_source,
    assert_executable_build_rejects_no_executable, assert_graph_only_library_command_cases,
    assert_transitive_gated_executable_dependency_rejected,
    assert_transitive_gated_test_dependency_rejected, assert_zen_failure_contains,
    build_graph_source, run_zen_in, write_file, write_graph_only_library_project,
    write_graph_only_library_test_project, GraphOnlyLibrarySuccess, GraphOnlyProjectWriter,
    BUILD_ARGS, BUILD_GRAPH_ARGS, CHECK_ARGS, DEPENDENT_EXECUTABLE_ENV_READ_CASE, DIRECT_ARGS,
    EMIT_ARGS, EXECUTABLE_ARGS, EXECUTABLE_COMMAND_LABELS, EXECUTABLE_DEPENDENCY_SHAPE_CASES,
    EXECUTABLE_ENV_READ_BASIC_CASES, EXECUTABLE_LIBRARY_ENV_READ_CASE, TEST_ARGS,
    TEST_DEPENDENCY_SHAPE_CASES, TEST_ENV_READ_BASIC_CASES, TEST_LIBRARY_ENV_READ_CASE,
    UNRELATED_EXECUTABLE_ENV_READ_CASE, UNRELATED_TEST_ENV_READ_CASE,
};

const EXECUTABLE_COMMANDS: &[(&[&str], &str)] = &[
    (BUILD_ARGS, "build command"),
    (DIRECT_ARGS, "direct build.zen command"),
    (BUILD_GRAPH_ARGS, "build-graph command"),
];

#[test]
fn executable_commands_reject_dependency_shape_errors() {
    for (args, label) in EXECUTABLE_COMMANDS
        .iter()
        .copied()
        .chain([(CHECK_ARGS, "zen check build.zen")])
    {
        assert_dependency_shape_rejections(args, EXECUTABLE_DEPENDENCY_SHAPE_CASES, label);
    }
}

#[test]
fn emit_command_rejects_dependency_shape_errors() {
    for &(targets, diagnostic) in EXECUTABLE_DEPENDENCY_SHAPE_CASES {
        super::emit_direct_validation::assert_emit_command_rejects_without_outputs(
            &build_graph_source(targets),
            diagnostic,
        );
    }
}

#[test]
fn test_command_rejects_dependency_shape_errors() {
    assert_dependency_shape_rejections(TEST_ARGS, TEST_DEPENDENCY_SHAPE_CASES, "test command");
}

#[test]
fn executable_commands_reject_gated_test_dependencies() {
    for (args, label) in EXECUTABLE_COMMANDS
        .iter()
        .copied()
        .chain([(EMIT_ARGS, "zen emit build.zen")])
    {
        assert_direct_gated_test_dependency_rejected(args, label);
        assert_transitive_gated_test_dependency_rejected(args, label);
    }
}

#[test]
fn test_command_rejects_gated_executable_dependencies() {
    assert_direct_gated_executable_dependency_rejected(TEST_ARGS, "test command");
    assert_transitive_gated_executable_dependency_rejected(TEST_ARGS, "test command");
}

#[test]
fn command_modes_handle_graph_only_libraries() {
    for (args, writer, label, success) in [
        (
            BUILD_ARGS,
            write_graph_only_library_project as GraphOnlyProjectWriter,
            "build command after graph source validation failure",
            GraphOnlyLibrarySuccess::Build,
        ),
        (
            DIRECT_ARGS,
            write_graph_only_library_project as GraphOnlyProjectWriter,
            "direct build.zen command after graph source validation failure",
            GraphOnlyLibrarySuccess::Build,
        ),
        (
            BUILD_GRAPH_ARGS,
            write_graph_only_library_project as GraphOnlyProjectWriter,
            "build-graph command after graph source validation failure",
            GraphOnlyLibrarySuccess::Build,
        ),
        (
            EMIT_ARGS,
            write_graph_only_library_project as GraphOnlyProjectWriter,
            "zen emit build.zen after graph source validation failure",
            GraphOnlyLibrarySuccess::Emit,
        ),
        (
            TEST_ARGS,
            write_graph_only_library_test_project as GraphOnlyProjectWriter,
            "test command after graph source validation failure",
            GraphOnlyLibrarySuccess::Test,
        ),
    ] {
        assert_graph_only_library_command_cases(args, writer, label, success);
    }
}

#[test]
fn executable_commands_reject_graphs_without_executable_targets() {
    for (args, label) in EXECUTABLE_COMMANDS {
        assert_executable_build_rejects_no_executable(args, label);
    }
}

#[test]
fn executable_commands_accept_library_dependencies() {
    for args in EXECUTABLE_ARGS {
        assert_executable_build_accepts_library_dependency(args);
    }
}

#[test]
fn executable_commands_ignore_unrelated_test_source_errors() {
    for args in EXECUTABLE_ARGS {
        assert_executable_build_ignores_unrelated_test_source(args);
    }
}

#[test]
fn build_graph_command_rejects_missing_root_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "build.zen",
        &build_graph_source(&[
            r#"    b.add(Executable { name: "myapp", main: "missing.zen", out_dir: "build/" })"#,
        ]),
    );

    let output = run_zen_in(&tmp, BUILD_GRAPH_ARGS);

    assert_zen_failure_contains(
        BUILD_GRAPH_ARGS,
        &output,
        "build graph target `myapp` root source not found: missing.zen",
    );
}

#[test]
fn command_modes_reject_basic_env_reads() {
    for (args, label) in EXECUTABLE_COMMAND_LABELS {
        for case in EXECUTABLE_ENV_READ_BASIC_CASES {
            assert_env_read_rejected(args, case, label);
        }
    }
    for case in TEST_ENV_READ_BASIC_CASES {
        assert_env_read_rejected(TEST_ARGS, case, "zen test build.zen");
    }
}

#[test]
fn command_modes_reject_env_reads_before_later_validation() {
    for case in [
        DEPENDENT_EXECUTABLE_ENV_READ_CASE,
        EXECUTABLE_LIBRARY_ENV_READ_CASE,
        UNRELATED_TEST_ENV_READ_CASE,
    ] {
        assert_env_read_rejected(BUILD_ARGS, &case, "zen build build.zen");
    }
    for (args, label) in &EXECUTABLE_COMMAND_LABELS[1..] {
        for case in [
            EXECUTABLE_LIBRARY_ENV_READ_CASE,
            UNRELATED_TEST_ENV_READ_CASE,
        ] {
            assert_env_read_rejected(args, &case, label);
        }
    }
    for case in [
        TEST_LIBRARY_ENV_READ_CASE,
        UNRELATED_EXECUTABLE_ENV_READ_CASE,
    ] {
        assert_env_read_rejected(TEST_ARGS, &case, "zen test build.zen");
    }
}
