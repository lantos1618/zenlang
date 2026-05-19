use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_hir_generic_ufc_dedup_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_ufc_dedup.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic UFC dedup input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic UFC dedup HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("HIR generic UFC dedup stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic UFC dedup stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_ufc_dedup.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
