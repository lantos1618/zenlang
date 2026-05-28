use std::path::{Path, PathBuf};
use std::process::{Command, Output};
mod env_read_assertions;
mod file_read_assertions;
mod file_read_graphs;
mod file_read_rejections;
mod gated_dependency_graphs;
mod graph_only_libraries;
mod target_metadata_cases;
mod test_file_read_assertions;

pub(super) use env_read_assertions::*;
pub(super) use file_read_assertions::*;
pub(super) use file_read_rejections::*;
pub(super) use gated_dependency_graphs::*;
pub(super) use graph_only_libraries::*;
pub(super) use target_metadata_cases::*;
pub(super) use test_file_read_assertions::*;

pub(super) const BUILD_ARGS: &[&str] = &["build", "build.zen"];
pub(super) const DIRECT_ARGS: &[&str] = &["build.zen"];
pub(super) const BUILD_GRAPH_ARGS: &[&str] = &["build-graph", "build.zen"];
pub(super) const CHECK_ARGS: &[&str] = &["check", "build.zen"];
pub(super) const EMIT_ARGS: &[&str] = &["emit", "build.zen"];
pub(super) const TEST_ARGS: &[&str] = &["test", "build.zen"];
pub(super) const EXECUTABLE_ARGS: &[&[&str]] = &[BUILD_ARGS, DIRECT_ARGS, BUILD_GRAPH_ARGS];
pub(super) const EXECUTABLE_COMMAND_LABELS: &[(&[&str], &str)] = &[
    (BUILD_ARGS, "zen build build.zen"),
    (DIRECT_ARGS, "zen build.zen"),
    (BUILD_GRAPH_ARGS, "zen build-graph build.zen"),
];
pub(super) const BUILD_ZEN_VALIDATION_ARGS: &[&[&str]] = &[
    BUILD_ARGS,
    DIRECT_ARGS,
    CHECK_ARGS,
    TEST_ARGS,
    BUILD_GRAPH_ARGS,
];
pub(super) const ALL_BUILD_ZEN_COMMAND_ARGS: &[&[&str]] = &[
    BUILD_ARGS,
    DIRECT_ARGS,
    CHECK_ARGS,
    TEST_ARGS,
    EMIT_ARGS,
    BUILD_GRAPH_ARGS,
];

pub(super) fn write_single_executable_graph(tmp: &tempfile::TempDir) {
    write_target_graph(
        tmp,
        &[r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#],
    );
    write_zero_main_sources(tmp, &["main.zen"]);
}

fn write_target_graph(tmp: &tempfile::TempDir, targets: &[&str]) {
    write_file(tmp, "build.zen", &build_graph_source(targets));
}

pub(super) fn build_graph_source(targets: &[&str]) -> String {
    format!(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
{}
    .Ok(b.config())
}}
"#,
        targets.join("\n"),
    )
}

pub(super) fn run_zen_in(tmp: &tempfile::TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen command")
}

pub(super) fn run_zen(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .output()
        .expect("run zen command")
}

pub(super) fn run_emit_json_build_graph_source(source: &str) -> Output {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", source);
    run_zen_in(&tmp, &["emit-json", "build-graph", "build.zen"])
}

pub(super) fn run_emit_json_build_graph_targets(targets: &[&str]) -> Output {
    run_emit_json_build_graph_source(&build_graph_source(targets))
}

pub(super) fn run_build_zen_command(
    args: &[&str],
    targets: &[&str],
    files: &[(&str, &str)],
) -> (tempfile::TempDir, Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", &build_graph_source(targets));
    for (path, source) in files {
        write_file(&tmp, path, source);
    }
    let output = run_zen_in(&tmp, args);
    (tmp, output)
}

pub(super) fn assert_zen_success(args: &[&str], output: &Output) {
    assert_zen_status(args, output, true, "failed");
}

pub(super) fn assert_zen_failure(args: &[&str], output: &Output) {
    assert_zen_status(args, output, false, "unexpectedly succeeded");
}

fn assert_zen_status(args: &[&str], output: &Output, success: bool, message: &str) {
    assert!(
        output.status.success() == success,
        "zen {args:?} {message}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn assert_zen_failure_contains(args: &[&str], output: &Output, expected: &str) {
    assert_zen_failure(args, output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "expected diagnostic `{expected}`, stderr={stderr}"
    );
}

pub(super) fn assert_stderr_lacks(output: &Output, forbidden: &[&str], message: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    for forbidden in forbidden {
        assert!(
            !stderr.contains(forbidden),
            "{message}: {forbidden}, stderr={stderr}"
        );
    }
}

pub(super) fn assert_stdout_contains(output: &Output, expected: &str, message: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(expected), "{message}, stdout={stdout}");
}

