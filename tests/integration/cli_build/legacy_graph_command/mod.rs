use super::support::{
    assert_zen_failure, assert_zen_failure_contains, build_graph_source, run_zen, run_zen_in,
    write_file,
};

#[test]
fn cli_usage_describes_build_graph_executable_targets() {
    let output = run_zen(&[]);

    assert_zen_failure(&[], &output);
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build-graph <build.zen>   Compile executable targets"),
        "expected build-graph plural target usage, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "build.zen",
        &build_graph_source(&[
            r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#,
        ]),
    );

    for mode in ["ast", "symbols", "typed", "diagnostics"] {
        let args = ["emit-json", mode, "build.zen"];
        let output = run_zen_in(&tmp, &args);
        assert_zen_failure_contains(
            &args,
            &output,
            "this emit-json mode does not support build.zen; use `emit-json build-graph`",
        );
    }
}
