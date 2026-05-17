use std::process::Command;

#[test]
fn build_graph_command_compiles_single_executable_target() {
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
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write main.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph");

    assert!(
        output.status.success(),
        "zen build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bin_path = tmp.path().join("build").join("myapp");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );
    let run = Command::new(&bin_path).output().expect("run built binary");
    assert!(
        run.status.success(),
        "built binary exited with {}",
        run.status
    );
}

#[test]
fn build_graph_command_compiles_multiple_executable_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    std::fs::write(
        tmp.path().join("tool.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write tool.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph");

    assert!(
        output.status.success(),
        "zen build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
        let run = Command::new(&bin_path).output().expect("run built binary");
        assert!(
            run.status.success(),
            "built binary {} exited with {}",
            bin_path.display(),
            run.status
        );
    }
}

#[test]
fn build_graph_command_compiles_executable_dependencies_first() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    std::fs::write(
        tmp.path().join("tool.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write tool.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph");

    assert!(
        output.status.success(),
        "zen build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tool_emit = stdout
        .find("build/tool/tool.c")
        .unwrap_or_else(|| panic!("expected tool emission in stdout={stdout}"));
    let app_emit = stdout
        .find("build/app/app.c")
        .unwrap_or_else(|| panic!("expected app emission in stdout={stdout}"));
    assert!(
        tool_emit < app_emit,
        "expected dependency target to compile before dependent target, stdout={stdout}"
    );

    for bin_path in [
        tmp.path().join("build").join("tool").join("tool"),
        tmp.path().join("build").join("app").join("app"),
    ] {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
        let run = Command::new(&bin_path).output().expect("run built binary");
        assert!(
            run.status.success(),
            "built binary {} exited with {}",
            bin_path.display(),
            run.status
        );
    }
}
