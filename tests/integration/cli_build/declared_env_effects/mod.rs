mod executable;
mod rejections;
mod test_command;

use std::process::Output;

use super::support::{
    assert_no_build_dir, assert_path_exists, assert_stdout_contains, assert_zen_success,
    build_graph_source, run_zen_in, write_file, LIBRARY_SOURCE as LIB_ONE, MAIN_ZERO,
};

#[derive(Clone, Copy)]
enum ExecutableCommandExpectation {
    BuildOutput,
    EmitStdout,
}

const MAIN_SOURCE: &[(&str, &str)] = &[("main.zen", MAIN_ZERO)];
const APP_AND_LIB_SOURCES: &[(&str, &str)] = &[("app.zen", MAIN_ZERO), ("lib.zen", LIB_ONE)];
const APP_TOOL_SOURCES: &[(&str, &str)] = &[("app.zen", MAIN_ZERO), ("tool.zen", MAIN_ZERO)];
const APP_UNIT_AND_LIB_SOURCES: &[(&str, &str)] = &[
    ("app.zen", MAIN_ZERO),
    ("unit.zen", MAIN_ZERO),
    ("lib.zen", LIB_ONE),
];

const DECLARED_ENV_READ_FALLBACK_CASES: &[(&str, &str)] = &[
    ("declared", r#"| .Err { "default" }"#),
    ("wildcard", r#"| _ { "default" }"#),
    ("identifier", r#"| err { "default" }"#),
];

const SINGLE_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "app", main: "main.zen", out_dir: "build/" })"#];
const MULTIPLE_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
];
const EXECUTABLE_WITH_MISSING_TEST_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "missing_unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];
const EXECUTABLE_WITH_VALID_UNSELECTED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];

fn assert_executable_command_accepts_declared_env_read(
    args: &[&str],
    fallback_arm: &str,
    case_name: &str,
    expectation: ExecutableCommandExpectation,
) {
    let (tmp, output) =
        run_declared_env_read_command(args, fallback_arm, SINGLE_EXECUTABLE_TARGETS, MAIN_SOURCE);
    match expectation {
        ExecutableCommandExpectation::BuildOutput => {
            assert_path_exists(tmp.path().join("build").join("app"));
        }
        ExecutableCommandExpectation::EmitStdout => {
            assert_stdout_contains(
                &output,
                "int32_t zen_main(void)",
                &format!("{case_name}: expected C output after declared env effect"),
            );
            assert_no_build_dir(tmp.path(), &format!("{case_name}: zen emit build.zen"));
        }
    }
}

fn run_declared_env_read_command(
    args: &[&str],
    fallback_arm: &str,
    targets: &[&str],
    sources: &[(&str, &str)],
) -> (tempfile::TempDir, Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_declared_env_read_graph(&tmp, fallback_arm, targets);
    write_sources(&tmp, sources);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    (tmp, output)
}

fn write_declared_env_read_graph(tmp: &tempfile::TempDir, fallback_arm: &str, targets: &[&str]) {
    let body = format!(
        r#"    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
{}"#,
        targets.join("\n"),
    );
    write_file(tmp, "build.zen", &build_graph_source(&[&body]));
}

fn write_sources(tmp: &tempfile::TempDir, sources: &[(&str, &str)]) {
    for &(path, source) in sources {
        write_file(tmp, path, source);
    }
}
