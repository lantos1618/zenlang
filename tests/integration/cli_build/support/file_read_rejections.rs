use super::{
    assert_stderr_lacks, assert_zen_failure_contains, build_graph_source, run_zen_in, write_file,
};

const FILE_READ_DIAGNOSTIC: &str = "undeclared host effect: read file `build.targets`";
const UNDECLARED_FILE_READ: &str = r#"manifest = b.os.read_file("build.targets")"#;
const MISSING_FALLBACK_FILE_READ: &str = r#"manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }"#;
const SINGLE_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#];
const UNSELECTED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "missing_unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["missing_lib.zen"] })"#,
];

pub(crate) fn assert_check_file_read_rejected_before_source_validation(
    args: &[&str],
    source: impl AsRef<str>,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", source.as_ref());

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(args, &output, FILE_READ_DIAGNOSTIC);
    assert_stderr_lacks(
        &output,
        &["source not found"],
        "host-effect validation should run before source validation",
    );
}

pub(crate) fn undeclared_single_executable_file_read_graph() -> String {
    file_read_graph(UNDECLARED_FILE_READ, SINGLE_EXECUTABLE_TARGETS)
}

pub(crate) fn missing_fallback_single_executable_file_read_graph() -> String {
    file_read_graph(MISSING_FALLBACK_FILE_READ, SINGLE_EXECUTABLE_TARGETS)
}

pub(crate) fn undeclared_unselected_file_read_graph() -> String {
    file_read_graph(UNDECLARED_FILE_READ, UNSELECTED_TARGETS)
}

pub(crate) fn missing_fallback_unselected_file_read_graph() -> String {
    file_read_graph(MISSING_FALLBACK_FILE_READ, UNSELECTED_TARGETS)
}

fn file_read_graph(effect_line: &str, targets: &[&str]) -> String {
    build_graph_source(&[&format!("    {effect_line}\n{}", targets.join("\n"))])
}