pub(super) fn assert_stdout_empty(output: &Output, message: &str) {
    assert!(
        output.stdout.is_empty(),
        "{message}, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

pub(super) fn assert_path_exists(path: impl AsRef<Path>) {
    let path = path.as_ref();
    assert!(path.exists(), "expected {} to exist", path.display());
}

pub(super) fn assert_build_zen_rejected(
    args: &[&str],
    build_source: &str,
    expected_diagnostic: &str,
    command_label: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", build_source);
    assert_zen_rejected_without_build_outputs(&tmp, args, expected_diagnostic, command_label);
}

pub(super) fn assert_zen_rejected_without_build_outputs(
    tmp: &tempfile::TempDir,
    args: &[&str],
    expected_diagnostic: &str,
    command_label: &str,
) {
    let output = run_zen_in(tmp, args);
    assert_zen_failure_contains(args, &output, expected_diagnostic);
    assert_no_build_dir(tmp.path(), command_label);
}

pub(super) fn assert_check_build_zen_summary(tmp: &tempfile::TempDir, expected_summary: &str) {
    let output = run_zen_in(tmp, CHECK_ARGS);
    assert_zen_success(CHECK_ARGS, &output);
    assert_stdout_contains(
        &output,
        expected_summary,
        &format!("expected build graph check summary `{expected_summary}`"),
    );
    assert_no_build_dir(tmp.path(), "zen check build.zen");
}

pub(super) fn assert_built_binaries_run(paths: &[PathBuf]) {
    for bin_path in paths {
        assert_path_exists(bin_path);
        let run = Command::new(bin_path).output().expect("run built binary");
        assert!(
            run.status.success(),
            "built binary {} exited with {}",
            bin_path.display(),
            run.status
        );
    }
}

pub(super) fn assert_test_binary_and_output(tmp: &tempfile::TempDir, output: &Output, name: &str) {
    assert_path_exists(tmp.path().join("build").join("tests").join(name));
    assert_stdout_contains(
        output,
        &format!("test {name} passed"),
        "expected test pass output",
    );
}

pub(super) fn assert_single_executable_build(args: &[&str]) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_graph(&tmp);
    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_built_binaries_run(&[tmp.path().join("build").join("myapp")]);
}

pub(super) fn assert_multiple_executable_build(args: &[&str]) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_target_graph(
        &tmp,
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
        ],
    );
    write_zero_main_sources(&tmp, &["app.zen", "tool.zen"]);
    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_built_binaries_run(&[
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ]);
}

pub(super) fn assert_dependent_executable_build_order(args: &[&str]) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_target_graph(
        &tmp,
        &[
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })"#,
            r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
        ],
    );
    write_zero_main_sources(&tmp, &["app.zen", "tool.zen"]);
    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);

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

    assert_built_binaries_run(&[
        tmp.path().join("build").join("tool").join("tool"),
        tmp.path().join("build").join("app").join("app"),
    ]);
}

pub(super) fn assert_executable_build_rejects_no_executable(args: &[&str], command_label: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_target_graph(
        &tmp,
        &[r#"    b.add(Test { name: "unit", root: "test.zen" })"#],
    );
    write_zero_main_sources(&tmp, &["test.zen"]);
    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(
        args,
        &output,
        "build graph execution requires at least one executable target",
    );
    assert_no_build_dir(tmp.path(), command_label);
}

pub(super) fn assert_executable_build_accepts_library_dependency(args: &[&str]) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_target_graph(
        &tmp,
        &[
            r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })"#,
        ],
    );
    write_file(&tmp, "lib.zen", LIBRARY_SOURCE);
    write_zero_main_sources(&tmp, &["app.zen"]);
    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_path_exists(tmp.path().join("build").join("app").join("app"));
}

pub(super) fn assert_executable_build_ignores_unrelated_test_source(args: &[&str]) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_target_graph(
        &tmp,
        &[
            r#"    b.add(Test { name: "unit", root: "missing_test.zen" })"#,
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
        ],
    );
    write_zero_main_sources(&tmp, &["app.zen"]);
    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_path_exists(tmp.path().join("build").join("app").join("app"));
}

pub(super) fn write_file(tmp: &tempfile::TempDir, path: &str, source: &str) {
    std::fs::write(tmp.path().join(path), source).unwrap_or_else(|err| {
        panic!("write {path}: {err}");
    });
}

pub(super) fn write_zero_main_sources(tmp: &tempfile::TempDir, paths: &[&str]) {
    for path in paths {
        write_file(tmp, path, MAIN_ZERO);
    }
}

pub(super) const MAIN_ZERO: &str = "\nmain = () i32 {\n    0\n}\n";
pub(super) const MAIN_TRUE: &str = "\nmain = () i32 {\n    true\n}\n";

pub(super) const LIBRARY_SOURCE: &str = "\nvalue = () i32 {\n    1\n}\n";
pub(super) const LIBRARY_TRUE: &str = "\nvalue = () i32 {\n    true\n}\n";
