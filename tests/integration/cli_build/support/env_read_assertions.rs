use super::{
    assert_check_build_zen_summary, assert_no_build_dir, assert_stderr_lacks,
    assert_zen_failure_contains, build_graph_source, run_zen_in, write_file,
    write_zero_main_sources, LIBRARY_SOURCE, LIBRARY_TRUE, MAIN_TRUE, MAIN_ZERO,
};

const ENV_READ_DIAGNOSTIC: &str = "undeclared host effect: read env `ZEN_STD`";

const NO_SOURCES: &[(&str, &str)] = &[];
const NO_FORBIDDEN_STDERR: &[&str] = &[];
const RETURN_TYPE_MISMATCH: &[&str] = &["return type mismatch"];
const MISSING_TEST_SOURCE: &[&str] = &["missing_test.zen"];
const MISSING_APP_SOURCE: &[&str] = &["missing_app.zen"];
const SOURCE_NOT_FOUND: &[&str] = &["source not found"];

const SINGLE_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#];
const MISSING_EXECUTABLE_TARGETS: &[&str] =
    &[r#"    b.add(Executable { name: "myapp", main: "missing.zen", out_dir: "build/" })"#];
const MULTIPLE_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
];
const DEPENDENT_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })"#,
    r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
];
const APP_AND_LIBRARY_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];
const UNRELATED_TEST_TARGETS: &[&str] = &[
    r#"    b.add(Test { name: "unit", root: "missing_test.zen" })"#,
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
];
const SINGLE_TEST_TARGETS: &[&str] = &[r#"    b.add(Test { name: "unit", root: "test.zen" })"#];
const MULTIPLE_TEST_TARGETS: &[&str] = &[
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Test { name: "integration", root: "integration.zen" })"#,
];
const TEST_AND_LIBRARY_TARGETS: &[&str] = &[
    r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];
const UNRELATED_EXECUTABLE_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
];
const MIXED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];

const APP_SOURCE: &[(&str, &str)] = &[("app.zen", MAIN_ZERO)];
const MAIN_TRUE_SOURCE: &[(&str, &str)] = &[("main.zen", MAIN_TRUE)];
const APP_AND_TRUE_LIB_SOURCES: &[(&str, &str)] =
    &[("app.zen", MAIN_ZERO), ("lib.zen", LIBRARY_TRUE)];
const TEST_AND_TRUE_LIB_SOURCES: &[(&str, &str)] =
    &[("test.zen", MAIN_ZERO), ("lib.zen", LIBRARY_TRUE)];
const TEST_SOURCE: &[(&str, &str)] = &[("test.zen", MAIN_ZERO)];

pub(crate) type EnvReadRejectionCase<'a> = (&'a [&'a str], &'a [(&'a str, &'a str)], &'a [&'a str]);

pub(crate) const EXECUTABLE_ENV_READ_BASIC_CASES: &[EnvReadRejectionCase<'static>] = &[
    (SINGLE_EXECUTABLE_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
    (MULTIPLE_EXECUTABLE_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
];

pub(crate) const TEST_ENV_READ_BASIC_CASES: &[EnvReadRejectionCase<'static>] = &[
    (SINGLE_TEST_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
    (MULTIPLE_TEST_TARGETS, NO_SOURCES, NO_FORBIDDEN_STDERR),
];

pub(crate) const DEPENDENT_EXECUTABLE_ENV_READ_CASE: EnvReadRejectionCase<'static> = (
    DEPENDENT_EXECUTABLE_TARGETS,
    NO_SOURCES,
    NO_FORBIDDEN_STDERR,
);
pub(crate) const EXECUTABLE_LIBRARY_ENV_READ_CASE: EnvReadRejectionCase<'static> = (
    APP_AND_LIBRARY_TARGETS,
    APP_AND_TRUE_LIB_SOURCES,
    RETURN_TYPE_MISMATCH,
);
pub(crate) const UNRELATED_TEST_ENV_READ_CASE: EnvReadRejectionCase<'static> =
    (UNRELATED_TEST_TARGETS, APP_SOURCE, MISSING_TEST_SOURCE);
pub(crate) const TEST_LIBRARY_ENV_READ_CASE: EnvReadRejectionCase<'static> = (
    TEST_AND_LIBRARY_TARGETS,
    TEST_AND_TRUE_LIB_SOURCES,
    RETURN_TYPE_MISMATCH,
);
pub(crate) const UNRELATED_EXECUTABLE_ENV_READ_CASE: EnvReadRejectionCase<'static> = (
    UNRELATED_EXECUTABLE_TARGETS,
    TEST_SOURCE,
    MISSING_APP_SOURCE,
);
pub(crate) const MISSING_SOURCE_ENV_READ_CASE: EnvReadRejectionCase<'static> =
    (MISSING_EXECUTABLE_TARGETS, NO_SOURCES, SOURCE_NOT_FOUND);
pub(crate) const TYPE_MISMATCH_EXECUTABLE_ENV_READ_CASE: EnvReadRejectionCase<'static> = (
    SINGLE_EXECUTABLE_TARGETS,
    MAIN_TRUE_SOURCE,
    RETURN_TYPE_MISMATCH,
);

pub(crate) const DECLARED_ENV_READ_FALLBACK_ARMS: &[&str] = &[
    r#"| .Err { "~/.zen/std" }"#,
    r#"| _ { "~/.zen/std" }"#,
    r#"| err { "~/.zen/std" }"#,
];

pub(crate) fn assert_env_read_rejected(
    args: &[&str],
    case: &EnvReadRejectionCase<'_>,
    command_label: &str,
) -> std::process::Output {
    let (targets, sources, forbidden_stderr) = *case;
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_env_read_graph(&tmp, r#"std_path = b.os.env("ZEN_STD")"#, targets);
    for &(path, source) in sources {
        write_file(&tmp, path, source);
    }

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(args, &output, ENV_READ_DIAGNOSTIC);
    assert_stderr_lacks(
        &output,
        forbidden_stderr,
        "host-effect validation should run first",
    );
    assert_no_build_dir(tmp.path(), command_label);
    output
}

pub(crate) fn assert_declared_env_read_single_executable_check(fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_declared_env_read_graph(&tmp, fallback_arm, SINGLE_EXECUTABLE_TARGETS);
    write_zero_main_sources(&tmp, &["main.zen"]);
    assert_check_build_zen_summary(&tmp, "1 build targets");
}

pub(crate) fn assert_declared_env_read_mixed_target_check(fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_declared_env_read_graph(&tmp, fallback_arm, MIXED_TARGETS);
    write_zero_main_sources(&tmp, &["app.zen", "unit.zen"]);
    write_file(&tmp, "lib.zen", LIBRARY_SOURCE);
    assert_check_build_zen_summary(&tmp, "3 build targets");
}

fn write_declared_env_read_graph(tmp: &tempfile::TempDir, fallback_arm: &str, targets: &[&str]) {
    let env_read = format!(
        r#"std_path = b.os.env("ZEN_STD") ?
        | .Ok(path) {{ path }}
        {fallback_arm}"#,
    );
    write_env_read_graph(tmp, &env_read, targets);
}

fn write_env_read_graph(tmp: &tempfile::TempDir, env_read: &str, targets: &[&str]) {
    let body = format!("    {env_read}\n{}", targets.join("\n"));
    write_file(tmp, "build.zen", &build_graph_source(&[&body]));
}
