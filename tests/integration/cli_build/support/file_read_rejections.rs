use super::{
    assert_build_zen_rejected, assert_no_build_dir, assert_stderr_lacks, assert_stdout_empty,
    assert_zen_failure_contains, build_graph_source, run_zen_in, write_file,
};

const FILE_READ_DIAGNOSTIC: &str = "undeclared host effect: read file `build.targets`";
const UNDECLARED_FILE_READ: &str = r#"manifest = b.os.read_file("build.targets")"#;
const MISSING_FALLBACK_FILE_READ: &str = r#"manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }"#;
const SINGLE_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#];
const MULTIPLE_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
];
const UNSELECTED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "missing_unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["missing_lib.zen"] })"#,
];

pub(crate) fn assert_build_file_read_rejected(
    args: &[&str],
    source: impl AsRef<str>,
    command_label: &str,
) {
    assert_build_zen_rejected(args, source.as_ref(), FILE_READ_DIAGNOSTIC, command_label);
}

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

pub(crate) fn assert_emit_file_read_rejected(args: &[&str], source: impl AsRef<str>) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", source.as_ref());

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(args, &output, FILE_READ_DIAGNOSTIC);
    assert_stdout_empty(
        &output,
        "emit should not write C source after graph validation fails",
    );
    assert_no_build_dir(tmp.path(), &format!("zen {}", args.join(" ")));
}

pub(crate) fn assert_emit_file_read_rejected_before_unselected_targets(
    args: &[&str],
    source: impl AsRef<str>,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", source.as_ref());

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(args, &output, FILE_READ_DIAGNOSTIC);
    assert_stderr_lacks(
        &output,
        &["missing_app.zen", "missing_unit.zen", "missing_lib.zen"],
        "emit should reject file reads before target source validation",
    );
    assert_stdout_empty(
        &output,
        "emit should not write C source after graph validation fails",
    );
    assert_no_build_dir(tmp.path(), &format!("zen {}", args.join(" ")));
}

pub(crate) fn undeclared_single_executable_file_read_graph() -> String {
    file_read_graph(UNDECLARED_FILE_READ, SINGLE_EXECUTABLE_TARGETS)
}

pub(crate) fn missing_fallback_single_executable_file_read_graph() -> String {
    file_read_graph(MISSING_FALLBACK_FILE_READ, SINGLE_EXECUTABLE_TARGETS)
}

pub(crate) fn undeclared_multiple_executable_file_read_graph() -> String {
    file_read_graph(UNDECLARED_FILE_READ, MULTIPLE_EXECUTABLE_TARGETS)
}

pub(crate) fn missing_fallback_multiple_executable_file_read_graph() -> String {
    file_read_graph(MISSING_FALLBACK_FILE_READ, MULTIPLE_EXECUTABLE_TARGETS)
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
