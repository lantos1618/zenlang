use super::fixture;
use std::process::Command;

fn assert_hir_golden(source: &str, golden: &str, description: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", fixture(source).to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json hir on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked {description} HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("HIR {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("HIR {description} stdout is JSON: {err}"));
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_result_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_result_enum.zen",
        "tests/fixtures/ir_json/hir_generic_result.golden.json",
        "generic Result program",
    );
}

#[test]
fn emit_json_hir_generic_option_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_enum_option.zen",
        "tests/fixtures/ir_json/hir_generic_option.golden.json",
        "generic Option program",
    );
}

#[test]
fn emit_json_hir_generic_option_multi_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/hir_generic_option_multi.golden.json",
        "generic Option multi-specialization",
    );
}

#[test]
fn emit_json_hir_generic_result_multi_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_result_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/hir_generic_result_multi.golden.json",
        "generic Result multi-specialization",
    );
}
