use super::support::{
    assert_stdout_empty, assert_zen_failure_contains, run_emit_json_build_graph_targets,
};
mod file_reads;

const ENV_READ_DIAGNOSTIC: &str = "undeclared host effect: read env `ZEN_STD`";
const BUILD_GRAPH_JSON_ARGS: &[&str] = &["emit-json", "build-graph", "build.zen"];

#[test]
fn emit_json_build_graph_rejects_undeclared_env_effects_before_lowering() {
    for target_add in [
        r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#,
        r#"    b.add(Test { root: "test.zen" })"#,
        r#"    b.add(Library { name: "core", exports: ["src/math.zen"] })"#,
        r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
        features: ["release"],
    })"#,
    ] {
        let target = env_read_target(target_add);
        assert_emit_json_build_graph_failure_contains(&[&target], ENV_READ_DIAGNOSTIC);
    }
}

fn env_read_target(target_add: &str) -> String {
    format!("    std_path = b.os.env(\"ZEN_STD\")\n{target_add}")
}

fn assert_emit_json_build_graph_failure_contains(
    targets: &[&str],
    expected_diagnostic: &str,
) -> std::process::Output {
    let output = run_emit_json_build_graph_targets(targets);
    assert_zen_failure_contains(BUILD_GRAPH_JSON_ARGS, &output, expected_diagnostic);
    output
}
