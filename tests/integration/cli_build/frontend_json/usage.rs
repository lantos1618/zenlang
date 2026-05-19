use crate::support::*;
use std::process::Command;

#[test]
fn emit_json_usage_lists_supported_and_gated_modes() {
    let missing_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("emit-json")
        .output()
        .expect("run zen emit-json without mode");
    assert!(
        !missing_mode_output.status.success(),
        "zen emit-json without mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&missing_mode_output.stdout),
        String::from_utf8_lossy(&missing_mode_output.stderr)
    );

    let unknown_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "unknown",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json unknown mode");
    assert!(
        !unknown_mode_output.status.success(),
        "zen emit-json unknown mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&unknown_mode_output.stdout),
        String::from_utf8_lossy(&unknown_mode_output.stderr)
    );

    let expected_usage =
        "Usage: zen emit-json <ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml> <file.zen>";
    for output in [&missing_mode_output, &unknown_mode_output] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_usage),
            "emit-json usage should list supported and gated modes, stderr={stderr}"
        );
    }
}
