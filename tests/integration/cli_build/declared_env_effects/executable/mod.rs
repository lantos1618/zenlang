use super::super::support::{
    BUILD_ARGS, BUILD_GRAPH_ARGS, DIRECT_ARGS, EMIT_ARGS, EXECUTABLE_ARGS,
};

const SINGLE_TARGET_COMMANDS: &[(&[&str], &str, super::ExecutableCommandExpectation)] = &[
    (
        BUILD_ARGS,
        "build_command",
        super::ExecutableCommandExpectation::BuildOutput,
    ),
    (
        BUILD_GRAPH_ARGS,
        "build_graph",
        super::ExecutableCommandExpectation::BuildOutput,
    ),
    (
        DIRECT_ARGS,
        "direct_file",
        super::ExecutableCommandExpectation::BuildOutput,
    ),
    (
        EMIT_ARGS,
        "emit",
        super::ExecutableCommandExpectation::EmitStdout,
    ),
];
#[test]
fn executable_commands_accept_declared_env_read_fallbacks() {
    for (args, label, expectation) in SINGLE_TARGET_COMMANDS {
        for (case, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
            super::assert_executable_command_accepts_declared_env_read(
                args,
                fallback_arm,
                &format!("{label}_single_target_{case}_fallback"),
                *expectation,
            );
        }
    }
}

#[test]
fn executable_commands_accept_declared_env_read_for_multiple_targets() {
    for args in EXECUTABLE_ARGS {
        for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
            assert_executable_command_accepts_declared_env_read_for_multiple_targets(
                args,
                fallback_arm,
            );
        }
    }
}

#[test]
fn executable_commands_accept_declared_env_read_with_unselected_targets() {
    for args in EXECUTABLE_ARGS {
        for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
            assert_executable_command_accepts_declared_env_read_with_unselected_targets(
                args,
                fallback_arm,
            );
        }
    }
    for (_, fallback_arm) in super::DECLARED_ENV_READ_FALLBACK_CASES {
        let (tmp, output) = super::run_declared_env_read_command(
            EMIT_ARGS,
            fallback_arm,
            super::EXECUTABLE_WITH_VALID_UNSELECTED_TARGETS,
            super::APP_UNIT_AND_LIB_SOURCES,
        );
        super::assert_stdout_contains(
            &output,
            "int32_t zen_main(void)",
            "expected C output after declared env effect",
        );
        super::assert_no_build_dir(tmp.path(), "zen emit build.zen");
    }
}

fn assert_executable_command_accepts_declared_env_read_with_unselected_targets(
    args: &[&str],
    fallback_arm: &str,
) {
    let (tmp, _) = super::run_declared_env_read_command(
        args,
        fallback_arm,
        super::EXECUTABLE_WITH_MISSING_TEST_TARGETS,
        super::APP_AND_LIB_SOURCES,
    );

    let bin_path = tmp.path().join("build").join("app").join("app");
    super::assert_path_exists(bin_path);
}

fn assert_executable_command_accepts_declared_env_read_for_multiple_targets(
    args: &[&str],
    fallback_arm: &str,
) {
    let (tmp, _) = super::run_declared_env_read_command(
        args,
        fallback_arm,
        super::MULTIPLE_EXECUTABLE_TARGETS,
        super::APP_TOOL_SOURCES,
    );

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        super::assert_path_exists(bin_path);
    }
}
