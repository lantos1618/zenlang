#[test]
fn build_zen_commands_reject_unsupported_target_fields() {
    for args in super::ALL_BUILD_ZEN_COMMAND_ARGS {
        for (field, value, expected, reason) in [
            (
                "packages",
                r#"["std"]"#,
                "unsupported field `packages` in `Executable` build target; package semantics are gated",
                "gated target field",
            ),
            (
                "output_dir",
                r#""build/app/""#,
                "unknown field `output_dir` in `Executable` build target",
                "unknown target field",
            ),
        ] {
            assert_build_zen_command_rejects_target_field(args, field, value, expected, reason);
        }
    }
}

fn assert_build_zen_command_rejects_target_field(
    args: &[&str],
    field: &str,
    value: &str,
    expected: &str,
    reason: &str,
) {
    let target_add = format!(
        r#"    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})"#,
    );
    let (tmp, output) = super::run_build_zen_command(args, &[&target_add], &[]);

    super::assert_rejected_without_outputs(&tmp, &output, args, expected.to_string(), reason);
}
