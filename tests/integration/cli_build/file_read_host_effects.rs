use super::support::{
    assert_build_file_read_rejected, assert_declared_file_read_emit,
    assert_declared_file_read_multiple_executable_build,
    assert_declared_file_read_single_executable_build,
    assert_declared_file_read_unselected_target_build,
    assert_declared_file_read_unselected_target_emit, assert_emit_file_read_rejected,
    assert_emit_file_read_rejected_before_unselected_targets,
    assert_file_read_rejected_before_unselected_targets,
    assert_test_command_declared_file_read_multiple_targets,
    assert_test_command_declared_file_read_single_target,
    assert_test_command_declared_file_read_unselected_targets,
    assert_test_command_file_read_rejected,
    assert_test_command_file_read_rejected_before_unselected_targets,
    missing_fallback_file_read_unselected_target_graph,
    missing_fallback_multiple_executable_file_read_graph,
    missing_fallback_single_executable_file_read_graph, missing_fallback_test_file_read_graph,
    missing_fallback_test_file_read_unselected_target_graph,
    missing_fallback_unselected_file_read_graph, undeclared_file_read_unselected_target_graph,
    undeclared_multiple_executable_file_read_graph, undeclared_single_executable_file_read_graph,
    undeclared_test_file_read_graph, undeclared_test_file_read_unselected_target_graph,
    undeclared_unselected_file_read_graph, BUILD_ARGS, BUILD_GRAPH_ARGS,
    DECLARED_FILE_READ_FALLBACK_ARMS, DIRECT_ARGS, EMIT_ARGS, EXECUTABLE_ARGS,
    EXECUTABLE_COMMAND_LABELS,
};

const SINGLE_TARGET_EXECUTABLE_COMMANDS: &[(&[&str], bool)] = &[
    (BUILD_ARGS, false),
    (DIRECT_ARGS, true),
    (BUILD_GRAPH_ARGS, true),
];
#[test]
fn executable_commands_accept_declared_file_read_effects() {
    for (args, run_all_binaries) in SINGLE_TARGET_EXECUTABLE_COMMANDS {
        for (index, fallback_arm) in DECLARED_FILE_READ_FALLBACK_ARMS.iter().enumerate() {
            assert_declared_file_read_single_executable_build(
                args,
                fallback_arm,
                *run_all_binaries || index == 0,
            );
        }
    }
}

#[test]
fn executable_commands_accept_declared_file_read_effects_for_multiple_targets() {
    for args in EXECUTABLE_ARGS {
        for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
            assert_declared_file_read_multiple_executable_build(args, fallback_arm);
        }
    }
}

#[test]
fn commands_accept_declared_file_read_effects_with_unselected_targets() {
    for args in EXECUTABLE_ARGS {
        for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
            assert_declared_file_read_unselected_target_build(args, fallback_arm);
        }
    }

    for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
        assert_declared_file_read_unselected_target_emit(EMIT_ARGS, fallback_arm);
        assert_test_command_declared_file_read_unselected_targets(fallback_arm);
    }
}

#[test]
fn emit_command_accepts_declared_file_read_effects() {
    for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
        assert_declared_file_read_emit(EMIT_ARGS, fallback_arm);
    }
}

#[test]
fn test_command_accepts_declared_file_read_effects() {
    for fallback_arm in DECLARED_FILE_READ_FALLBACK_ARMS {
        assert_test_command_declared_file_read_single_target(fallback_arm);
        assert_test_command_declared_file_read_multiple_targets(fallback_arm);
    }
}

#[test]
fn executable_commands_reject_file_read_before_unselected_targets() {
    for args in EXECUTABLE_ARGS {
        for source in [
            undeclared_file_read_unselected_target_graph(),
            missing_fallback_file_read_unselected_target_graph(),
        ] {
            assert_file_read_rejected_before_unselected_targets(args, source);
        }
    }
}

#[test]
fn emit_command_rejects_file_read_host_effects() {
    for source in [
        undeclared_single_executable_file_read_graph(),
        missing_fallback_single_executable_file_read_graph(),
    ] {
        assert_emit_file_read_rejected(EMIT_ARGS, source);
    }
}

#[test]
fn emit_command_rejects_file_read_before_unselected_targets() {
    for source in [
        undeclared_unselected_file_read_graph(),
        missing_fallback_unselected_file_read_graph(),
    ] {
        assert_emit_file_read_rejected_before_unselected_targets(EMIT_ARGS, source);
    }
}

#[test]
fn test_command_rejects_file_read_before_execution() {
    for source in [
        undeclared_test_file_read_graph(&[
            ("unit", "unit.zen"),
            ("integration", "integration.zen"),
        ]),
        missing_fallback_test_file_read_graph(&[("unit", "test.zen")]),
        missing_fallback_test_file_read_graph(&[
            ("unit", "unit.zen"),
            ("integration", "integration.zen"),
        ]),
    ] {
        assert_test_command_file_read_rejected(&source);
    }
}

#[test]
fn test_command_rejects_file_read_before_unselected_targets() {
    for source in [
        undeclared_test_file_read_unselected_target_graph(),
        missing_fallback_test_file_read_unselected_target_graph(),
    ] {
        assert_test_command_file_read_rejected_before_unselected_targets(source);
    }
}

#[test]
fn executable_commands_reject_file_reads_before_execution() {
    for (args, label) in EXECUTABLE_COMMAND_LABELS {
        for source in [
            undeclared_single_executable_file_read_graph(),
            undeclared_multiple_executable_file_read_graph(),
            missing_fallback_single_executable_file_read_graph(),
            missing_fallback_multiple_executable_file_read_graph(),
        ] {
            assert_build_file_read_rejected(args, source, label);
        }
    }
}
