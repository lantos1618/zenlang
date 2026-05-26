use super::{main_source, write_file};
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

    write_file(
        tmp,
        "build.zen",
        &format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
{target_add}
    b.add(Library {{ name: "core", exports: ["{export}"] }})
    .Ok(b.config())
}}
"#,
        ),
    );
    write_file(tmp, entry_source, main_source("0").as_str());

    match source {
        GraphOnlyLibrarySource::Missing => {}
        GraphOnlyLibrarySource::Valid => write_file(tmp, "lib.zen", function_source("1").as_str()),
        GraphOnlyLibrarySource::TypeError => {
            write_file(tmp, "lib.zen", function_source("true").as_str());
        }
    }
}

pub(crate) fn assert_graph_only_library_build_output(project_dir: &Path) {
    assert!(
        project_dir.join("build").join("app").join("app").exists(),
        "expected executable output to exist"
    );
}

pub(crate) fn assert_graph_only_library_test_output(project_dir: &Path, output: &Output) {
    assert!(
        project_dir
            .join("build")
            .join("tests")
            .join("unit")
            .exists(),
        "expected test binary output to exist"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test unit passed"),
        "expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

pub(crate) fn assert_no_build_dir(project_dir: &Path, command_label: &str) {
    assert!(
        !project_dir.join("build").exists(),
        "{command_label} should not create build outputs"
    );
}

pub(crate) fn assert_emit_c_source(output: &Output) {
    let c_source = String::from_utf8_lossy(&output.stdout);
    assert!(
        c_source.contains("int32_t zen_main(void)"),
        "expected target C source, stdout={c_source}"
    );
}

fn function_source(value: &str) -> String {
    format!(
        r#"
value = () i32 {{
    {value}
}}
"#,
    )
}
