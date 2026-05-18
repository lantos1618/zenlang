use std::process::Command;

#[test]
fn root_usage_lists_supported_and_gated_emit_json_modes() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .output()
        .expect("run zen without args");

    assert!(
        !output.status.success(),
        "zen without args should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "emit-json ast <file>   Emit unchecked AST JSON",
        "emit-json symbols <file>",
        "emit-json typed <file>",
        "emit-json diagnostics <file>",
        "emit-json build-graph <build.zen>",
        "emit-json hir <file>   Emit checked HIR JSON",
        "emit-json mir <file>   Gated MIR JSON",
        "emit-json layout <file>   Emit checked type layout JSON",
        "emit-json target-yaml <file>   Validate target YAML",
    ] {
        assert!(
            stderr.contains(expected),
            "root usage should list `{expected}`, stderr={stderr}"
        );
    }
}
