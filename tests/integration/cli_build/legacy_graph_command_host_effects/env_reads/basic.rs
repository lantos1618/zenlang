#[test]
fn build_graph_command_rejects_undeclared_host_effects() {
    let args = ["build-graph", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("main.zen"), main_source("0")).expect("write main.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    assert!(
        !tmp.path().join("build").exists(),
        "build-graph command should not start after graph validation fails"
    );
}

#[test]
fn build_graph_command_multi_target_rejects_undeclared_host_effects() {
    let args = ["build-graph", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    assert!(
        !tmp.path().join("build").exists(),
        "multi-target build-graph command should not start after graph validation fails"
    );
}

fn main_source(value: &str) -> String {
    format!(
        r#"
main = () i32 {{
    {value}
}}
"#,
    )
}
