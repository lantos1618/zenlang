use super::super::support::{
    assert_graph_only_library_build_output, assert_no_build_dir, assert_zen_failure_contains,
    assert_zen_success, run_zen_in, write_graph_only_library_project, GraphOnlyLibrarySource,
    LIBRARY_TYPE_ERROR_DIAGNOSTIC, MISSING_LIBRARY_SOURCE_DIAGNOSTIC,
};

#[test]
fn build_graph_command_rejects_missing_graph_only_library_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_graph_only_library_project(&tmp, GraphOnlyLibrarySource::Missing);

    let args = ["build-graph", "build.zen"];
    let output = run_zen_in(&tmp, &args);

    assert_zen_failure_contains(&args, &output, MISSING_LIBRARY_SOURCE_DIAGNOSTIC);
    assert_no_build_dir(
        tmp.path(),
        "build-graph command after graph source validation failure",
    );
}

#[test]
fn build_graph_command_accepts_valid_graph_only_library_sources() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_graph_only_library_project(&tmp, GraphOnlyLibrarySource::Valid);

    let args = ["build-graph", "build.zen"];
    let output = run_zen_in(&tmp, &args);

    assert_zen_success(&args, &output);
    assert_graph_only_library_build_output(tmp.path());
}

#[test]
fn build_graph_command_rejects_graph_only_library_type_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_graph_only_library_project(&tmp, GraphOnlyLibrarySource::TypeError);

    let args = ["build-graph", "build.zen"];
    let output = run_zen_in(&tmp, &args);

    assert_zen_failure_contains(&args, &output, LIBRARY_TYPE_ERROR_DIAGNOSTIC);
    assert_no_build_dir(
        tmp.path(),
        "build-graph command after graph-only library typechecking failure",
    );
}
