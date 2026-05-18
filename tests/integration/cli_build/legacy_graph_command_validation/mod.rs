#[path = "dependency_shapes.rs"]
mod dependency_shapes;
#[path = "execution_scope.rs"]
mod execution_scope;
#[path = "gated_dependencies.rs"]
mod gated_dependencies;
#[path = "graph_only_libraries.rs"]
mod graph_only_libraries;
#[path = "target_metadata.rs"]
mod target_metadata;

use std::process::Command;

fn assert_build_graph_command_rejects_dependency_shape(
    project_dir: &std::path::Path,
    expected_diagnostic: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(project_dir)
        .output()
        .expect("run zen build-graph");

    assert!(
        !output.status.success(),
        "zen build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected dependency diagnostic `{expected_diagnostic}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !project_dir.join("build").exists(),
        "build-graph command should not create outputs after dependency validation fails"
    );
}

fn assert_build_graph_command_rejects_target_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph");

    assert!(
        !output.status.success(),
        "zen build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_diagnostic),
        "expected target metadata diagnostic `{expected_diagnostic}`, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "build-graph command should not create outputs after target metadata validation fails"
    );
}
