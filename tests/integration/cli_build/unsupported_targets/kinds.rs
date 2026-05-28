#[test]
fn build_zen_commands_reject_unsupported_target_kinds() {
    for args in super::ALL_BUILD_ZEN_COMMAND_ARGS {
        for target_kind in ["Package", "Link"] {
            assert_build_zen_command_rejects_unsupported_target_kind(args, target_kind);
        }
    }
}

fn assert_build_zen_command_rejects_unsupported_target_kind(args: &[&str], target_kind: &str) {
    let target_add = format!(r#"    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})"#);
    let (tmp, output) = super::run_build_zen_command(args, &[&target_add], &[]);

    super::assert_rejected_without_outputs(
        &tmp,
        &output,
        args,
        format!(
            "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
        ),
        "unsupported target kind",
    );
}
