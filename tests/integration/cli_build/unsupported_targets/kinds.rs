#[test]
fn build_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build", "build.zen"], "Package");
}

#[test]
fn build_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build", "build.zen"], "Link");
}

#[test]
fn direct_file_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build.zen"], "Package");
}

#[test]
fn direct_file_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build.zen"], "Link");
}

#[test]
fn check_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["check", "build.zen"], "Package");
}

#[test]
fn check_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["check", "build.zen"], "Link");
}

#[test]
fn test_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["test", "build.zen"], "Package");
}

#[test]
fn test_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["test", "build.zen"], "Link");
}

#[test]
fn emit_command_build_zen_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["emit", "build.zen"], "Package");
}

#[test]
fn emit_command_build_zen_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["emit", "build.zen"], "Link");
}

#[test]
fn build_graph_command_rejects_unsupported_package_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(
        &["build-graph", "build.zen"],
        "Package",
    );
}

#[test]
fn build_graph_command_rejects_unsupported_link_targets() {
    assert_build_zen_command_rejects_unsupported_target_kind(&["build-graph", "build.zen"], "Link");
}

fn assert_build_zen_command_rejects_unsupported_target_kind(args: &[&str], target_kind: &str) {
    let (tmp, output) = super::run_build_zen_command(
        args,
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})
    .Ok(b.config())
}}
"#
        ),
    );

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
