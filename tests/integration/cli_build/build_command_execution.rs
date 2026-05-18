#[test]
fn build_command_routes_build_zen_through_deterministic_graph() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::support::write_single_executable_graph(&tmp);
    let output = super::support::run_zen_in(&tmp, &args);
    super::support::assert_zen_success(&args, &output);

    super::support::assert_built_binaries_run(&[tmp.path().join("build").join("myapp")]);
}

#[test]
fn build_command_build_zen_compiles_multiple_executable_targets() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::support::write_multiple_executable_graph(&tmp);
    let output = super::support::run_zen_in(&tmp, &args);
    super::support::assert_zen_success(&args, &output);

    super::support::assert_built_binaries_run(&[
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ]);
}

#[test]
fn build_command_build_zen_compiles_executable_dependencies_first() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::support::write_dependent_executable_graph(&tmp);
    let output = super::support::run_zen_in(&tmp, &args);
    super::support::assert_zen_success(&args, &output);

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

    super::support::assert_built_binaries_run(&[
        tmp.path().join("build").join("tool").join("tool"),
        tmp.path().join("build").join("app").join("app"),
    ]);
}
