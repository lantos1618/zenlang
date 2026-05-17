use std::process::Command;

#[path = "legacy_graph_command/execution.rs"]
mod execution;

#[test]
fn cli_usage_describes_build_graph_executable_targets() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .output()
        .expect("run zen without args");

    assert!(
        !output.status.success(),
        "zen without args unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build-graph <build.zen>   Compile executable targets"),
        "expected build-graph plural target usage, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    for mode in ["ast", "symbols", "typed", "diagnostics"] {
        let output = Command::new(env!("CARGO_BIN_EXE_zen"))
            .args(["emit-json", mode, "build.zen"])
            .current_dir(tmp.path())
            .output()
            .unwrap_or_else(|err| panic!("run zen emit-json {mode} build.zen: {err}"));

        assert!(
            !output.status.success(),
            "zen emit-json {mode} build.zen unexpectedly succeeded: stdout={}, stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(
                "this emit-json mode does not support build.zen; use `emit-json build-graph`"
            ),
            "expected build graph diagnostic for emit-json {mode}, stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
