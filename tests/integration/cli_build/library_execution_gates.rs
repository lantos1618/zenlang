use super::support::{
    assert_no_build_dir, assert_zen_failure_contains, run_build_zen_command, EMIT_ARGS,
    EXECUTABLE_ARGS, LIBRARY_SOURCE, TEST_ARGS,
};

#[test]
fn library_only_graph_execution_rejections_match_command_mode() {
    for args in EXECUTABLE_ARGS {
        assert_library_only_graph_is_rejected(
            args,
            "build graph execution requires at least one executable target",
        );
    }
    assert_library_only_graph_is_rejected(
        EMIT_ARGS,
        "build graph C emission supports exactly one target, found 0",
    );
    assert_library_only_graph_is_rejected(
        TEST_ARGS,
        "build graph test execution requires at least one test target",
    );
}

fn assert_library_only_graph_is_rejected(args: &[&str], expected_stderr: &str) {
    let (tmp, output) = run_build_zen_command(
        args,
        &[r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#],
        &[("lib.zen", LIBRARY_SOURCE)],
    );
    assert_zen_failure_contains(args, &output, expected_stderr);
    assert_no_build_dir(tmp.path(), "library-only graph command");
}
