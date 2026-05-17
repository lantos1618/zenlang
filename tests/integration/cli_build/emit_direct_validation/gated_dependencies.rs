use std::process::Command;

#[test]
fn emit_command_build_zen_rejects_gated_test_dependencies() {
    assert_emit_rejects_gated_dependency(
        r#"b.add(Test { name: "unit", root: "test.zen" })"#,
        "unit",
        "test.zen",
        "test",
    );
}

#[test]
fn emit_command_build_zen_rejects_transitive_gated_test_dependencies() {
    let tmp = super::super::support::transitive_gated_test_dependency_graph();

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        !output.status.success(),
        "zen emit build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `core` depends on gated test target `unit`"),
        "expected transitive gated test dependency diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs after transitive gated dependency validation fails"
    );
}

fn assert_emit_rejects_gated_dependency(
    gated_target_decl: &str,
    gated_target_name: &str,
    gated_source_name: &str,
    gated_target_kind: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    {gated_target_decl}
    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["{gated_target_name}"],
    }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join(gated_source_name),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write gated target source");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        !output.status.success(),
        "zen emit build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(&format!(
            "build graph target `app` depends on gated {gated_target_kind} target `{gated_target_name}`"
        )),
        "expected gated dependency diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs after gated dependency validation fails"
    );
}
