#[test]
fn build_graph_command_compiles_single_executable_target() {
    let args = ["build-graph", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::super::support::write_single_executable_graph(&tmp);
    let output = super::super::support::run_zen_in(&tmp, &args);
    super::super::support::assert_zen_success(&args, &output);

    super::super::support::assert_built_binaries_run(&[tmp.path().join("build").join("myapp")]);
}

#[test]
fn build_graph_command_compiles_multiple_executable_targets() {
    let args = ["build-graph", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::super::support::write_multiple_executable_graph(&tmp);
    let output = super::super::support::run_zen_in(&tmp, &args);
    super::super::support::assert_zen_success(&args, &output);

    super::super::support::assert_built_binaries_run(&[
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ]);
}

#[test]
fn build_graph_command_compiles_executable_dependencies_first() {
    let args = ["build-graph", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::super::support::write_dependent_executable_graph(&tmp);
    let output = super::super::support::run_zen_in(&tmp, &args);
    super::super::support::assert_zen_success(&args, &output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tool_emit = stdout
        .find("build/tool/tool.c")
        .unwrap_or_else(|| panic!("expected tool emission in stdout={stdout}"));
    let app_emit = stdout
        .find("build/app/app.c")
        .unwrap_or_else(|| panic!("expected app emission in stdout={stdout}"));
    assert!(
        tool_emit < app_emit,
        "expected dependency target to compile before dependent target, stdout={stdout}"
    );

    super::super::support::assert_built_binaries_run(&[
        tmp.path().join("build").join("tool").join("tool"),
        tmp.path().join("build").join("app").join("app"),
    ]);
}
