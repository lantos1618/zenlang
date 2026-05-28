use super::super::support::{assert_zen_success, run_zen_in, TEST_ARGS};

const SINGLE_TEST_TARGETS: &[&str] = &[r#"    b.add(Test { name: "unit", root: "unit.zen" })"#];
const MULTIPLE_TEST_TARGETS: &[&str] = &[
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Test { name: "integration", root: "integration.zen" })"#,
];
const TEST_WITH_UNSELECTED_TARGETS: &[&str] = &[
    r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
    r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
    r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
];

const UNIT_SOURCE: &[(&str, &str)] = &[("unit.zen", super::MAIN_ZERO)];
const UNIT_AND_INTEGRATION_SOURCES: &[(&str, &str)] = &[
    ("unit.zen", super::MAIN_ZERO),
    ("integration.zen", super::MAIN_ZERO),
];
const UNIT_AND_LIB_SOURCES: &[(&str, &str)] =
    &[("unit.zen", super::MAIN_ZERO), ("lib.zen", super::LIB_ONE)];

#[test]
fn test_command_build_zen_accepts_declared_env_read_fallbacks() {
    for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
        let output = run_test_command_build_zen(fallback_arm, SINGLE_TEST_TARGETS, UNIT_SOURCE);
        assert_test_output_contains(&output, &["unit"]);
    }
}

#[test]
fn test_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
        let output = run_test_command_build_zen(
            fallback_arm,
            MULTIPLE_TEST_TARGETS,
            UNIT_AND_INTEGRATION_SOURCES,
        );
        assert_test_output_contains(&output, &["unit", "integration"]);
    }
}

#[test]
fn test_command_build_zen_accepts_declared_env_read_with_unselected_targets() {
    for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
        let output = run_test_command_build_zen(
            fallback_arm,
            TEST_WITH_UNSELECTED_TARGETS,
            UNIT_AND_LIB_SOURCES,
        );
        assert_test_output_contains(&output, &["unit"]);
    }
}

fn run_test_command_build_zen(
    fallback_arm: &str,
    targets: &[&str],
    files: &[(&str, &str)],
) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("create temp dir");
    super::write_declared_env_read_graph(&tmp, fallback_arm, targets);
    super::write_sources(&tmp, files);

    let output = run_zen_in(&tmp, TEST_ARGS);
    assert_zen_success(TEST_ARGS, &output);
    output
}

fn assert_test_output_contains(output: &std::process::Output, test_names: &[&str]) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    for test_name in test_names {
        assert!(
            stdout.contains(&format!("test {test_name} passed")),
            "expected {test_name} to pass, stdout={stdout}"
        );
    }
}
