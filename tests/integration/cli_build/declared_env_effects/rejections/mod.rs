use super::super::support::{
    assert_no_build_dir, assert_stderr_lacks, assert_stdout_empty, assert_zen_failure_contains,
    run_zen_in, BUILD_ARGS, BUILD_GRAPH_ARGS, CHECK_ARGS, DIRECT_ARGS, EMIT_ARGS, TEST_ARGS,
};

const ENV_READ_DIAGNOSTIC: &str = "undeclared host effect: read env `ZEN_STD`";

const NO_SOURCES: &[(&str, &str)] = &[];
const NO_FORBIDDEN_STDERR: &[&str] = &[];
const APP_SOURCE: &[(&str, &str)] = &[("app.zen", super::MAIN_ZERO)];
const TEST_AND_LIB_SOURCES: &[(&str, &str)] =
    &[("unit.zen", super::MAIN_ZERO), ("lib.zen", super::LIB_ONE)];

const SOURCE_NOT_FOUND: &[&str] = &["source not found"];
const MISSING_UNIT_SOURCE: &[&str] = &["missing_unit.zen"];
const MISSING_APP_SOURCE: &[&str] = &["missing_app.zen"];

const MISSING_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "app", main: "missing.zen", out_dir: "build/" })"#];
const MULTIPLE_MISSING_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "missing_unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["missing_lib.zen"] })"#,
];
const EMIT_UNSELECTED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "missing_test.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["missing_lib.zen"] })"#,
];
const SINGLE_TEST_TARGETS: &[&str] = &[r#"    b.add(Test { name: "unit", root: "test.zen" })"#];
const MULTIPLE_TEST_TARGETS: &[&str] = &[
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Test { name: "integration", root: "integration.zen" })"#,
];
const UNSELECTED_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];

type EnvReadGraphCase = (
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
    &'static [&'static str],
);

const EXECUTABLE_REJECTION_CASES: &[EnvReadGraphCase] = &[
    (
        super::SINGLE_EXECUTABLE_TARGETS,
        NO_SOURCES,
        NO_FORBIDDEN_STDERR,
    ),
    (
        super::MULTIPLE_EXECUTABLE_TARGETS,
        NO_SOURCES,
        NO_FORBIDDEN_STDERR,
    ),
    (
        super::EXECUTABLE_WITH_MISSING_TEST_TARGETS,
        super::APP_AND_LIB_SOURCES,
        MISSING_UNIT_SOURCE,
    ),
];
const CHECK_REJECTION_CASES: &[EnvReadGraphCase] = &[
    (MISSING_EXECUTABLE_TARGETS, NO_SOURCES, SOURCE_NOT_FOUND),
    (MULTIPLE_MISSING_TARGETS, NO_SOURCES, SOURCE_NOT_FOUND),
];
const EMIT_REJECTION_CASES: &[EnvReadGraphCase] = &[
    (
        super::SINGLE_EXECUTABLE_TARGETS,
        super::MAIN_SOURCE,
        NO_FORBIDDEN_STDERR,
    ),
    (EMIT_UNSELECTED_TARGETS, APP_SOURCE, SOURCE_NOT_FOUND),
];
const TEST_REJECTION_CASES: &[EnvReadGraphCase] = &[
    (SINGLE_TEST_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
    (
        UNSELECTED_EXECUTABLE_TARGETS,
        TEST_AND_LIB_SOURCES,
        MISSING_APP_SOURCE,
    ),
    (MULTIPLE_TEST_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
];

#[test]
fn declared_env_effect_commands_reject_env_read_without_fallback() {
    for (args, label) in [
        (BUILD_ARGS, "zen build build.zen"),
        (BUILD_GRAPH_ARGS, "zen build-graph build.zen"),
        (DIRECT_ARGS, "zen build.zen"),
    ] {
        for case in EXECUTABLE_REJECTION_CASES {
            assert_env_read_without_fallback_rejected(args, case, label, false);
        }
    }

    for (args, label, cases, expect_empty_stdout) in [
        (
            CHECK_ARGS,
            "zen check build.zen",
            CHECK_REJECTION_CASES,
            false,
        ),
        (EMIT_ARGS, "zen emit build.zen", EMIT_REJECTION_CASES, true),
        (TEST_ARGS, "zen test build.zen", TEST_REJECTION_CASES, false),
    ] {
        for case in cases {
            assert_env_read_without_fallback_rejected(args, case, label, expect_empty_stdout);
        }
    }
}

fn assert_env_read_without_fallback_rejected(
    args: &[&str],
    case: &EnvReadGraphCase,
    command_label: &str,
    expect_empty_stdout: bool,
) {
    let (targets, sources, forbidden_stderr) = *case;
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::write_declared_env_read_graph(&tmp, "", targets);
    super::write_sources(&tmp, sources);

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(args, &output, ENV_READ_DIAGNOSTIC);
    assert_stderr_lacks(
        &output,
        forbidden_stderr,
        "host-effect validation should run first",
    );
    if expect_empty_stdout {
        assert_stdout_empty(
            &output,
            &format!("{command_label} should not write C source after graph validation fails"),
        );
    }
    assert_no_build_dir(tmp.path(), command_label);
}
