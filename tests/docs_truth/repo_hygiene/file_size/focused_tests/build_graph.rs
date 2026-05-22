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

    for test_name in [
        "build_program_lowering_rejects_undeclared_env_reads",
        "build_program_lowering_accepts_declared_env_reads",
        "build_program_lowering_accepts_wildcard_fallback_declared_env_reads",
        "build_program_lowering_accepts_identifier_fallback_declared_env_reads",
        "build_program_lowering_rejects_env_read_without_fallback",
    ] {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "env read host-effect test should move out of the root module: {test_name}"
        );
        assert!(
            env_reads.contains(&fn_name),
            "env_reads.rs should keep env read host-effect test: {test_name}"
        );
    }

    for test_name in [
        "build_program_lowering_accepts_declared_file_reads",
        "build_program_lowering_accepts_wildcard_fallback_declared_file_reads",
        "build_program_lowering_accepts_identifier_fallback_declared_file_reads",
        "build_program_lowering_rejects_undeclared_file_reads",
        "build_program_lowering_rejects_file_read_without_fallback",
    ] {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "file read host-effect test should move out of the root module: {test_name}"
        );
        assert!(
            file_reads.contains(&fn_name),
            "file_reads.rs should keep file read host-effect test: {test_name}"
        );
    }
}
