use super::super::super::*;

#[test]
fn build_graph_host_effect_tests_stay_split_by_effect_kind() {
    let root = read("tests/build_graph/host_effects.rs");
    let env_reads = read("tests/build_graph/host_effects/env_reads.rs");
    let file_reads = read("tests/build_graph/host_effects/file_reads.rs");

    assert!(
        root.lines().count() < 40,
        "host_effects.rs should only route focused build graph host-effect test modules"
    );
    for module in ["mod env_reads;", "mod file_reads;"] {
        assert!(
            root.contains(module),
            "host_effects.rs should include focused module `{module}`"
        );
    }

    assert_tests_live_in_focused_module(
        &root,
        &env_reads,
        "env read host-effect",
        &[
            "build_program_lowering_rejects_undeclared_env_reads",
            "build_program_lowering_accepts_declared_env_reads",
            "build_program_lowering_accepts_wildcard_fallback_declared_env_reads",
            "build_program_lowering_accepts_identifier_fallback_declared_env_reads",
            "build_program_lowering_rejects_env_read_without_fallback",
        ],
    );
    assert_tests_live_in_focused_module(
        &root,
        &file_reads,
        "file read host-effect",
        &[
            "build_program_lowering_accepts_declared_file_reads",
            "build_program_lowering_accepts_wildcard_fallback_declared_file_reads",
            "build_program_lowering_accepts_identifier_fallback_declared_file_reads",
            "build_program_lowering_rejects_undeclared_file_reads",
            "build_program_lowering_rejects_file_read_without_fallback",
        ],
    );
}

#[test]
fn build_graph_target_lowering_tests_stay_split_by_surface() {
    let root = read("tests/build_graph/targets.rs");
    let project = read("tests/build_graph/targets/project.rs");
    let kinds = read("tests/build_graph/targets/kinds.rs");
    let control_flow = read("tests/build_graph/targets/control_flow.rs");
    let metadata = read("tests/build_graph/targets/metadata.rs");

    assert!(
        root.lines().count() < 25,
        "targets.rs should only route focused build graph target test modules"
    );
    for module in [
        "mod project;",
        "mod kinds;",
        "mod control_flow;",
        "mod metadata;",
    ] {
        assert!(
            root.contains(module),
            "targets.rs should include focused module `{module}`"
        );
    }

    assert_tests_live_in_focused_module(
        &root,
        &project,
        "project fixture",
        &["parsed_project_build_zen_lowers_to_executable_and_test_graph"],
    );
    assert_tests_live_in_focused_module(
        &root,
        &kinds,
        "target kind",
        &[
            "build_program_lowering_collects_test_target",
            "build_program_lowering_collects_library_target",
            "build_program_lowering_collects_multiple_executable_targets",
        ],
    );
    assert_tests_live_in_focused_module(
        &root,
        &control_flow,
        "control-flow",
        &[
            "build_program_lowering_collects_static_block_targets",
            "build_program_lowering_rejects_dynamic_target_adds",
        ],
    );
    assert_tests_live_in_focused_module(
        &root,
        &metadata,
        "target metadata",
        &["build_program_lowering_collects_target_dependencies_and_features"],
    );
}

fn assert_tests_live_in_focused_module(
    root: &str,
    focused_module: &str,
    label: &str,
    test_names: &[&str],
) {
    for test_name in test_names {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "{label} build graph test should move out of the root module: {test_name}"
        );
        assert!(
            focused_module.contains(&fn_name),
            "{label} module should keep build graph test: {test_name}"
        );
    }
}
