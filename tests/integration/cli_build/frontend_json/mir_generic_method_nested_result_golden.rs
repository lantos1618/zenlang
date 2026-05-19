use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_mir_generic_method_nested_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_method_nested_result.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic method nested result input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic method nested result MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR generic method nested result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic method nested result stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/mir_generic_method_nested_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
