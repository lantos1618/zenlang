#[test]
fn build_zen_commands_reject_package_fields() {
    assert_build_zen_commands_reject_gated_target_field("packages", r#"["std"]"#);
}

#[test]
fn build_zen_commands_reject_link_fields() {
    assert_build_zen_commands_reject_gated_target_field("link", r#"["m"]"#);
}

#[test]
fn build_zen_commands_reject_unknown_target_fields() {
    assert_build_zen_commands_reject_unknown_target_field("output_dir", r#""build/app/""#);
}

fn assert_build_zen_commands_reject_gated_target_field(field: &str, value: &str) {
    for args in super::all_build_zen_command_args() {
        assert_build_zen_command_rejects_gated_target_field(args, field, value);
    }
}

fn assert_build_zen_command_rejects_gated_target_field(args: &[&str], field: &str, value: &str) {
    let (tmp, output) = super::run_build_zen_command(
        args,
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})
    .Ok(b.config())
}}
"#
        ),
    );

    super::assert_rejected_without_outputs(
        &tmp,
        &output,
        args,
        format!("unsupported field `{field}` in `Executable` build target; package/link semantics are gated"),
        "gated target field",
    );
}

fn assert_build_zen_commands_reject_unknown_target_field(field: &str, value: &str) {
    for args in super::all_build_zen_command_args() {
        assert_build_zen_command_rejects_unknown_target_field(args, field, value);
    }
}

fn assert_build_zen_command_rejects_unknown_target_field(args: &[&str], field: &str, value: &str) {
    let (tmp, output) = super::run_build_zen_command(
        args,
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})
    .Ok(b.config())
}}
"#
        ),
    );

    super::assert_rejected_without_outputs(
        &tmp,
        &output,
        args,
        format!("unknown field `{field}` in `Executable` build target"),
        "unknown target field",
    );
}
