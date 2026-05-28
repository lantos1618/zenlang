use super::{
    assert_path_exists, assert_stdout_contains, assert_test_binary_and_output,
    assert_zen_failure_contains, assert_zen_success, build_graph_source, run_zen_in, write_file,
    LIBRARY_SOURCE, LIBRARY_TRUE, MAIN_ZERO,
};
use std::path::Path;
use std::process::Output;

pub(crate) const MISSING_LIBRARY_SOURCE_DIAGNOSTIC: &str =
    "build graph target `core` source not found: missing_lib.zen";
pub(crate) const LIBRARY_TYPE_ERROR_DIAGNOSTIC: &str =
    "return type mismatch: expected `i32`, found `bool`";

#[derive(Clone, Copy)]
pub(crate) enum GraphOnlyLibrarySource {
    Missing,
    Valid,
    TypeError,
}

pub(crate) type GraphOnlyProjectWriter = fn(&tempfile::TempDir, GraphOnlyLibrarySource);

pub(crate) enum GraphOnlyLibrarySuccess {
    Build,
    Emit,
    Test,
}

pub(crate) fn write_graph_only_library_project(
    tmp: &tempfile::TempDir,
    source: GraphOnlyLibrarySource,
) {
    write_graph_only_library_fixture(
        tmp,
        r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
        "app.zen",
        source,
    );
}

pub(crate) fn write_graph_only_library_test_project(
    tmp: &tempfile::TempDir,
    source: GraphOnlyLibrarySource,
) {
    write_graph_only_library_fixture(
        tmp,
        r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
        "test.zen",
        source,
    );
}

fn write_graph_only_library_fixture(
    tmp: &tempfile::TempDir,
    target_add: &str,
    entry_source: &str,
    source: GraphOnlyLibrarySource,
) {
    let export = match source {
        GraphOnlyLibrarySource::Missing => "missing_lib.zen",
        GraphOnlyLibrarySource::Valid | GraphOnlyLibrarySource::TypeError => "lib.zen",
    };

    let library_add = format!(r#"    b.add(Library {{ name: "core", exports: ["{export}"] }})"#);
    write_file(
        tmp,
        "build.zen",
        &build_graph_source(&[target_add, &library_add]),
    );
    write_file(tmp, entry_source, MAIN_ZERO);

    match source {
        GraphOnlyLibrarySource::Missing => {}
        GraphOnlyLibrarySource::Valid => write_file(tmp, "lib.zen", LIBRARY_SOURCE),
        GraphOnlyLibrarySource::TypeError => write_file(tmp, "lib.zen", LIBRARY_TRUE),
    }
}

pub(crate) fn assert_graph_only_library_command_cases(
    args: &[&str],
    write_project: GraphOnlyProjectWriter,
    command_label: &str,
    success: GraphOnlyLibrarySuccess,
) {
    assert_graph_only_library_rejected(
        args,
        write_project,
        GraphOnlyLibrarySource::Missing,
        MISSING_LIBRARY_SOURCE_DIAGNOSTIC,
        command_label,
    );
    let (tmp, output) =
        run_graph_only_library_command(args, write_project, GraphOnlyLibrarySource::Valid);

    assert_zen_success(args, &output);
    match success {
        GraphOnlyLibrarySuccess::Build => {
            assert_path_exists(tmp.path().join("build").join("app").join("app"))
        }
        GraphOnlyLibrarySuccess::Emit => {
            assert_emit_c_source(&output);
            assert_no_build_dir(tmp.path(), "zen emit build.zen");
        }
        GraphOnlyLibrarySuccess::Test => {
            assert_test_binary_and_output(&tmp, &output, "unit");
        }
    }
    assert_graph_only_library_rejected(
        args,
        write_project,
        GraphOnlyLibrarySource::TypeError,
        LIBRARY_TYPE_ERROR_DIAGNOSTIC,
        command_label,
    );
}

fn run_graph_only_library_command(
    args: &[&str],
    write_project: GraphOnlyProjectWriter,
    source: GraphOnlyLibrarySource,
) -> (tempfile::TempDir, Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_project(&tmp, source);
    let output = run_zen_in(&tmp, args);
    (tmp, output)
}

fn assert_graph_only_library_rejected(
    args: &[&str],
    write_project: GraphOnlyProjectWriter,
    source: GraphOnlyLibrarySource,
    expected_diagnostic: &str,
    command_label: &str,
) {
    let (tmp, output) = run_graph_only_library_command(args, write_project, source);
    assert_zen_failure_contains(args, &output, expected_diagnostic);
    assert_no_build_dir(tmp.path(), command_label);
}

pub(crate) fn assert_no_build_dir(project_dir: &Path, command_label: &str) {
    assert!(
        !project_dir.join("build").exists(),
        "{command_label} should not create build outputs"
    );
}

pub(crate) fn assert_emit_c_source(output: &Output) {
    assert_stdout_contains(output, "int32_t zen_main(void)", "expected target C source");
}
